use common::{load_app, LoadedApp};
use distributed::{
    init_logger,
    service::{map_reduce_client::*, task::Inner, *},
    temp_file, ADDR,
};
use eyre::Result;
use futures::{future::try_join_all, FutureExt};
use itertools::Itertools;
use log::info;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::Debug,
    hash::{Hash, Hasher},
    path::PathBuf,
    process,
    time::Duration,
};
use structopt::StructOpt;
use tokio::{
    fs::{self, read_to_string, File},
    io::AsyncWriteExt,
    time,
};
use uuid::Uuid;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long, default_value = ADDR)]
    connect: String,
    #[structopt(short, long)]
    app_name: PathBuf,
}

#[allow(dead_code)]
struct Worker {
    opt: Opt,
    id: String,
    app: LoadedApp,
    client: MapReduceClient<tonic::transport::Channel>,
}

macro_rules! write_kv {
    ($file:expr, $k:expr, $v:expr) => {
        $file.write_all(format!("{} {}\n", $k, $v).as_bytes())
    };
}

impl Worker {
    pub fn new(opt: Opt, client: MapReduceClient<tonic::transport::Channel>) -> Result<Self> {
        let app = load_app(&opt.app_name)?;
        let id = process::id().to_string();
        Ok(Self {
            opt,
            id,
            app,
            client,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            let PollTaskReply { task, shutdown } = self
                .client
                .poll_task(PollTaskRequest {})
                .await?
                .into_inner();

            if shutdown {
                info!("shutdown");
                return Ok(());
            }

            match task {
                Some(task) => {
                    info!("task: {:?}", task);
                    let complete: CompleteTaskRequest = match &task.inner {
                        Some(Inner::MapTask(map)) => {
                            let reduce_files = self.run_map(map.clone()).await?;
                            CompleteTaskRequest {
                                task: Some(task),
                                reduce_files,
                            }
                        }
                        Some(Inner::ReduceTask(reduce)) => {
                            self.run_reduce(reduce.clone()).await?;
                            CompleteTaskRequest {
                                task: Some(task),
                                reduce_files: Default::default(),
                            }
                        }
                        None => unreachable!(),
                    };

                    self.client.complete_task(complete).await?;
                    info!("task completed");
                }
                None => {
                    time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    pub async fn run_map(&self, task: MapTask) -> Result<HashMap<u64, String>> {
        let MapTask {
            index,
            files,
            n_reduce,
        } = task;

        let k1v1s = {
            let kv_futures = files.into_iter().map(|file| {
                read_to_string(file.clone()).map(|result| result.map(|content| (file, content)))
            });
            try_join_all(kv_futures).await?
        };

        let k2v2s = k1v1s.into_iter().flat_map(|(k, v)| self.app.map(k, v));

        let intermediate_filenames = (0..n_reduce)
            .map(|j| format!("out/mr-{}-{}-{}", index, j, Uuid::new_v4().to_string()))
            .collect_vec();
        let mut intermediate_files =
            try_join_all(intermediate_filenames.iter().map(File::create)).await?;

        for (k2, v2) in k2v2s {
            let file_index = {
                let mut hasher = DefaultHasher::new();
                k2.hash(&mut hasher);
                (hasher.finish() % n_reduce) as usize
            };
            let file = intermediate_files.get_mut(file_index).unwrap();
            write_kv!(file, k2, v2).await?;
        }

        // sync
        try_join_all(intermediate_files.iter().map(|f| f.sync_all())).await?;

        Ok(intermediate_filenames
            .into_iter()
            .enumerate()
            .map(|(i, f)| (i as u64, f))
            .collect())
    }

    pub async fn run_reduce(&self, task: ReduceTask) -> Result<()> {
        let ReduceTask { index, files } = task;

        let k2v2s = {
            let kv_futures = files.into_iter().map(|file| {
                read_to_string(file.clone()).map(|result| {
                    result.map(|content| {
                        content
                            .lines()
                            .map(|line| {
                                let mut tokens = line.split_whitespace();
                                (
                                    tokens.next().unwrap().to_owned(),
                                    tokens.next().unwrap().to_owned(),
                                )
                            })
                            .collect_vec()
                    })
                })
            });

            let mut k2v2s = try_join_all(kv_futures)
                .await?
                .into_iter()
                .flatten()
                .collect_vec();

            k2v2s.sort();
            k2v2s
        };

        let (temp_path, output_path) = (temp_file(), format!("out/mr-out-{}", index));
        let mut temp_file = File::create(&temp_path).await?;
        for (k, kvs) in k2v2s.into_iter().group_by(|kv| kv.0.clone()).into_iter() {
            let output = self.app.reduce(k.clone(), kvs.map(|kv| kv.1).collect_vec());
            write_kv!(temp_file, k, output).await?;
        }

        // sync
        temp_file.sync_all().await?;
        // rename
        fs::rename(temp_path, output_path).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let opt = Opt::from_args();
    let addr = format!("http://{}", opt.connect);
    let client = MapReduceClient::connect(addr).await?;

    let worker = Worker::new(opt, client)?;
    worker.run().await
}

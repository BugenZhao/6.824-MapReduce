use std::fs::{self, File};
use std::io::Write;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use common::declare_app;
use common::App;
use rand::Rng;

#[derive(Debug, Default)]
struct IndexerApp {
    count: AtomicUsize,
}

impl App for IndexerApp {
    fn map(&self, _: String, _: String) -> Vec<(String, String)> {
        /*
        me := os.Getpid()
        f := fmt.Sprintf("mr-worker-jobcount-%d-%d", me, count)
        count++
        err := ioutil.WriteFile(f, []byte("x"), 0666)
        if err != nil {
            panic(err)
        }
        time.Sleep(time.Duration(2000+rand.Intn(3000)) * time.Millisecond)
        return []mr.KeyValue{mr.KeyValue{"a", "x"}}
        */

        let file = format!(
            "mr-worker-jobcount-{}-{}",
            process::id(),
            self.count.fetch_add(1, Ordering::SeqCst)
        );
        File::create(file).unwrap().write_all(b"x").unwrap();
        thread::sleep(Duration::from_millis(
            2000 + rand::thread_rng().gen_range(0..3000),
        ));

        vec![("a".to_owned(), "x".to_owned())]
    }

    fn reduce(&self, _: String, _: Vec<String>) -> String {
        /*
        files, err := ioutil.ReadDir(".")
        if err != nil {
            panic(err)
        }
        invocations := 0
        for _, f := range files {
            if strings.HasPrefix(f.Name(), "mr-worker-jobcount") {
                invocations++
            }
        }
        return strconv.Itoa(invocations)
        */

        let invocations = fs::read_dir(".")
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .starts_with("mr-worker-jobcount")
            })
            .count();

        invocations.to_string()
    }
}

declare_app!(IndexerApp::default);

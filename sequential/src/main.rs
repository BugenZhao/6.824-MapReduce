use common::{load_app, Result};
use itertools::Itertools;
use std::{
    fs::{read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long)]
    app_name: PathBuf,
    #[structopt(short, long)]
    input_files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let app = load_app(opt.app_name)?;

    let mut intermediate = opt
        .input_files
        .iter()
        .flat_map(|file| {
            let content = read_to_string(file).unwrap();
            app.map(file.to_string_lossy().into_owned(), content)
        })
        .collect_vec();
    intermediate.sort();

    let mut output_file = File::create(&Path::new("mr-out-0"))?;
    for (k, kvs) in intermediate
        .into_iter()
        .group_by(|kv| kv.0.clone())
        .into_iter()
    {
        let output = app.reduce(k.clone(), kvs.map(|kv| kv.1).collect_vec());
        writeln!(output_file, "{} {}", k, output)?;
    }

    Ok(())
}

use std::env;
use uuid::Uuid;

pub mod service {
    tonic::include_proto!("service");
}

pub const ADDR: &str = "[::1]:56789";

pub fn init_logger() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init()
}

pub fn temp_file() -> String {
    let mut path = env::temp_dir();
    path.push(Uuid::new_v4().to_string());
    path.to_string_lossy().into_owned()
}

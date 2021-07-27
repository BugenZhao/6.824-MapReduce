pub mod service {
    tonic::include_proto!("service");
}

pub const ADDR: &str = "[::1]:56789";

pub fn init_logger() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init()
}

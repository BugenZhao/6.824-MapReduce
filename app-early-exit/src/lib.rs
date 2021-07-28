use std::sync::atomic::AtomicUsize;
use std::thread;
use std::time::Duration;

use common::declare_app;
use common::App;

#[derive(Debug, Default)]
struct IndexerApp {
    count: AtomicUsize,
}

impl App for IndexerApp {
    fn map(&self, filename: String, _: String) -> Vec<(String, String)> {
        vec![(filename, "1".to_owned())]
    }

    fn reduce(&self, key: String, values: Vec<String>) -> String {
        if key.contains("sherlock") || key.contains("tom") {
            thread::sleep(Duration::from_secs(3));
        }
        values.len().to_string()
    }
}

declare_app!(IndexerApp::default);

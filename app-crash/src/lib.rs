use std::env;
use std::process;
use std::thread;
use std::time::Duration;

use common::declare_app;
use common::App;
use rand::thread_rng;
use rand::Rng;

fn maybe_crash() {
    if env::var("CRASH").unwrap_or_default() != "1" {
        return;
    }
    let rr = thread_rng().gen_range(0..1000);
    if rr < 330 {
        println!("Crash");
        process::exit(1);
    } else if rr < 660 {
        println!("Sleep");
        let ms = thread_rng().gen_range(0..10000);
        thread::sleep(Duration::from_millis(ms)); // bad idea
    }
}

#[derive(Debug, Default)]
struct CrashApp;

impl App for CrashApp {
    fn map(&self, filename: String, contents: String) -> Vec<(String, String)> {
        maybe_crash();
        vec![
            ("a".to_owned(), filename.clone()),
            ("b".to_owned(), filename.len().to_string()),
            ("c".to_owned(), contents.len().to_string()),
            ("d".to_owned(), "xyzzy".to_owned()),
        ]
    }

    fn reduce(&self, _key: String, mut values: Vec<String>) -> String {
        maybe_crash();
        values.sort();
        values.join(" ")
    }
}

declare_app!(CrashApp::default);

use std::collections::HashMap;

use common::declare_app;
use common::App;

#[derive(Debug, Default)]
struct IndexerApp;

impl App for IndexerApp {
    fn map(&self, filename: String, contents: String) -> Vec<(String, String)> {
        let words = contents
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty());

        let mut occur = HashMap::new();
        for word in words {
            occur.entry(word.to_owned()).or_insert(filename.clone());
        }

        occur.into_iter().collect()
    }

    fn reduce(&self, _word: String, filenames: Vec<String>) -> String {
        format!("{} {}", filenames.len(), filenames.join(","))
    }
}

declare_app!(IndexerApp::default);

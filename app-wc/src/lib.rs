use common::declare_app;
use common::App;

#[derive(Debug, Default)]
struct WcApp;

impl App for WcApp {
    fn map(&self, _filename: String, contents: String) -> Vec<(String, String)> {
        contents
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .map(|w| (w.to_owned(), "1".to_owned()))
            .collect()
    }

    fn reduce(&self, _word: String, counts: Vec<String>) -> String {
        counts.len().to_string()
    }
}

declare_app!(WcApp::default);

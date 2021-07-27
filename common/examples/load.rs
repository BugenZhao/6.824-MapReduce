use common::load_app;

fn main() {
    let app = load_app("app_wc").unwrap();
    app.map("file".to_owned(), "Hello world".to_owned());
}

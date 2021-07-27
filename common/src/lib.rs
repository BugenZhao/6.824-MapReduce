pub use eyre::Result;
use libloading::{library_filename, Library};
use std::{ffi::OsStr, fmt::Debug, ops::Deref, sync::Arc};

pub trait App: Debug + Send + Sync {
    fn map(&self, k: String, v: String) -> Vec<(String, String)>;
    fn reduce(&self, k: String, vs: Vec<String>) -> String;
}

type BuildFn = fn() -> Box<dyn App>;

#[macro_export]
macro_rules! declare_app {
    ($constructor:expr) => {
        #[no_mangle]
        pub fn _build_app() -> Box<dyn ::common::App> {
            Box::new($constructor())
        }
    };
}

pub struct LoadedApp(Arc<dyn App>);

impl Deref for LoadedApp {
    type Target = Arc<dyn App>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn load_app(name: impl AsRef<OsStr>) -> Result<LoadedApp> {
    let app = unsafe {
        let lib = Library::new(library_filename(name))?;
        let build_fn = lib.get::<BuildFn>(b"_build_app\0")?;
        build_fn()
    };
    Ok(LoadedApp(Arc::from(app)))
}

use ruffle_core::backend::navigator::{
    NavigationMethod, NavigatorBackend, NullExecutor, NullSpawner, OwnedFuture, Request, Response,
};
use ruffle_core::backend::ui::LoaderError;
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::path::{Path, PathBuf};
use url::Url;

pub struct TestNavigatorBackend {
    spawner: NullSpawner,

    /// The base path for all relative fetches.
    relative_base_path: PathBuf,
}

impl TestNavigatorBackend {
    pub fn new() -> Self {
        let executor = NullExecutor::new();
        Self {
            spawner: executor.spawner(),
            relative_base_path: PathBuf::new(),
        }
    }

    pub fn with_base_path(path: &Path, executor: &NullExecutor) -> Self {
        Self {
            spawner: executor.spawner(),
            relative_base_path: path.canonicalize().unwrap(),
        }
    }

    #[cfg(any(unix, windows, target_os = "redox"))]
    fn url_from_file_path(path: &Path) -> Result<Url, ()> {
        Url::from_file_path(path)
    }

    #[cfg(not(any(unix, windows, target_os = "redox")))]
    fn url_from_file_path(_path: &Path) -> Result<Url, ()> {
        Err(())
    }
}

impl NavigatorBackend for TestNavigatorBackend {
    fn navigate_to_url(
        &self,
        _url: String,
        _window: Option<String>,
        _vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Response, Error> {
        if request.url().contains("?debug-success") {
            return Box::pin(async move {
                Ok(Response {
                    url: request.url().to_string(),
                    body: b"Hello, World!".to_vec(),
                })
            });
        }

        if request.url().contains("?debug-error") {
            return Box::pin(
                async move { Err(Error::FetchError("Error opening URL".to_string())) },
            );
        }

        let mut path = self.relative_base_path.clone();
        path.push(request.url());

        Box::pin(async move {
            let url = Self::url_from_file_path(&path)
                .map_err(|()| Error::FetchError("Invalid URL".to_string()))?
                .into();

            let body = std::fs::read(path).map_err(|e| Error::FetchError(e.to_string()))?;

            Ok(Response { url, body })
        })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.spawner.spawn_local(future);
    }

    fn pre_process_url(&self, url: Url) -> Url {
        url
    }
}

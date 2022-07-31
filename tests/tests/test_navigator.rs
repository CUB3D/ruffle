use ruffle_core::backend::navigator::{
    FetchError, NavigationMethod, NavigatorBackend, NullExecutor, NullSpawner, OwnedFuture,
    Request, Response,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::path::{Path, PathBuf};
use url::Url;

/// This is an implementation of [`NavigatorBackend`], designed for use in tests
///
/// This is essentially the same as [`NullNavigatorBackend`], however attempting to fetch URLs containing
/// the following "hints" will cause a simulated response:
/// * "?debug-success" -> Simulates a successful fetch, with body "Hello, World!"
/// * "?debug-error-statuscode" -> Simulates a failed fetch due to a unsuccessful status
/// * "?debug-error-dns" -> Simulates a failed fetch due to a dns resolution error
///
/// These are formatted as query params, rather than domains/whole URLs, so that real/real-invalid
/// URLs can be used in Flash Player when writing tests
pub struct TestNavigatorBackend {
    spawner: NullSpawner,

    /// The base path for all relative fetches.
    relative_base_path: PathBuf,
}

impl TestNavigatorBackend {
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
        _target: String,
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

        if request.url().contains("?debug-error-statuscode") {
            return Box::pin(async move {
                Err(Error::FetchError(FetchError::UnsuccessfulStatusCode {
                    body: vec![0u8; 10],
                }))
            });
        }
        if request.url().contains("?debug-error-dns") {
            return Box::pin(async move { Err(Error::FetchError(FetchError::InvalidDomain)) });
        }

        let mut path = self.relative_base_path.clone();
        path.push(request.url());

        Box::pin(async move {
            let url = Self::url_from_file_path(&path)
                .map_err(|()| Error::FetchError(FetchError::Other("Invalid URL".to_string())))?
                .into();

            let body = std::fs::read(path)
                .map_err(|e| Error::FetchError(FetchError::Other(e.to_string())))?;

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

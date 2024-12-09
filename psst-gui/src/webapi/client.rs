use std::{fmt::Display, io, path::PathBuf, sync::Arc, thread, time::Duration};

use druid::{im::Vector, image};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use psst_core::{
    session::{access_token::TokenProvider, SessionService},
    util::default_ureq_agent_builder,
};
use serde::de::DeserializeOwned;
use ureq::{Agent, Request, Response};

use crate::{
    data::{utils::Page, Playlist},
    error::Error,
};

use super::local::LocalTrackManager;

pub struct WebApi {
    session: SessionService,
    local_track_manager: Mutex<LocalTrackManager>,
    agent: Agent,
    token_provider: TokenProvider,
    paginated_limit: usize,
}

impl WebApi {
    pub fn new(
        session: SessionService,
        proxy_url: Option<&str>,
        cache_base: Option<PathBuf>,
        paginated_limit: usize,
    ) -> Self {
        let agent = default_ureq_agent_builder(proxy_url).unwrap().build();
        Self {
            session,
            agent,
            token_provider: TokenProvider::new(),
            local_track_manager: Mutex::new(LocalTrackManager::new()),
            paginated_limit,
        }
    }

    fn access_token(&self) -> Result<String, Error> {
        self.token_provider
            .get(&self.session)
            .map_err(|err| Error::WebApiError(err.to_string()))
            .map(|t| t.token)
    }

    fn build_request(
        &self,
        method: &str,
        base_url: &str,
        path: impl Display,
    ) -> Result<Request, Error> {
        let token = self.access_token()?;
        let request = self
            .agent
            .request(method, &format!("https://{base_url}/{path}"))
            .set("Authorization", &format!("Bearer {token}"));
        Ok(request)
    }

    fn request(&self, method: &str, base_url: &str, path: impl Display) -> Result<Request, Error> {
        self.build_request(method, base_url, path)
    }

    fn get(&self, path: impl Display, base_url: Option<&str>) -> Result<Request, Error> {
        self.request("GET", base_url.unwrap_or("api.spotify.com"), path)
    }

    fn post(&self, path: impl Display, base_url: Option<&str>) -> Result<Request, Error> {
        self.request("POST", base_url.unwrap_or("api.spotify.com"), path)
    }

    fn with_retry(f: impl Fn() -> Result<Response, Error>) -> Result<Response, Error> {
        loop {
            let response = f()?;
            match response.status() {
                429 => {
                    let retry_after_secs = response
                        .header("Retry-After")
                        .and_then(|secs| secs.parse().ok())
                        .unwrap_or(2);
                    thread::sleep(Duration::from_secs(retry_after_secs));
                }
                _ => {
                    break Ok(response);
                }
            }
        }
    }

    /// Send a request with a empty JSON object, throw away the response body.
    /// Use for POST/PUT/DELETE requests.
    fn send_empty_json(&self, request: Request) -> Result<(), Error> {
        Self::with_retry(|| Ok(request.clone().send_string("{}")?)).map(|_| ())
    }

    /// Send a request and return the deserialized JSON body.  Use for GET
    /// requests.
    fn load<T: DeserializeOwned>(&self, request: Request) -> Result<T, Error> {
        let response = Self::with_retry(|| Ok(request.clone().call()?))?;
        Ok(response.into_json()?)
    }

    /// Iterate a paginated result set by sending `request` with added
    /// pagination parameters.  Mostly used through `load_all_pages`.
    fn for_all_pages<T: DeserializeOwned + Clone>(
        &self,
        request: Request,
        mut func: impl FnMut(Page<T>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        // TODO: Some result sets, like very long playlists and saved tracks/albums can
        // be very big.  Implement virtualized scrolling and lazy-loading of results.
        let mut limit = 50;
        let mut offset = 0;
        loop {
            let req = request
                .clone()
                .query("limit", &limit.to_string())
                .query("offset", &offset.to_string());
            let page: Page<T> = self.load(req)?;

            let page_total = page.total;
            let page_offset = page.offset;
            let page_limit = page.limit;
            func(page)?;

            if page_total > offset && offset < self.paginated_limit {
                limit = page_limit;
                offset = page_offset + page_limit;
            } else {
                break Ok(());
            }
        }
    }

    /// Load a paginated result set by sending `request` with added pagination
    /// parameters and return the aggregated results.  Use with GET requests.
    fn load_all_pages<T: DeserializeOwned + Clone>(
        &self,
        request: Request,
    ) -> Result<Vector<T>, Error> {
        let mut results = Vector::new();

        self.for_all_pages(request, |page| {
            results.append(page.items);
            Ok(())
        })?;

        Ok(results)
    }

    /// Load local track files from the official client's database.
    pub fn load_local_tracks(&self, username: &str) {
        if let Err(err) = self
            .local_track_manager
            .lock()
            .load_tracks_for_user(username)
        {
            log::error!("failed to read local tracks: {}", err);
        }
    }
}

static GLOBAL_WEBAPI: OnceCell<Arc<WebApi>> = OnceCell::new();

/// Global instance.
impl WebApi {
    pub fn install_as_global(self) {
        GLOBAL_WEBAPI
            .set(Arc::new(self))
            .map_err(|_| "Cannot install more than once")
            .unwrap()
    }

    pub fn global() -> Arc<Self> {
        GLOBAL_WEBAPI.get().unwrap().clone()
    }
}

/// Playlist endpoints.
impl WebApi {
    // https://developer.spotify.com/documentation/web-api/reference/get-a-list-of-current-users-playlists
    pub fn get_playlists(&self) -> Result<Vector<Playlist>, Error> {
        let request = self.get("v1/me/playlists", None)?;
        let result = self.load_all_pages(request)?;
        Ok(result)
    }

    // https://developer.spotify.com/documentation/web-api/reference/add-tracks-to-playlist
    pub fn add_track_to_playlist(&self, playlist_id: &str, track_uri: &str) -> Result<(), Error> {
        let request = self
            .post(format!("v1/playlists/{}/tracks", playlist_id), None)?
            .query("uris", track_uri);
        self.send_empty_json(request)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::WebApiError(err.to_string())
    }
}

impl From<ureq::Error> for Error {
    fn from(err: ureq::Error) -> Self {
        Error::WebApiError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::WebApiError(err.to_string())
    }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Error::WebApiError(err.to_string())
    }
}

use std::{fmt::Display, io, path::PathBuf, sync::Arc, thread, time::Duration};

use druid::{im::Vector, image, Data};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use psst_core::{
    session::{access_token::TokenProvider, SessionService},
    util::default_ureq_agent_builder,
};
use serde::{de::DeserializeOwned, Deserialize};
use ureq::{json, Agent, Request, Response};

use crate::{
    data::{
        self, utils::Page, Album, AlbumType, Artist, ArtistLink, Cached, MixedView, Playlist,
        PublicUser, Show, UserProfile,
    },
    error::Error,
};

use super::{cache::WebApiCache, local::LocalTrackManager};

pub struct WebApi {
    session: SessionService,
    local_track_manager: Mutex<LocalTrackManager>,
    agent: Agent,
    cache: WebApiCache,
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
            cache: WebApiCache::new(cache_base),
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

    /// Send a request using `self.load()`, but only if it isn't already present
    /// in cache.
    fn load_cached<T: Data + DeserializeOwned>(
        &self,
        request: Request,
        bucket: &str,
        key: &str,
    ) -> Result<Cached<T>, Error> {
        if let Some(file) = self.cache.get(bucket, key) {
            let cached_at = file.metadata()?.modified()?;
            let value = serde_json::from_reader(file)?;
            Ok(Cached::new(value, cached_at))
        } else {
            let response = Self::with_retry(|| Ok(request.clone().call()?))?;
            let body = {
                let mut reader = response.into_reader();
                let mut body = Vec::new();
                reader.read_to_end(&mut body)?;
                body
            };

            let value = serde_json::from_slice(&body)?;
            self.cache.set(bucket, key, &body);
            Ok(Cached::fresh(value))
        }
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
            let req: Request = request
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

    fn load_and_return_home_section(&self, request: Request) -> Result<MixedView, Error> {
        #[derive(Deserialize)]
        pub struct Welcome {
            data: WelcomeData,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct WelcomeData {
            home_sections: HomeSections,
        }

        #[derive(Deserialize)]
        pub struct HomeSections {
            sections: Vec<Section>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Section {
            data: SectionData,
            section_items: SectionItems,
        }

        #[derive(Deserialize)]
        pub struct SectionData {
            title: Title,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Title {
            text: String,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct SectionItems {
            items: Vec<Item>,
        }

        #[derive(Deserialize)]
        pub struct Item {
            content: Content,
        }

        #[derive(Deserialize)]
        pub struct Content {
            data: ContentData,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct ContentData {
            #[serde(rename = "__typename")]
            typename: DataTypename,
            name: Option<String>,
            uri: String,

            // Playlist-specific fields
            attributes: Option<Vec<Attribute>>,
            description: Option<String>,
            images: Option<Images>,
            owner_v2: Option<OwnerV2>,

            // Artist-specific fields
            artists: Option<Artists>,
            profile: Option<Profile>,
            visuals: Option<Visuals>,

            // Show-specific fields
            cover_art: Option<CoverArt>,
            publisher: Option<Publisher>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Visuals {
            avatar_image: CoverArt,
        }

        #[derive(Deserialize)]
        pub struct Artists {
            items: Vec<ArtistsItem>,
        }

        #[derive(Deserialize)]
        pub struct ArtistsItem {
            profile: Profile,
            uri: String,
        }

        #[derive(Deserialize)]
        pub struct Profile {
            name: String,
        }

        #[derive(Deserialize)]
        pub struct Attribute {
            key: String,
            value: String,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct CoverArt {
            sources: Vec<Source>,
        }

        #[derive(Deserialize)]
        pub struct Source {
            url: String,
        }

        #[derive(Deserialize)]
        pub enum MediaType {
            #[serde(rename = "AUDIO")]
            Audio,
            #[serde(rename = "MIXED")]
            Mixed,
        }

        #[derive(Deserialize)]
        pub struct Publisher {
            name: String,
        }

        #[derive(Deserialize)]
        pub enum DataTypename {
            Podcast,
            Playlist,
            Artist,
            Album,
        }

        #[derive(Deserialize)]
        pub struct Images {
            items: Vec<ImagesItem>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct ImagesItem {
            sources: Vec<Source>,
        }

        #[derive(Deserialize)]
        pub struct OwnerV2 {
            data: OwnerV2Data,
        }

        #[derive(Deserialize)]
        pub struct OwnerV2Data {
            #[serde(rename = "__typename")]
            name: String,
        }

        // Extract the playlists
        let result: Welcome = self.load(request)?;

        let mut title: Arc<str> = Arc::from("");
        let mut playlist: Vector<Playlist> = Vector::new();
        let mut album: Vector<Arc<Album>> = Vector::new();
        let mut artist: Vector<Artist> = Vector::new();
        let mut show: Vector<Arc<Show>> = Vector::new();

        result
            .data
            .home_sections
            .sections
            .iter()
            .for_each(|section| {
                title = section.data.title.text.clone().into();

                section.section_items.items.iter().for_each(|item| {
                    let uri = item.content.data.uri.clone();
                    let id = uri.split(':').last().unwrap_or("").to_string();

                    match item.content.data.typename {
                        DataTypename::Playlist => {
                            playlist.push_back(Playlist {
                                id: id.into(),
                                name: Arc::from(item.content.data.name.clone().unwrap()),
                                owner: PublicUser {
                                    id: Arc::from(""),
                                    display_name: Arc::from(
                                        item.content
                                            .data
                                            .owner_v2
                                            .as_ref()
                                            .map(|owner| owner.data.name.as_str())
                                            .unwrap_or_default(),
                                    ),
                                },
                            });
                        }
                        DataTypename::Artist => artist.push_back(Artist {
                            id: id.into(),
                            name: Arc::from(
                                item.content.data.profile.as_ref().unwrap().name.clone(),
                            ),
                            images: item.content.data.visuals.as_ref().map_or_else(
                                Vector::new,
                                |images| {
                                    images
                                        .avatar_image
                                        .sources
                                        .iter()
                                        .map(|img| data::utils::Image {
                                            url: Arc::from(img.url.as_str()),
                                            width: None,
                                            height: None,
                                        })
                                        .collect()
                                },
                            ),
                        }),
                        DataTypename::Album => album.push_back(Arc::new(Album {
                            id: id.into(),
                            name: Arc::from(item.content.data.name.clone().unwrap()),
                            album_type: AlbumType::Album,
                            images: item.content.data.cover_art.as_ref().map_or_else(
                                Vector::new,
                                |images| {
                                    images
                                        .sources
                                        .iter()
                                        .map(|src| data::utils::Image {
                                            url: Arc::from(src.url.clone()),
                                            width: None,
                                            height: None,
                                        })
                                        .collect()
                                },
                            ),
                            artists: item.content.data.artists.as_ref().map_or_else(
                                Vector::new,
                                |artists| {
                                    artists
                                        .items
                                        .iter()
                                        .map(|artist| ArtistLink {
                                            id: Arc::from(
                                                artist
                                                    .uri
                                                    .split(':')
                                                    .last()
                                                    .unwrap_or("")
                                                    .to_string(),
                                            ),
                                            name: Arc::from(artist.profile.name.clone()),
                                        })
                                        .collect()
                                },
                            ),
                            copyrights: Vector::new(),
                            label: "".into(),
                            tracks: Vector::new(),
                            release_date: None,
                            release_date_precision: None,
                        })),
                        DataTypename::Podcast => show.push_back(Arc::new(Show {
                            id: id.into(),
                            name: Arc::from(item.content.data.name.clone().unwrap()),
                            images: item.content.data.cover_art.as_ref().map_or_else(
                                Vector::new,
                                |images| {
                                    images
                                        .sources
                                        .iter()
                                        .map(|src| data::utils::Image {
                                            url: Arc::from(src.url.clone()),
                                            width: None,
                                            height: None,
                                        })
                                        .collect()
                                },
                            ),
                            publisher: Arc::from(
                                item.content.data.publisher.as_ref().unwrap().name.clone(),
                            ),
                            description: "".into(),
                        })),
                    }
                });
            });

        Ok(MixedView {
            title,
            playlists: playlist,
            artists: artist,
            albums: album,
            shows: show,
        })
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

/// User endpoints.
impl WebApi {
    // https://developer.spotify.com/documentation/web-api/reference/get-users-profile
    pub fn get_user_profile(&self) -> Result<UserProfile, Error> {
        let request = self.get("v1/me", None)?;
        let result = self.load(request)?;
        Ok(result)
    }
}

// From https://github.com/KRTirtho/spotube/blob/9b024120601c0d381edeab4460cb22f87149d0f8/lib%2Fservices%2Fcustom_spotify_endpoints%2Fspotify_endpoints.dart keep and eye on this and change accordingly
const EXTENSIONS_JSON: &str = r#"{
    "persistedQuery": {
        "version": 1,
        "sha256Hash": "eb3fba2d388cf4fc4d696b1757a58584e9538a3b515ea742e9cc9465807340be"
    }
}"#;

/// View endpoints.
impl WebApi {
    pub fn get_user_info(&self) -> Result<(String, String), Error> {
        #[derive(Deserialize, Clone, Data)]
        struct User {
            region: String,
            timezone: String,
        }
        let token = self.access_token()?;
        let request = self
            .agent
            .request("GET", &format!("http://{}/{}", "ip-api.com", "json"))
            .query("fields", "260")
            .set("Authorization", &format!("Bearer {token}"));

        let result: Cached<User> = self.load_cached(request, "User_info", "usrinfo")?;

        Ok((result.data.region, result.data.timezone))
    }

    fn build_home_request(&self, section_uri: &str) -> Result<String, Error> {
        let (time_zone, country) = self.get_user_info()?;
        let access_token = self.access_token()?;

        let variables = json!( {
            "uri": section_uri,
            "timeZone": time_zone,
            "sp_t": access_token,
            "country": country,
            "sectionItemsOffset": 0,
            "sectionItemsLimit": 20,
        });

        serde_json::to_string(&variables)
            .map_err(|e| Error::WebApiError(format!("Couldn't serialize variables: {e}")))
    }
    pub fn get_section(&self, section_uri: &str) -> Result<MixedView, Error> {
        let request = self
            .get("pathfinder/v1/query", Some("api-partner.spotify.com"))?
            .query("operationName", "homeSection")
            .query("variables", &self.build_home_request(section_uri)?)
            .query("extensions", EXTENSIONS_JSON);

        // Extract the playlists
        self.load_and_return_home_section(request)
    }

    pub fn get_made_for_you(&self) -> Result<MixedView, Error> {
        // 0JQ5DAUnp4wcj0bCb3wh3S -> Daily mixes
        self.get_section("spotify:section:0JQ5DAUnp4wcj0bCb3wh3S")
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

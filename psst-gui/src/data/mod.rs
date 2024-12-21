pub mod config;
mod ctx;
mod nav;
mod playback;
mod playlist;
mod promise;
mod user;
pub mod utils;

use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

pub use crate::data::{
    ctx::Ctx,
    nav::Nav,
    playlist::{Playlist, PlaylistLink},
    promise::{Promise, PromiseState},
};
use config::{Authentication, Preferences, PreferencesTab};
use druid::{im::Vector, Data, Lens};
use playback::Playback;
use psst_core::session::SessionService;
use user::UserProfile;

pub use crate::data::config::Config;

#[derive(Clone, Data, Lens)]
pub struct AppState {
    #[data(ignore)]
    pub session: SessionService,
    pub config: Config,
    pub preferences: Preferences,
    pub playback: Playback,
    pub library: Arc<Library>,
    pub common_ctx: Arc<CommonCtx>,
    pub alerts: Vector<Alert>,
}

impl AppState {
    pub fn default_with_config(config: Config) -> Self {
        let library = Arc::new(Library {
            user_profile: Promise::Empty,
            playlists: Promise::Empty,
        });
        let common_ctx = Arc::new(CommonCtx {
            library: Arc::clone(&library),
            show_track_cover: config.show_track_cover,
        });
        let playback = Playback { now_playing: None };
        Self {
            session: SessionService::empty(),
            config,
            preferences: Preferences {
                active: PreferencesTab::General,
                auth: Authentication {
                    username: String::new(),
                    password: String::new(),
                    access_token: String::new(),
                    result: Promise::Empty,
                },
                cache_size: Promise::Empty,
            },
            playback,
            library,
            common_ctx,
            alerts: Vector::new(),
        }
    }
}

impl AppState {
    pub fn common_ctx_mut(&mut self) -> &mut CommonCtx {
        Arc::make_mut(&mut self.common_ctx)
    }
    pub fn with_library_mut(&mut self, func: impl FnOnce(&mut Library)) {
        func(Arc::make_mut(&mut self.library));
        self.library_update();
    }

    fn library_update(&mut self) {
        if let Some(now_playing) = &mut self.playback.now_playing {
            now_playing.library = Arc::clone(&self.library);
        }
        self.common_ctx_mut().library = Arc::clone(&self.library);
    }
}

impl AppState {
    pub fn add_alert(&mut self, message: impl Display, style: AlertStyle) {
        let alert = Alert {
            message: message.to_string().into(),
            style,
            id: Alert::fresh_id(),
            create_at: Instant::now(),
        };
        self.alerts.push_back(alert);
    }
    pub fn error_alert(&mut self, message: impl Display) {
        self.add_alert(message, AlertStyle::Error);
    }
}

#[derive(Clone, Data, Lens)]
pub struct Library {
    pub user_profile: Promise<UserProfile>,
    pub playlists: Promise<Vector<Playlist>>,
}

impl Library {
    pub fn is_created_by_user(&self, playlist: &Playlist) -> bool {
        if let Some(profile) = self.user_profile.resolved() {
            profile.id == playlist.owner.id
        } else {
            false
        }
    }

    pub fn contains_playlist(&self, playlist: &Playlist) -> bool {
        if let Some(playlists) = self.playlists.resolved() {
            playlists.iter().any(|p| p.id == playlist.id)
        } else {
            false
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct CommonCtx {
    pub library: Arc<Library>,
    pub show_track_cover: bool,
}

pub type WithCtx<T> = Ctx<Arc<CommonCtx>, T>;

static ALERT_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Data, Lens)]
pub struct Alert {
    pub id: usize,
    pub message: Arc<str>,
    pub style: AlertStyle,
    pub create_at: Instant,
}

impl Alert {
    fn fresh_id() -> usize {
        ALERT_ID.fetch_add(1, Ordering::SeqCst)
    }
}

#[derive(Clone, Data, Eq, PartialEq)]
pub enum AlertStyle {
    Error,
    Info,
}

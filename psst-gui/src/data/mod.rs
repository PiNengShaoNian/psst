pub mod config;
mod promise;

use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use config::{Authentication, Preferences, PreferencesTab};
use druid::{im::Vector, Data, Lens};
use promise::Promise;
use psst_core::session::SessionService;

pub use crate::data::config::Config;

#[derive(Clone, Data, Lens)]
pub struct AppState {
    #[data(ignore)]
    pub session: SessionService,
    pub config: Config,
    pub preferences: Preferences,
    pub alerts: Vector<Alert>,
}

impl AppState {
    pub fn default_with_config(config: Config) -> Self {
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
            },
            alerts: Vector::new(),
        }
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

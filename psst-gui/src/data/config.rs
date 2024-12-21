use std::{
    env::{self, VarError},
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use druid::{Data, Lens, Size};
use psst_core::{
    cache::mkdir_if_not_exists,
    connection::Credentials,
    session::{SessionConfig, SessionConnection},
};
use serde::{Deserialize, Serialize};

use crate::ui::theme;

use super::{promise::Promise, Nav, SliderScrollScale};

#[derive(Clone, Debug, Data, Lens)]
pub struct Preferences {
    pub active: PreferencesTab,
    pub cache_size: Promise<u64, (), ()>,
    pub auth: Authentication,
}

impl Preferences {
    pub fn measure_cache_usage() -> Option<u64> {
        Config::cache_dir().and_then(|path| get_dir_size(&path))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Data)]
pub enum PreferencesTab {
    General,
    Account,
    Cache,
    About,
}

#[derive(Clone, Debug, Data, Lens)]
pub struct Authentication {
    pub username: String,
    pub password: String,
    pub access_token: String,
    pub result: Promise<(), (), String>,
}

impl Authentication {
    pub fn session_config(&self) -> SessionConfig {
        SessionConfig {
            login_creds: if !self.access_token.is_empty() {
                Credentials::from_access_token(self.access_token.clone())
            } else {
                Credentials::from_username_and_password(
                    self.username.clone(),
                    self.password.clone(),
                )
            },
            proxy_url: Config::proxy(),
        }
    }

    pub fn authenticate_and_get_credentials(config: SessionConfig) -> Result<Credentials, String> {
        let connection = SessionConnection::open(config).map_err(|err| err.to_string())?;
        Ok(connection.credentials)
    }
}

const PROXY_ENV_VAR: &str = "SOCKS_PROXY";

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[data(ignore)]
    credentials: Option<Credentials>,
    pub audio_quality: AudioQuality,
    pub theme: Theme,
    pub volume: f64,
    pub last_route: Option<Nav>,
    pub show_track_cover: bool,
    pub window_size: Size,
    pub slider_scroll_scale: SliderScrollScale,
    pub paginated_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            credentials: Default::default(),
            audio_quality: Default::default(),
            theme: Default::default(),
            volume: 1.0,
            last_route: Default::default(),
            show_track_cover: Default::default(),
            window_size: Size::new(theme::grid(80.0), theme::grid(100.0)),
            slider_scroll_scale: Default::default(),
            paginated_limit: 500,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data, Serialize, Deserialize)]
pub enum AudioQuality {
    Low,
    Normal,
    High,
}

impl Default for AudioQuality {
    fn default() -> Self {
        Self::High
    }
}

const APP_NAME: &str = "Psst";
const CONFIG_FILENAME: &str = "config.json";

impl Config {
    fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("", "", APP_NAME)
    }

    pub fn spotify_local_files_file(username: &str) -> Option<PathBuf> {
        ProjectDirs::from("", "", "spotify").map(|dirs| {
            let path = format!("Users/{}-user/local-files.bnk", username);
            dirs.config_dir().join(path)
        })
    }

    pub fn cache_dir() -> Option<PathBuf> {
        Self::project_dirs().map(|dirs| dirs.cache_dir().to_path_buf())
    }

    pub fn config_dir() -> Option<PathBuf> {
        Self::project_dirs().map(|dirs| dirs.cache_dir().to_path_buf())
    }
    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join(CONFIG_FILENAME))
    }
    pub fn load() -> Option<Config> {
        let path = Self::config_path().expect("Failed to get config path");
        if let Ok(file) = File::open(&path) {
            log::info!("loading config: {:?}", &path);
            let reader = BufReader::new(file);
            Some(serde_json::from_reader(reader).expect("Failed to read config"))
        } else {
            None
        }
    }

    pub fn save(&self) {
        let dir = Self::config_dir().expect("Failed to get config dir");
        let path = Self::config_path().expect("Failed to get config path");
        mkdir_if_not_exists(&dir).expect("Failed to create config dir");

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(target_family = "unix")]
        options.mode(0o600);

        let file = options.open(&path).expect("Failed to create config");
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self).expect("Failed to write config");
        log::info!("saved config: {:?}", &path);
    }

    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }

    pub fn store_credentials(&mut self, credential: Credentials) {
        self.credentials.replace(credential);
    }

    pub fn clear_credentials(&mut self) {
        self.credentials = Default::default()
    }

    pub fn username(&self) -> Option<&str> {
        self.credentials
            .as_ref()
            .and_then(|c| c.username.as_deref())
    }

    pub fn session(&self) -> SessionConfig {
        SessionConfig {
            login_creds: self.credentials.clone().expect("Missing credentials"),
            proxy_url: Config::proxy(),
        }
    }

    pub fn proxy() -> Option<String> {
        env::var(PROXY_ENV_VAR).map_or_else(
            |err| match err {
                VarError::NotPresent => None,
                VarError::NotUnicode(_) => {
                    log::error!("proxy URL is not a valid unicode");
                    None
                }
            },
            Some,
        )
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Light
    }
}

fn get_dir_size(path: &Path) -> Option<u64> {
    fs::read_dir(path).ok()?.try_fold(0, |acc, entry| {
        let entry = entry.ok()?;
        let size = if entry.file_type().ok()?.is_dir() {
            get_dir_size(&entry.path())?
        } else {
            entry.metadata().ok()?.len()
        };
        Some(acc + size)
    })
}

use std::{fs::File, io::BufReader, path::PathBuf};

use config_derived_lenses::credentials;
use directories::ProjectDirs;
use druid::{Data, Lens};
use psst_core::connection::Credentials;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[data(ignore)]
    credentials: Option<Credentials>,
    pub audio_quality: AudioQuality,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            credentials: Default::default(),
            audio_quality: Default::default(),
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

impl Config {
    fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("", "", APP_NAME)
    }

    pub fn config_dir() -> Option<PathBuf> {
        Self::project_dirs().map(|dirs| dirs.cache_dir().to_path_buf())
    }
    fn config_path() -> Option<PathBuf> {
        Self::config_dir()
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
}

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

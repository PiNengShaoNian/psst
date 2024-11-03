pub mod config;

use druid::{Data, Lens};

pub use crate::data::config::Config;

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub config: Config,
}

impl AppState {
    pub fn default_with_config(config: Config) -> Self {
        Self { config }
    }
}

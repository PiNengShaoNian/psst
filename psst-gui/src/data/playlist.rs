use std::sync::Arc;

use druid::{Data, Lens};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Data, Lens, Deserialize)]
pub struct Playlist {
    pub id: Arc<str>,
    pub name: Arc<str>,
}

impl Playlist {
    pub fn link(&self) -> PlaylistLink {
        PlaylistLink {
            id: self.id.clone(),
            name: self.name.clone(),
        }
    }
}

#[derive(Clone, Debug, Data, Lens, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct PlaylistLink {
    pub id: Arc<str>,
    pub name: Arc<str>,
}

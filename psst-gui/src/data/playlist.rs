use std::sync::Arc;

use druid::{Data, Lens};
use serde::{Deserialize, Serialize};

use super::user::PublicUser;

#[derive(Clone, Debug, Data, Lens, Deserialize)]
pub struct Playlist {
    pub id: Arc<str>,
    pub name: Arc<str>,
    pub owner: PublicUser,
}

impl Playlist {
    pub fn link(&self) -> PlaylistLink {
        PlaylistLink {
            id: self.id.clone(),
            name: self.name.clone(),
        }
    }

    pub fn url(&self) -> String {
        format!("https://open.spotify.com/playlist/{id}", id = self.id)
    }
}

#[derive(Clone, Debug, Data, Lens, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct PlaylistLink {
    pub id: Arc<str>,
    pub name: Arc<str>,
}

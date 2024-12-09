use druid::Data;
use serde::{Deserialize, Serialize};

use super::playlist::PlaylistLink;

#[derive(Default, Clone, Debug, Data, PartialEq, Eq, Deserialize, Serialize)]
pub enum Nav {
    #[default]
    Home,
    PlaylistDetail(PlaylistLink),
}

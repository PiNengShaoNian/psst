use std::sync::Arc;

use druid::{Data, Lens};

use super::Library;

#[derive(Clone, Data, Lens)]
pub struct Playback {
    pub now_playing: Option<NowPlaying>,
}

#[derive(Clone, Data, Lens)]
pub struct NowPlaying {
    // Although keeping a ref to the `Library` here is a bit of a hack, it dramatically
    // simplifies displaying the track context menu in the playback bar.
    pub library: Arc<Library>,
}

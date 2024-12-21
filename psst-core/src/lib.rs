use git_version::git_version;

pub const GIT_VERSION: &str = git_version!();
pub const BUILD_TIME: &str = include!(concat!(env!("OUT_DIR"), "/build-time.txt"));
pub const REMOTE_URL: &str = include!(concat!(env!("OUT_DIR"), "/remote-url.txt"));

pub mod audio;
pub mod cache;
pub mod cdn;
pub mod connection;
pub mod error;
pub mod item_id;
pub mod oauth;
pub mod session;
pub mod util;

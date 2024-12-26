use std::{
    fs::{self, File},
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
};

use druid::ImageBuf;
use lru::LruCache;
use parking_lot::Mutex;
use psst_core::cache::mkdir_if_not_exists;

pub struct WebApiCache {
    base: Option<PathBuf>,
    images: Mutex<LruCache<Arc<str>, ImageBuf>>,
}

impl WebApiCache {
    pub fn new(base: Option<PathBuf>) -> Self {
        const IMAGE_CACHE_SIZE: usize = 256;
        Self {
            base,
            images: Mutex::new(LruCache::new(NonZeroUsize::new(IMAGE_CACHE_SIZE).unwrap())),
        }
    }

    pub fn get(&self, bucket: &str, key: &str) -> Option<File> {
        self.key(bucket, key).and_then(|path| File::open(path).ok())
    }

    pub fn set(&self, bucket: &str, key: &str, value: &[u8]) {
        if let Some(path) = self.bucket(bucket) {
            if let Err(err) = mkdir_if_not_exists(&path) {
                log::error!("failed to create WebAPI cache bucket: {:?}", err);
            }
        }
        if let Some(path) = self.key(bucket, key) {
            if let Err(err) = fs::write(path, value) {
                log::error!("failed to save to WebAPI cache: {:?}", err);
            }
        }
    }

    fn bucket(&self, bucket: &str) -> Option<PathBuf> {
        self.base.as_ref().map(|path| path.join(bucket))
    }

    fn key(&self, bucket: &str, key: &str) -> Option<PathBuf> {
        self.bucket(bucket).map(|path| path.join(key))
    }
}

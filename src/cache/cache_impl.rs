// Standard library imports
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::SystemTime;
use hyper::body::Bytes;
// External crate imports
use hyper::Uri;
use tokio::sync::RwLock;

pub struct Cache {
    content: RwLock<HashMap<Uri, (Bytes, SystemTime, String)>>,
}

impl Cache {
    pub(crate) fn new() -> Arc<Self> {
          Arc::new(Self { 
              content: RwLock::new(HashMap::new()),
        })
    }

    pub(crate) async fn read_cache(cache: Arc<Self>, uri: &Uri) -> Option<(Bytes, SystemTime, String)> {
        let content_guard = cache.content.read().await;
        return match content_guard.get(uri) {
            Some((http_content, last_modified, etag)) => {
                // start
                Some((http_content.clone(), last_modified.clone(), etag.clone()))
            }
            None => None
        }
    }

    // writes to cache, returning the generated etag for the cache instance
    pub(crate) async fn write_cache(cache: Arc<Self>, uri: &Uri, http_content: &Bytes, last_modified: &SystemTime, etag: &String) {
        // Insert into content
        let mut content_guard = cache.content.write().await;
        content_guard.insert(uri.clone(), (http_content.clone(), last_modified.clone(), etag.clone()));
    }

    pub(crate) fn generate_etag(http_content: &Bytes) -> String {
        let mut hasher = DefaultHasher::new();
        (*http_content).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
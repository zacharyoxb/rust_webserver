// Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

use std::time::SystemTime;
use hyper::body::Bytes;
use hyper::http::HeaderValue;
// External crate imports
use hyper::Uri;
use tokio::sync::RwLock;
use crate::request_handler::handler_utils;

pub struct Cache {
    content: RwLock<HashMap<Uri, (Bytes, SystemTime)>>,
    etag_map: RwLock<HashMap<String, Uri>>
}

impl Cache {
    pub(crate) fn new() -> Arc<Self> {
          Arc::new(Self { 
              content: RwLock::new(HashMap::new()),
              etag_map: RwLock::new(HashMap::new())
        })
    }

    pub(crate) async fn read_cache(cache: Arc<Self>, uri: &Uri) -> Option<(Bytes, SystemTime)> {
        let content_guard = cache.content.read().await;
        return match content_guard.get(uri) {
            Some((http_content, last_modified)) => {
                // start
                Some((http_content.clone(), last_modified.clone()))
            }
            None => None
        }
    }

    pub(crate) async fn write_cache(cache: Arc<Self>, uri: &Uri, http_content: &Bytes, last_modified: &SystemTime) {
        // insert into content
        let mut content_guard = cache.content.write().await;
        content_guard.insert(uri.clone(), (http_content.clone(), last_modified.clone()));
        // insert etag into etag_map
        let mut guard_etag = cache.etag_map.write().await;
        guard_etag.insert(handler_utils::generate_etag(http_content), uri.clone());
    }

    pub async fn add_etag(&self, etag: String, uri: Uri) {
        let mut etag_guard = self.etag_map.write().await;
        etag_guard.insert(etag, uri);
    }

    pub async fn remove_etag(&self, etag: &str) {
        let mut etag_guard = self.etag_map.write().await;
        etag_guard.remove(etag);
    }
    
    pub(crate) async fn etag_match(cache: Arc<Self>, etags: &HeaderValue) -> Result<bool, Box<dyn std::error::Error>> {
        let etags_str = etags.to_str()?;
        let etags_vec: Vec<&str> = etags_str.split(',').map(|s| s.trim()).collect();

        let guard = cache.etag_map.read().await;
        for etag in etags_vec {
            if guard.contains_key(etag) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
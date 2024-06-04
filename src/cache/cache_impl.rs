// Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

use std::time::SystemTime;
// External crate imports
use hyper::Uri;
use tokio::sync::RwLock;
use crate::request_handler::handler_utils;

pub struct Cache {
    content: RwLock<HashMap<Uri, (String, SystemTime)>>,
    etag_map: RwLock<HashMap<String, Uri>>
}

impl Cache {
    pub(crate) fn new() -> Arc<Self> {
          Arc::new(Self { 
              content: RwLock::new(HashMap::new()),
              etag_map: RwLock::new(HashMap::new())
        })
    }

    pub(crate) async fn read_cache(cache: Arc<Self>, uri: &Uri) -> Option<(String, SystemTime)> {
        let guard = cache.content.read().await;
        return match guard.get(uri) {
            Some((http_content, last_modified)) => {
                // start
                Some((http_content.clone(), last_modified.clone()))
            }
            None => None
        }
    }
    
    pub(crate) async fn read_cache_with_etag(cache: Arc<Self>, etag: String) -> Option<(String, SystemTime)> {
        let guard = cache.etag_map.read().await;
        match guard.get(&etag) {
            Some(uri) => Self::read_cache(Arc::clone(&cache), uri).await,
            None => None
        }
    }

    pub(crate) async fn write_cache(cache: Arc<Self>, uri: &Uri, http_content: &String, last_modified: &SystemTime) {
        // insert into content
        let mut guard_content = cache.content.write().await;
        guard_content.insert(uri.clone(), (http_content.clone(), last_modified.clone()));
        // insert etag into etag_map
        let mut guard_etag = cache.etag_map.write().await;
        guard_etag.insert(handler_utils::generate_etag(http_content), uri.clone());
    }
}
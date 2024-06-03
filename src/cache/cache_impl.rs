use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use hyper::Uri;
use tokio::sync::RwLock;

pub struct Cache {
    content: RwLock<HashMap<Uri, (String, SystemTime)>>
}

impl Cache {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            content: RwLock::new(HashMap::new()),
        })
    }

    pub(crate) async fn read_cache(cache: Arc<Self>, uri: &Uri) -> Option<(String, SystemTime)> {
        let guard = cache.content.read().await;
        return match guard.get(uri) {
            Some((http_content, last_modified)) => Some((http_content.clone(), last_modified.clone())),
            None => None
        }
    }

    pub(crate) async fn write_cache(cache: Arc<Self>, uri: &Uri, http_content: &String, last_modified: &SystemTime) {
        let mut guard = cache.content.write().await;
        guard.insert(uri.clone(), (http_content.clone(), last_modified.clone()));
    }
}
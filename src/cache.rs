use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::SystemTime;

use hyper::body::Bytes;
use hyper::Uri;
use tokio::sync::RwLock;

/// cache for holding the resource contents, when the resource was last modified, and its etag.
pub struct Cache {
    content: RwLock<HashMap<Uri, (Bytes, String, SystemTime, String)>>,
}

impl Cache {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            content: RwLock::new(HashMap::new()),
        })
    }

    /// reads cache using the uri, either returning its contents and metadata or None if it's not in the cache
    pub(crate) async fn read_cache(
        cache: Arc<Self>,
        uri: &Uri,
    ) -> Option<(Bytes, String, SystemTime, String)> {
        let content_guard = cache.content.read().await;
        content_guard
            .get(uri)
            .map(|(resource_content, content_type, last_modified, etag)| {
                (
                    resource_content.clone(),
                    content_type.clone(),
                    *last_modified,
                    etag.clone(),
                )
            })
    }

    /// writes to cache
    pub(crate) async fn write_cache(
        cache: Arc<Self>,
        uri: &Uri,
        resource_content: &Bytes,
        content_type: &str,
        last_modified: &SystemTime,
        etag: &str,
    ) {
        // Insert into content
        let mut content_guard = cache.content.write().await;
        content_guard.insert(
            uri.clone(),
            (
                resource_content.clone(),
                content_type.to_string(),
                *last_modified,
                etag.to_owned(),
            ),
        );
    }

    /// generates etag for content
    pub(crate) fn generate_etag(resource_content: &Bytes) -> String {
        let mut hasher = DefaultHasher::new();
        (*resource_content).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

// Standard library imports
use std::time::SystemTime;
// External crate imports
use hyper::Uri;
// Internal crates
use crate::Cache;

// If in cache returns http contents and last modified, otherwise none
pub(crate) async fn read_cache(cache: Cache, uri: &Uri) -> Option<(String, SystemTime)> {
    let guard = cache.read().await;
    return match guard.get(uri) {
        Some((http_content, last_modified)) => {
            //TODO do a modified check here
            Some((http_content.clone(), last_modified.clone()))
        } 
        None => return None
    }
}

pub(crate) async fn write_to_cache(cache: Cache, uri: &Uri, http_content: &String, last_modified: &SystemTime) {
    let mut guard = cache.write().await;
    guard.insert(uri.clone(), (http_content.clone(), last_modified.clone()));
}
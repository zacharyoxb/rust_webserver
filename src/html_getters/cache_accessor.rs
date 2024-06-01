use hyper::Uri;
use crate::Cache;

pub(crate) async fn read_cache(cache: Cache, uri: &Uri) -> String {
    let guard = cache.read().await;
    return match guard.get(uri) {
        Some(http_content) => http_content.clone(),
        _ => "Null".to_string()
    }
}

pub(crate) async fn write_to_cache(cache: Cache, uri: &Uri, http_content: &String) {
    let mut guard = cache.write().await;
    guard.insert(uri.clone(), http_content.clone());
}
// Standard library imports
use std::convert::Infallible;
use std::sync::Arc;
use std::time::SystemTime;
use http_body_util::Full;
// External crate imports
use hyper::{Request, Response, StatusCode};
use hyper::body::Bytes;
use hyper::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, DATE, ETAG, EXPIRES, LAST_MODIFIED, SERVER};
// Internal crates
use crate::Cache;
use crate::html_getters::{cache_accessor, dir_accessor};
use crate::request_handler::handler_utils;

// Handles get requests, returning either a get response packet or server error packet
pub(crate) async fn handle_get(req: Request<hyper::body::Incoming>, cache: Cache) -> Result<Response<Full<Bytes>>, Infallible> {
    // clone arc
    let cache_clone = Arc::clone(&cache);
    // check cache for the page
    let cache_results = cache_accessor::read_cache(cache, req.uri()).await;

    // create http_content and last_modified variables
    let http_content: String;
    let last_modified: SystemTime;

    match cache_results {
        Some((cache_content, cache_last_modified)) => {
            http_content = cache_content;
            last_modified = cache_last_modified
        }
        None => {
            // If not in cache read from file
            match dir_accessor::retrieve_from_path(req.uri()).await {
                Ok((read_content, Some(read_last_modified))) => {
                    // assign to variables then cache response
                    http_content = read_content;
                    last_modified = read_last_modified;
                    cache_accessor::write_to_cache(cache_clone, req.uri(), &http_content, &last_modified).await;
                }
                Ok((read_content, None)) => {
                    return handler_utils::send_not_found_packet(read_content)
                }
                Err(_) => {
                    return handler_utils::send_error_packet()
                }
            }
        }
    }
    
    handler_utils::send_default_ok_packet(&http_content, &last_modified)
}
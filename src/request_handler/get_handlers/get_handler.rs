// Standard library imports
use std::convert::Infallible;
use std::sync::Arc;
use std::time::SystemTime;
use http_body_util::Full;
// External crate imports
use hyper::{Request, Response, StatusCode};
use hyper::body::Bytes;
// Internal crates
use crate::Cache;
use crate::html_getters::{cache_accessor, dir_accessor};
use crate::request_handler::server_error_handler;

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
        Some((cache_content, _cache_last_modified)) => {
            http_content = cache_content;
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
                    // send not found packet
                    let response = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Full::new(Bytes::from(read_content)))
                        .unwrap();
                    return Ok(response)
                }
                Err(_) => {
                    return server_error_handler::send_error_packet()
                }
            }
        }
    }
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(Full::new(Bytes::from(http_content)))
        .unwrap();
    Ok(response)
}
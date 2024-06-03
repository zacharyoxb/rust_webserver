// Standard library imports
use std::convert::Infallible;
use std::time::SystemTime;
use http_body_util::Full;
// External crate imports
use hyper::{Request, Response, StatusCode};
use hyper::body::Bytes;
// Internal crates
use crate::Cache;
use crate::html_getters::{cache_accessor, dir_accessor};
use crate::request_handler::handler_utils;

// Handles get requests, returning either a get response packet or server error packet
pub(crate) async fn handle_last_modified(req: Request<hyper::body::Incoming>, cache: Cache) -> Result<Response<Full<Bytes>>, Infallible> {
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
                    http_content = read_content;
                    last_modified = read_last_modified;
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

    // return correct response, checking which modified header the request has
    return if req.headers().get("If-Modified-Since").is_some() {
        match handler_utils::modified_since(req.headers().get("If-Modified-Since").unwrap(), last_modified) {
            Ok(false) => {
                handler_utils::send_not_modified_packet()
            }
            Ok(true) => {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Full::new(Bytes::from(http_content)))
                    .unwrap();
                Ok(response)
            }
            Err(_) => handler_utils::send_error_packet()
        }
    } else {
        match handler_utils::modified_since(req.headers().get("If-Unmodified-Since").unwrap(), last_modified) {
            Ok(false) => {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Full::new(Bytes::from(http_content)))
                    .unwrap();
                Ok(response)
            }
            Ok(true) => {
                handler_utils::send_precondition_failed_packet()
            }
            Err(_) => handler_utils::send_error_packet()
        }
    }
}

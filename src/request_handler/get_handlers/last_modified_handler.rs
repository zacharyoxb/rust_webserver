// Standard library imports
use std::convert::Infallible;
use std::io;
use std::io::ErrorKind;
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use http_body_util::Full;
// External crate imports
use hyper::{Request, Response, StatusCode};
use hyper::body::Bytes;
use hyper::http::HeaderValue;
// Internal crates
use crate::Cache;
use crate::html_getters::{cache_accessor, dir_accessor};
use crate::request_handler::server_error_handler;

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

    // return correct response, checking which modified header the request has
    return if req.headers().get("If-Modified-Since").is_some() {
        match modified_since(req.headers().get("If-Modified-Since").unwrap(), last_modified) {
            Ok(false) => {
                // send back not modified packet
                let response = Response::builder()
                    .status(StatusCode::NOT_MODIFIED)
                    .body(Full::new(Bytes::new()))
                    .unwrap();
                Ok(response)
            }
            Ok(true) => {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Full::new(Bytes::from(http_content)))
                    .unwrap();
                Ok(response)
            }
            Err(_) => server_error_handler::send_error_packet()
        }
    } else {
        match modified_since(req.headers().get("If-Unmodified-Since").unwrap(), last_modified) {
            Ok(false) => {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Full::new(Bytes::from(http_content)))
                    .unwrap();
                Ok(response)
            }
            Ok(true) => {
                let response = Response::builder()
                    .status(StatusCode::PRECONDITION_FAILED)
                    .body(Full::new(Bytes::new()))
                    .unwrap();
                Ok(response)
            }
            Err(_) => server_error_handler::send_error_packet()
        }
    }
}

// checks if request header time is older than last page update
// TODO: the "modified since" function will be relevant to other handlers in the future. Later, move to "handler_utils" module.
fn modified_since(req_header_val: &HeaderValue, page_last_modified: SystemTime) -> Result<bool, io::Error> {
    // header to string
    let req_modified_str = req_header_val.to_str()
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid header value string"))?;
    // string to time
    let request_last_modified = DateTime::parse_from_rfc2822(req_modified_str)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid date format"))?;
    let page_modified_datetime: DateTime<Utc> = DateTime::from(page_last_modified);
    Ok(request_last_modified < page_modified_datetime)
}
// Standard library imports
use std::convert::Infallible;
use std::sync::Arc;
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
    // clone arc instance
    let cache_clone = Arc::clone(&cache);

    // check cache for the page
    let cache_results = cache_accessor::read_cache(cache, req.uri()).await;
    
    // if page is cached
    if cache_results != None {
        let (http_content, _last_modified) = cache_results.unwrap();
        let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(Full::new(Bytes::from(http_content)))
        .unwrap();
        return Ok(response)
    }

    // if not in cache, check if file exists
    let http_content_result = dir_accessor::retrieve_from_path(req.uri()).await;

    match http_content_result {
        // No error, page found
        Ok((http_content, Some(last_modified))) => {
            // cache content then send response
            cache_accessor::write_to_cache(cache_clone, req.uri(), &http_content, &last_modified).await;

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(Full::new(Bytes::from(http_content)))
                .unwrap();
            Ok(response)
        }
        // No error, page not found
        Ok((http_content, None)) => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from(http_content)))
                .unwrap();
            Ok(response)
        }
        // Error.
        Err(_error) => {
            server_error_handler::send_error_packet()
        }
    }
}
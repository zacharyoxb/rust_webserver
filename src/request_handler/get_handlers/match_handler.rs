// Standard library imports
use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;
use std::time::SystemTime;
use http_body_util::Full;
// External crate imports
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use hyper::header::ToStrError;
use crate::Cache;
use crate::html_getters::dir_accessor;
use crate::request_handler::handler_utils;

//TODO: handle match then handle both modified/match at the same time by calling both in main and analysing the responses
pub(crate) async fn handle_match(req: Request<hyper::body::Incoming>, cache: Arc<Cache>) -> Result<Response<Full<Bytes>>, Infallible> {
    // clone cache
    let cache_clone = Arc::clone(&cache);
    // check cache for the page
    let cache_results = Cache::read_cache(cache_clone, req.uri()).await;

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
            match dir_accessor::retrieve_resource(req.uri()).await {
                Ok((read_content, Some(read_last_modified))) => {
                    // assign to variables then cache response
                    http_content = read_content;
                    last_modified = read_last_modified;
                    Cache::write_cache(cache, req.uri(), &http_content, &last_modified).await;
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
    
    return if req.headers().get("If-Match").is_some() {
        // get string from header
        // TODO: If match can have several values, fix this
        let header_val = req.headers().get("If-Match").unwrap().to_str();
        let etag_to_test;
        match header_val {
            Ok(etag) => etag_to_test = etag.to_string(),
            Err(_) => return handler_utils::send_error_packet()
        }
        match Cache::read_cache_with_etag(Arc::clone(&cache_clone), etag_to_test).await {
            Some(_) => handler_utils::send_default_ok_packet(&http_content, &last_modified),
            None => handler_utils::send_precondition_failed_packet()
        }
    } else {
        match handler_utils::modified_since_request(req.headers().get("If-None-Match").unwrap(), last_modified) {
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
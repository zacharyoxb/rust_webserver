// Standard library imports
use std::sync::Arc;
use std::time::SystemTime;
// External crate imports
use hyper::Request;
use crate::Cache;
use crate::html_getters::dir_accessor;
use crate::request_handler::handler_utils;

//TODO: handle match then handle both modified/match at the same time by calling both in main and analysing the responses
pub(crate) async fn handle_match(req: Request<hyper::body::Incoming>, cache: Arc<Cache>) -> Result<bool, Box<dyn std::error::Error>> {
    // check cache for the page
    let cache_results = Cache::read_cache(Arc::clone(&cache), req.uri()).await;

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
                    Cache::write_cache(Arc::clone(&cache), req.uri(), &http_content, &last_modified).await;
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
        match Cache::etag_match(Arc::clone(&cache), req.headers().get("If-Match").unwrap()).await {
            Ok(true) => handler_utils::send_default_ok_packet(&http_content, &last_modified),
            Ok(false) => handler_utils::send_precondition_failed_packet(),
            Err(_) => handler_utils::send_error_packet()
        }
    } else {
        match Cache::etag_match(Arc::clone(&cache), req.headers().get("If-None-Match").unwrap()).await {
            Ok(true) => handler_utils::send_precondition_failed_packet(),
            Ok(false) => handler_utils::send_default_ok_packet(&http_content, &last_modified),
            Err(_) => handler_utils::send_error_packet()
        }
    }
}
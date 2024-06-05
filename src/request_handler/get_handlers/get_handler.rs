// Standard library imports
use std::convert::Infallible;
use std::sync::Arc;
use std::time::SystemTime;
use http_body_util::Full;
// External crate imports
use hyper::{Request, Response};
use hyper::body::Bytes;
// Internal crates
use crate::cache::cache_impl::Cache;
use crate::html_getters::dir_accessor;
use crate::request_handler::handler_utils;

// Handles get requests, returning either a get response packet / server error packet / 404 packet
// TODO: also handle matches
pub(crate) async fn handle_get(req: Request<hyper::body::Incoming>, cache: Arc<Cache>) -> Result<Response<Full<Bytes>>, Infallible> {
    // check cache to either use it or check if it's outdated if modified header
    let cache_results = Cache::read_cache(Arc::clone(&cache), req.uri()).await;
    
    // create variables to hold resource
    let mut http_content: Option<Bytes> = None;
    let mut last_modified: Option<SystemTime> = None;
    
    // create variables representing whether the send conditions have been triggered (if they exist)
    let mut modified_send: bool = false;
    let mut match_send: bool = false;
    
    // checks cache if there's no modified header
    if req.headers().get("If-Modified-Since").is_none() && req.headers().get("If-Unmodified-Since").is_none() {
        // if the cache contains the result
        if cache_results.is_some() {
            let (temp_content, temp_modified) = cache_results.clone().unwrap();
            (http_content, last_modified) = (Some(temp_content), Some(temp_modified))
        }
    }
    
    // checks resource directly if not in cache / didn't check cache
    if http_content.is_none() && last_modified.is_none() {
        match dir_accessor::retrieve_resource(req.uri()).await {
            Ok((read_content, Some(read_last_modified))) => {
                // assign to variables then cache response
                http_content = Some(Bytes::from(read_content));
                last_modified = Some(read_last_modified);
                // if not in cache (as opposed to not having read the cache)
                if cache_results.is_none() {
                    Cache::write_cache(Arc::clone(&cache), req.uri(), &http_content.clone().unwrap(), &last_modified.unwrap()).await;
                }
            }
            Ok((read_content, None)) => {
                return handler_utils::send_not_found_packet(read_content)
            }
            Err(_) => {
                return handler_utils::send_error_packet()
            }
        }
        
        // checks if request is stale by comparing directly with resource
        if req.headers().get("If-Modified-Since").is_some() || req.headers().get("If-Unmodified-Since").is_some() {
            if req.headers().get("If-Modified-Since").is_some() {
                // check if req is out of date
                match handler_utils::modified_since_request(req.headers().get("If-Modified-Since").unwrap(), last_modified.unwrap()) {
                    Ok(result) => modified_send = result,
                    Err(_) => return handler_utils::send_error_packet()
                }
            } else if req.headers().get("If-Unmodified-Since").is_some() {
                // check if req is out of date
                match handler_utils::modified_since_request(req.headers().get("If-Modified-Since").unwrap(), last_modified.unwrap()) {
                    Ok(result) => modified_send = !result,
                    Err(_) => return handler_utils::send_error_packet()
                }
            }
            // update cached resource if necessary
            if cache_results.is_some() {
                // if was in cache
                let (_cache_http_content, cache_last_modified) = cache_results.unwrap();
                // if cache is outdated, update it
                if handler_utils::modified_since_cache(cache_last_modified, last_modified.unwrap()) {
                    Cache::write_cache(cache, req.uri(), &http_content.clone().unwrap(), &last_modified.unwrap()).await
                }
            }
        }
    }
    
    handler_utils::send_default_ok_packet(&http_content.clone().unwrap(), &last_modified.unwrap())
}
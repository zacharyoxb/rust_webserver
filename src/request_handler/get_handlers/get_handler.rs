// Standard library imports
use std::convert::Infallible;
use std::sync::Arc;
use std::time::SystemTime;
use http_body_util::Full;
// External crate imports
use hyper::{HeaderMap, Request, Response};
use hyper::body::Bytes;
// Internal crates
use crate::cache::cache_impl::Cache;
use crate::html_getters::dir_accessor;
use crate::request_handler::handler_utils;

// Handles get requests, returning either a get response packet / server error packet / 404 packet
pub(crate) async fn handle_get(req: Request<hyper::body::Incoming>, cache: Arc<Cache>) -> Result<Response<Full<Bytes>>, Infallible> {
    // holds cache results (not found bool avoids borrow issues)
    let cache_result = Cache::read_cache(Arc::clone(&cache), req.uri()).await;
    let not_found_in_cache = cache_result.is_none();
    
    // variable indicating whether cache can be checked
    let can_check_cache = can_check_cache(req.headers());
    
    // variables for holding cache results/read results
    let mut content_tuple: Option<(Bytes, SystemTime, String)> = None;
    
    
    // check the cache for the requested resource
    if can_check_cache {
        if let Some(cached_content_tuple) = cache_result {
            content_tuple = Some(cached_content_tuple);
        }
    }
    
    // if wasn't in cache or couldn't check cache, do a direct read
    if content_tuple.is_none() {
        match dir_accessor::retrieve_resource(req.uri()).await {
            Ok((content, Some(last_modified))) => {
                let etag = Cache::generate_etag(&content);
                if not_found_in_cache {
                    // only cache if it isn't in cache
                    Cache::write_cache(Arc::clone(&cache), req.uri(), &content, &last_modified, &etag).await;
                }
                // store read values in tuple
                content_tuple = Some((content, last_modified, etag));
            }
            Ok((content, None)) => return handler_utils::send_not_found_packet(content),
            Err(..) => return handler_utils::send_error_packet()
        }
    }
    
    // forward read/cache results based on headers. If-None-Match overrides If-Modified-Since
    if req.headers().get("If-None-Match").is_some() {
        
    }
    
    todo!()
}

// handles an If-None-Match header
fn if_none_match() {
    
}

// returns false if the resource needs to be checked directly (i.e. conditional header)
fn can_check_cache(header_value: &HeaderMap) -> bool {
    if header_value.get("If-Match").is_some() || header_value.get("If-Unmodified-Since").is_some() {
        return false
    }
    true
}
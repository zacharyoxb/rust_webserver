// Standard library imports
use http_body_util::Full;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::SystemTime;
// External crate imports
use hyper::body::Bytes;
use hyper::{Request, Response};
// Internal crates
use crate::cache::cache_impl::Cache;
use crate::html_getters::dir_accessor;
use crate::method_handlers::handler_utils;

// Handles get requests, returning either a get response packet / server error packet / 404 packet
pub(crate) async fn handle_get(
    req: Request<hyper::body::Incoming>,
    cache: Arc<Cache>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // holds cache results (not found bool avoids borrow issues)
    let cache_result = Cache::read_cache(Arc::clone(&cache), req.uri()).await;
    let not_found_in_cache = cache_result.is_none();

    // variable indicating whether cache can be checked
    let can_check_cache = handler_utils::header_evals::can_check_cache(req.headers());

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
                    Cache::write_cache(
                        Arc::clone(&cache),
                        req.uri(),
                        &content,
                        &last_modified,
                        &etag,
                    )
                    .await;
                }
                // store read values in tuple
                content_tuple = Some((content, last_modified, etag));
            }
            Ok((content, None)) => {
                return handler_utils::packet_templates::send_not_found_packet(content)
            }
            Err(..) => return handler_utils::packet_templates::send_error_packet(),
        }
    }

    // PRECEDENCE OF PRECONDITIONS: https://www.rfc-editor.org/rfc/rfc9110#section-13.2.2
    if let Some(header) = req.headers().get("If-Match") {
        if !handler_utils::header_evals::if_match(header) {
            return handler_utils::packet_templates::send_precondition_failed_packet();
        }
    } else if let Some(header) = req.headers().get("If-Unmodified-Since") {
        if !handler_utils::header_evals::if_modified_since(header) {
            return handler_utils::packet_templates::send_precondition_failed_packet();
        }
    }

    if let Some(header) = req.headers().get("If-None-Match") {
        if !handler_utils::header_evals::if_none_match(header) {
            return handler_utils::packet_templates::send_not_modified_packet();
        }
    } else if let Some(header) = req.headers().get("If-Modified-Since") {
        if !handler_utils::header_evals::if_modified_since(header) {
            return handler_utils::packet_templates::send_not_modified_packet();
        }
    }

    // returning content if there is a range header
    if let (Some(range_header), Some(if_range_header)) =
        (req.headers().get("Range"), req.headers().get("If-Range"))
    {
        if let Some(partial_content) = handler_utils::header_evals::if_range(
            range_header,
            if_range_header,
            &content_tuple.as_ref().unwrap().0,
        ) {
            // send partial content
        }
    }

    // return content
    if let Some((content, last_modified, etag)) = content_tuple {
        handler_utils::packet_templates::send_default_ok_packet(&content, &last_modified, &etag)
    } else {
        eprintln!("content_tuple was none!");
        handler_utils::packet_templates::send_error_packet()
    }
}

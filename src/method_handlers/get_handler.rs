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

/// struct for holding web content
struct WebContent {
    data: Bytes,
    last_modified: SystemTime,
    etag: String,
}

impl WebContent {
    fn new(data: Bytes, last_modified: SystemTime, etag: String) -> Self {
        Self {
            data,
            last_modified,
            etag,
        }
    }
}

/// Handles get requests, returning either a get response packet / server error packet / 404 packet
pub(crate) async fn handle_get(
    req: Request<hyper::body::Incoming>,
    cache: Arc<Cache>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // holds cache results (not found bool avoids borrow issues)
    let cache_result = Cache::read_cache(Arc::clone(&cache), req.uri()).await;

    // variable indicating whether cache can be checked
    let can_check_cache = handler_utils::header_evals::can_check_cache(req.headers());

    // variable holding the etag of the cache value (if found) to check for staleness
    let cache_etag = cache_result
        .clone()
        .map(|(_, _, etag)| etag)
        .unwrap_or_else(|| "".to_string());

    // content temporarily wrapped in an option
    let mut wrapped_content: Option<WebContent> = None;

    // check the cache for the requested resource
    if can_check_cache {
        if let Some((data, last_modified, etag)) = cache_result {
            wrapped_content = Some(WebContent::new(data, last_modified, etag));
        }
    }

    // if wasn't in cache or couldn't check cache, do a direct read
    if wrapped_content.is_none() {
        match dir_accessor::retrieve_resource(req.uri()).await {
            Ok((data, Some(last_modified))) => {
                let etag = Cache::generate_etag(&data);
                // if wasn't in cache, or etags don't match
                if cache_etag.is_empty() || cache_etag != etag {
                    Cache::write_cache(Arc::clone(&cache), req.uri(), &data, &last_modified, &etag)
                        .await;
                }
                // store read values in struct
                wrapped_content = Some(WebContent::new(data, last_modified, etag));
            }
            Ok((content, None)) => {
                return handler_utils::packet_templates::send_not_found_packet(content)
            }
            Err(..) => return handler_utils::packet_templates::send_error_packet(),
        }
    }

    // WebContent should be Some by now: exit if it isn't
    if wrapped_content.is_none() {
        eprintln!("Wrapped content is still none!");
        return handler_utils::packet_templates::send_error_packet();
    }

    // now WebContent can be safely unwrapped
    let web_content: WebContent = wrapped_content.unwrap();

    // tracks valid headers
    let mut valid_is_match = false;
    let mut valid_if_none_match = false;

    // PRECEDENCE OF PRECONDITIONS: https://www.rfc-editor.org/rfc/rfc9110#section-13.2.2

    // Handle If-Match when header present
    if let Some(header) = req.headers().get("If-Match") {
        match handler_utils::header_evals::if_match(header, &web_content.etag) {
            Ok(true) => valid_is_match = true,
            Ok(false) => return handler_utils::packet_templates::send_precondition_failed_packet(),
            Err(_) => {}
        }
    }

    // Handle If-Unmodified-Since when header present and valid If-Match header is not present
    if let Some(header) = req.headers().get("If-Unmodified-Since") {
        if !valid_is_match {
            if let Ok(false) =
                handler_utils::header_evals::if_unmodified_since(header, &web_content.last_modified)
            {
                return handler_utils::packet_templates::send_precondition_failed_packet();
            }
        }
    }

    // Handle If-None-Match when header present
    if let Some(header) = req.headers().get("If-None-Match") {
        match handler_utils::header_evals::if_none_match(header, &web_content.etag) {
            Ok(true) => valid_if_none_match = true,
            Ok(false) => return handler_utils::packet_templates::send_not_modified_packet(),
            Err(_) => {}
        }
    }

    // Handle If-Modified-Since when header present and valid If-None-Match is not present
    if let Some(header) = req.headers().get("If-Modified-Since") {
        if !valid_if_none_match {
            if let Ok(false) =
                handler_utils::header_evals::if_modified_since(header, &web_content.last_modified)
            {
                return handler_utils::packet_templates::send_not_modified_packet();
            }
        }
    }

    // Handle If-Range when header present
    if let (Some(range_header), if_range_header, date_header) = (
        req.headers().get("Range"),
        req.headers().get("If-Range"),
        req.headers().get("Date"),
    ) {
        if let Ok(true) = handler_utils::header_evals::if_range(
            if_range_header,
            &web_content.last_modified,
            &web_content.etag,
            date_header,
        ) {
            if let Ok(sliced_content) =
                handler_utils::header_evals::range(&web_content.data, range_header)
            {
                return if sliced_content.len() == 1 {
                    let (data, start, end) = &sliced_content[0];
                    handler_utils::packet_templates::send_partial_content_packet(
                        data.clone(),
                        start,
                        end,
                        &web_content.data.len(),
                        &web_content.last_modified,
                        &web_content.etag,
                    )
                } else {
                    handler_utils::packet_templates::send_multipart_packet(
                        sliced_content,
                        &web_content.data.len(),
                        &web_content.last_modified,
                        &web_content.etag,
                    )
                };
            }
        }
    }

    // If no If-Range header, send ok response
    handler_utils::packet_templates::send_default_ok_packet(
        web_content.data,
        web_content.last_modified,
        &web_content.etag,
    )
}

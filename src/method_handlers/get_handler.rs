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
use crate::html_getters;
use crate::html_getters::dir_accessor;
use crate::method_handlers::{handler_utils, response_gen};

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
    match html_getters::web_content::get_web_content(&req, Arc::clone(&cache)).await {
        Ok(web_content) => response_gen::head_get_resp::generate_response(&req, web_content).await,
        Err(_) => handler_utils::packet_templates::send_error_packet(),
    }
}

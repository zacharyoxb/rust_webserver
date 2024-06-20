// Standard library imports
use http_body_util::Full;
use std::convert::Infallible;
use std::sync::Arc;
// External crate imports
use crate::cache::cache_impl::Cache;
use crate::html_getters;
use crate::method_handlers::{handler_utils, response_gen};
use chrono::format::StrftimeItems;
use chrono::offset;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use sysinfo::System;

// Handles option requests, returning either a option response packet or server error packet
pub(crate) async fn handle_head(
    req: Request<hyper::body::Incoming>,
    cache: Arc<Cache>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match html_getters::web_content::get_web_content(&req, Arc::clone(&cache)).await {
        Ok(web_content) => {
            let mut response =
                response_gen::head_get_resp::generate_response(&req, web_content, true).await?;
            *response.body_mut() = Full::from(Bytes::new());
            Ok(response)
        }
        Err(_) => handler_utils::packet_templates::send_error_packet(),
    }
}

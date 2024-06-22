use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

use crate::cache::cache_impl::Cache;
use crate::method_handlers::{handler_utils, response_gen};
use crate::resource_getters;

// Handles option requests, returning either a option response packet or server error packet
pub(crate) async fn handle_head(
    req: Request<hyper::body::Incoming>,
    cache: Arc<Cache>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match resource_getters::web_content::get_web_content(&req, Arc::clone(&cache)).await {
        Ok(web_content) => {
            let mut response = response_gen::get_resp::generate_response(&req, web_content).await?;
            *response.body_mut() = Full::from(Bytes::new());
            Ok(response)
        }
        Err(_) => handler_utils::packet_templates::send_error_packet(),
    }
}

// Standard library imports
use std::convert::Infallible;
use std::sync::Arc;
use http_body_util::Full;
// External crate imports
use hyper::body::Bytes;
use hyper::{Request, Response};
use crate::Cache;
use crate::request_handler::handler_utils;

//TODO: handle match then handle both modified/match at the same time by calling both in main and analysing the responses
pub(crate) async fn handle_match(_req: Request<hyper::body::Incoming>, _cache: Arc<Cache>) -> Result<Response<Full<Bytes>>, Infallible> {
    handler_utils::send_not_implemented_packet()
}
// Standard library imports
use std::convert::Infallible;
use http_body_util::Full;
// External crate imports
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use crate::Cache;

//TODO: handle match then handle both modified/match at the same time by calling both in main and analysing the responses
pub(crate) async fn handle_match(req: Request<hyper::body::Incoming>, cache: Cache) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    return Ok(response)
}
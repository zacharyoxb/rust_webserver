// Standard library imports
use http_body_util::Full;
use std::convert::Infallible;
// External crate imports
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};

// Handles option requests, returning either a option response packet or server error packet
pub(crate) async fn handle_delete(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    return Ok(response);
}

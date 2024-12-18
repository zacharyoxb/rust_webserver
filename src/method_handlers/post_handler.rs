use std::convert::Infallible;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};

// Handles option requests, returning either a option response packet or server error packet
pub(crate) async fn handle_post(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

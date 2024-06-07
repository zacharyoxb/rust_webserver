// Standard library imports
use http_body_util::Full;
use std::convert::Infallible;
// External crate imports
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};

// Handles option requests, returning either a option response packet or server error packet
pub(crate) async fn handle_option(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    //TODO: When completing handlers, add to the headers
    let response = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Allow", "GET, OPTIONS")
        .header("Access-Control-Allow-Methods", "GET, OPTIONS, HEAD")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Credentials", "true")
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

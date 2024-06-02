use std::convert::Infallible;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use hyper::body::Bytes;

pub(crate) fn send_error_packet() -> Result<Response<Full<Bytes>>, Infallible>{
    let response = Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}
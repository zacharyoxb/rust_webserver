use chrono::{DateTime, Days, Utc};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{
    CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, DATE, ETAG, EXPIRES, LAST_MODIFIED, SERVER,
};
use hyper::{Response, StatusCode};
use std::convert::Infallible;
use std::time::SystemTime;

pub(crate) fn send_default_ok_packet(
    http_content: &Bytes,
    last_modified: &SystemTime,
    etag: &str,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(DATE, get_current_http_date())
        .header(CONTENT_TYPE, "text/html")
        .header(CONTENT_LENGTH, (*http_content).len())
        .header(LAST_MODIFIED, system_time_to_http_date(&last_modified))
        .header(EXPIRES, get_http_expiry_date())
        .header(ETAG, etag)
        .header(CACHE_CONTROL, "max-age=36000")
        .header(SERVER, "RUST-SERVER-ZACHARYOXB")
        .body(Full::new(Bytes::from(http_content.clone())))
        .unwrap();
    Ok(response)
}

pub(crate) fn send_not_found_packet(
    http_content: Bytes,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(http_content))
        .unwrap();
    return Ok(response);
}
pub(crate) fn send_error_packet() -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

pub(crate) fn send_not_implemented_packet() -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

pub(crate) fn send_precondition_failed_packet() -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::PRECONDITION_FAILED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

pub(crate) fn send_not_modified_packet() -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

fn system_time_to_http_date(time: &SystemTime) -> String {
    let datetime: DateTime<Utc> = (*time).into();
    datetime.to_rfc2822()
}

fn get_current_http_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc2822()
}

fn get_http_expiry_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    (now + Days::new(4)).to_rfc2822()
}

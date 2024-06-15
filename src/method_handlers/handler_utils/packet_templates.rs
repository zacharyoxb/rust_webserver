use chrono::{DateTime, Days, Utc};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{
    CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, DATE, ETAG, EXPIRES, LAST_MODIFIED,
    SERVER,
};
use hyper::{Response, StatusCode};
use std::convert::Infallible;
use std::time::SystemTime;

/// sends ok packet
pub(crate) fn send_default_ok_packet(
    http_content: Bytes,
    last_modified: SystemTime,
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
        .body(Full::new(http_content.clone()))
        .unwrap();
    Ok(response)
}

/// sends partial content packet (where there is only 1 part)
pub(crate) fn send_partial_content_packet(
    partial_content_tuple: (Bytes, u64, u64),
    last_modified: &SystemTime,
    etag: &str,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let content_range = format!(
        "bytes {}-{}/{}",
        partial_content_tuple.1,
        partial_content_tuple.2,
        partial_content_tuple.0.len()
    );

    let response = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(DATE, get_current_http_date())
        .header(CONTENT_TYPE, "text/html")
        .header(CONTENT_RANGE, content_range)
        .header(CONTENT_LENGTH, partial_content_tuple.0.len())
        .header(LAST_MODIFIED, system_time_to_http_date(last_modified))
        .header(EXPIRES, get_http_expiry_date())
        .header(ETAG, etag)
        .header(CACHE_CONTROL, "max-age=36000")
        .header(SERVER, "RUST-SERVER-ZACHARYOXB")
        .body(Full::new(partial_content_tuple.0))
        .unwrap();
    Ok(response)
}

/// sends partial content packet (where there are several parts)
pub(crate) fn send_multipart_packet() {}

/// sends 404 not found packet
pub(crate) fn send_not_found_packet(
    http_content: Bytes,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(http_content))
        .unwrap();
    Ok(response)
}

/// sends internal server error packet
pub(crate) fn send_error_packet() -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

/// sends not implemented packet
pub(crate) fn send_not_implemented_packet() -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

/// sends a precondition failed packet
pub(crate) fn send_precondition_failed_packet() -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::PRECONDITION_FAILED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

/// sends not modified packet
pub(crate) fn send_not_modified_packet() -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}

/// converts system time to http formatted date for packet sending
fn system_time_to_http_date(time: &SystemTime) -> String {
    let datetime: DateTime<Utc> = (*time).into();
    datetime.to_rfc2822()
}

/// gets the current date in http format
fn get_current_http_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc2822()
}

/// gets the set expiry date in http format
fn get_http_expiry_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    (now + Days::new(4)).to_rfc2822()
}

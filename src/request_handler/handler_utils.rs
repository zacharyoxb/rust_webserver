// Standard library imports
use std::convert::Infallible;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::io::ErrorKind;
use std::time::SystemTime;
use chrono::{DateTime, Days, Utc};
use http_body_util::Full;
// External crate imports
use hyper::{Response, StatusCode};
use hyper::body::Bytes;
use hyper::header::HeaderValue;


pub(crate) fn send_not_found_packet(http_content: String) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from(http_content)))
        .unwrap();
    return Ok(response)
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

pub(crate) fn modified_since(req_header_val: &HeaderValue, page_last_modified: SystemTime) -> Result<bool, io::Error> {
    // header to string
    let req_modified_str = req_header_val.to_str()
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid header value string"))?;
    // string to time
    let request_last_modified = DateTime::parse_from_rfc2822(req_modified_str)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid date format"))?;
    let page_modified_datetime: DateTime<Utc> = DateTime::from(page_last_modified);
    Ok(request_last_modified < page_modified_datetime)
}

pub(crate) fn system_time_to_http_date(time: &SystemTime) -> String {
    let datetime: DateTime<Utc> = (*time).into();
    datetime.to_rfc2822()
}

pub(crate) fn get_current_http_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc2822()
}

pub(crate) fn get_http_expiry_date() -> String {
    let now: DateTime<Utc> = Utc::now();

    (now + Days::new(4)).to_rfc2822()
}

pub(crate) fn generate_etag(http_content: &String) -> String {
    let mut hasher = DefaultHasher::new();
    (*http_content).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
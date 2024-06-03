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
use hyper::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, DATE, ETAG, EXPIRES, HeaderValue, LAST_MODIFIED, SERVER};

//TODO: handle several content types
pub(crate) fn send_default_ok_packet(http_content: &String, last_modified: &SystemTime) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(DATE, get_current_http_date())
        .header(CONTENT_TYPE, "text/html")
        .header(CONTENT_LENGTH, (*http_content).len())
        .header(LAST_MODIFIED, system_time_to_http_date(&last_modified))
        .header(EXPIRES, get_http_expiry_date())
        .header(ETAG, generate_etag(&http_content))
        .header(CACHE_CONTROL, "max-age=36000")
        .header(SERVER, "RUST-SERVER-ZACHARYOXB")
        .body(Full::new(Bytes::from(http_content.clone())))
        .unwrap();
    Ok(response)
}

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

// checks if the request is out of date or not, returning true if the page has been modified since
// takes a HeaderValue and a SystemTime value
pub(crate) fn modified_since_request(req_last_modified: &HeaderValue, file_last_modified: SystemTime) -> Result<bool, io::Error> {
    // header to string
    let req_modified_str = req_last_modified.to_str()
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid header value string"))?;
    // string to time
    let request_last_modified = DateTime::parse_from_rfc2822(req_modified_str)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid date format"))?;
    let page_modified_datetime: DateTime<Utc> = DateTime::from(file_last_modified);
    Ok(request_last_modified < page_modified_datetime)
}

// checks if the request is out of date or not, returning true if the page has been modified since
// this version takes 2 SystemTime variables
pub(crate) fn modified_since_cache(cache_last_modified: SystemTime, file_last_modified: SystemTime) -> bool {
    cache_last_modified < file_last_modified
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
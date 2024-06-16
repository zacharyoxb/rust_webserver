use chrono::{DateTime, Days, Utc};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{ACCEPT_RANGES, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, DATE, ETAG, EXPIRES, LAST_MODIFIED, SERVER};
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
        .header(CONTENT_LENGTH, http_content.len())
        .header(LAST_MODIFIED, system_time_to_http_date(&last_modified))
        .header(EXPIRES, get_http_expiry_date())
        .header(ETAG, etag)
        .header(ACCEPT_RANGES, "bytes")
        .header(CACHE_CONTROL, "max-age=36000")
        .header(SERVER, "RUST-SERVER-ZACHARYOXB")
        .body(Full::new(http_content))
        .unwrap();
    Ok(response)
}

/// sends partial content packet (where there is only 1 part)
pub(crate) fn send_partial_content_packet(
    data_slice: Bytes,
    slice_start: &u64,
    slice_end: &u64,
    original_length: &usize,
    last_modified: &SystemTime,
    etag: &str,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let content_range = format!(
        "bytes {}-{}/{}",
        slice_start,
        slice_end,
        original_length
    );

    let response = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(DATE, get_current_http_date())
        .header(CONTENT_TYPE, "text/html")
        .header(CONTENT_RANGE, content_range)
        .header(CONTENT_LENGTH, data_slice.len())
        .header(LAST_MODIFIED, system_time_to_http_date(last_modified))
        .header(EXPIRES, get_http_expiry_date())
        .header(ETAG, etag)
        .header(ACCEPT_RANGES, "bytes")
        .header(CACHE_CONTROL, "max-age=36000")
        .header(SERVER, "RUST-SERVER-ZACHARYOXB")
        .body(Full::new(data_slice))
        .unwrap();
    Ok(response)
}

/// sends partial content packet (where there are several parts)
pub(crate) fn send_multipart_packet(ranges_vector: Vec<(Bytes, u64, u64)>, original_length: &usize) -> Result<Response<Full<Bytes>>, Infallible> {
    let boundary = "BOUNDARY";
    
    let mut multipart_body = Vec::new();
    
    for (slice, start, end) in ranges_vector {
        multipart_body.push(format!(
            "--{}\r\nContent-Type: {}\r\nContent-Range: bytes {}-{}/{}\r\n\r\n",
            boundary, "text/html", start, end, original_length
        ).into_bytes());
        multipart_body.push(slice.to_vec());
        multipart_body.push(b"\r\n".to_vec());
    }
    multipart_body.push(format!("--{}--\r\n", boundary).into_bytes());

    // Flatten the body into a single Vec<u8>
    let body: Vec<u8> = multipart_body.into_iter().flatten().collect();

    // Create the response
    let response = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header("Content-Type", format!("multipart/byteranges; boundary={}", boundary))
        .header("Content-Length", body.len())
        .body(Full::from(Bytes::from(body)))
        .unwrap();

    Ok(response)
}

/// sends 404 not found packet
pub(crate) fn send_not_found_packet(
    data: Bytes,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(data))
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

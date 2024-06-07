use std::time::SystemTime;
use chrono::{DateTime, Utc};
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::HeaderMap;

// evaluates If-Match precondition (None = invalid header, ignore this header)
pub(crate) fn if_match(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    if let Ok(etag_str) = etag_header.to_str() {
        // convert split to vector
        let etags: Vec<&str> = etag_str.split(", ").collect();
        
        if etags.len() == 1 {
            Some(etags[0] == "*" || etags[0] == resource_etag)
        } else {
            Some(etags.iter().any(|&etag| etag == resource_etag))
        }
    } else {
        None
    }
}

// evaluates If-Unmodified-Since precondition (None = invalid header, ignore this header)
pub(crate) fn if_unmodified_since(header_modified_since: &HeaderValue, resource_modified_since: &SystemTime) -> Option<bool> {
    if let Ok(header_modified_str) = header_modified_since.to_str() {
        // convert header val to datetime
        if let Ok(header_datetime) = DateTime::parse_from_rfc2822(header_modified_str) {
            // convert header
            let header_datetime_utc: DateTime<Utc> = header_datetime.with_timezone(&Utc);
            // convert resource
            let resource_datetime_utc: DateTime<Utc> = DateTime::from(*resource_modified_since);

            return Some(header_datetime_utc > resource_datetime_utc)
        }
    }
    None
}

// evaluates If-None-Match precondition
pub(crate) fn if_none_match(header: &HeaderValue) -> bool {
    todo!()
}

// evaluates If-Modified-Since precondition
pub(crate) fn if_modified_since(header: &HeaderValue) -> bool {
    todo!()
}

// evaluates If-Range precondition (returns partial content if range is applicable, otherwise None)
pub(crate) fn if_range(
    range_header: &HeaderValue,
    if_range_header: &HeaderValue,
    http_content: &Bytes,
) -> Option<Bytes> {
    todo!()
}

// returns true if according to http spec the cache can be checked based on req headers
pub(crate) fn can_check_cache(header_value: &HeaderMap) -> bool {
    if header_value.get("If-Match").is_some() || header_value.get("If-Unmodified-Since").is_some() {
        return false;
    }
    true
}

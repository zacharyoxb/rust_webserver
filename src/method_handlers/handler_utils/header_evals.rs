use chrono::{DateTime, Utc};
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use std::time::SystemTime;

/// evaluates If-Match precondition (None = invalid header, ignore this header)
pub(crate) fn if_match(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    get_any_matches(etag_header, resource_etag)
}

/// evaluates If-Unmodified-Since precondition (None = invalid header, ignore this header)
pub(crate) fn if_unmodified_since(
    header_modified_since: &HeaderValue,
    resource_modified_since: &SystemTime,
) -> Option<bool> {
    convert_to_datetime(header_modified_since, resource_modified_since)
        .map(|(header_date, resource_date)| header_date >= resource_date)
}

/// evaluates If-None-Match precondition (None = invalid header, ignore this header)
pub(crate) fn if_none_match(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    get_any_matches(etag_header, resource_etag).map(|is_match| !is_match)
}

/// evaluates If-Modified-Since precondition (None = invalid header, ignore this header)
pub(crate) fn if_modified_since(
    header_modified_since: &HeaderValue,
    resource_modified_since: &SystemTime,
) -> Option<bool> {
    convert_to_datetime(header_modified_since, resource_modified_since)
        .map(|(header_date, resource_date)| header_date < resource_date)
}

/// evaluates If-Range precondition (returns partial content if range is applicable, otherwise None)
pub(crate) fn if_range(
    range_header: &HeaderValue,
    if_range_header: &HeaderValue,
    http_content: &Bytes,
) -> Option<Bytes> {
    todo!()
}

/// returns true if according to http spec the cache can be checked based on request headers
pub(crate) fn can_check_cache(header_value: &HeaderMap) -> bool {
    if header_value.get("If-Match").is_some() || header_value.get("If-Unmodified-Since").is_some() {
        return false;
    }
    true
}

/// converts the HeaderValue from the request header and the SystemTime from the cache/metadata
/// into DateTime<Utc>. Used for handling Modified headers.
fn convert_to_datetime(
    header_date: &HeaderValue,
    resource_date: &SystemTime,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    if let Ok(header_date_str) = header_date.to_str() {
        // convert header val to datetime
        if let Ok(header_datetime) = DateTime::parse_from_rfc2822(header_date_str) {
            // convert header
            let header_datetime_utc: DateTime<Utc> = header_datetime.with_timezone(&Utc);
            // convert resource
            let resource_datetime_utc: DateTime<Utc> = DateTime::from(*resource_date);

            Some((header_datetime_utc, resource_datetime_utc))
        } else {
            None
        }
    } else {
        None
    }
}

/// checks if there are any matches between etag(s) in the header and the resource etag
fn get_any_matches(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    if let Ok(etag_str) = etag_header.to_str() {
        // convert split to vector
        let etags: Vec<&str> = etag_str.split(", ").collect();

        if etags.len() > 1 && etags.contains(&"*") {
            None
        } else {
            Some(
                etags
                    .iter()
                    .any(|&etag| etag == "*" || etag == resource_etag),
            )
        }
    } else {
        None
    }
}

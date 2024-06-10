use chrono::{DateTime, Utc};
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use std::time::SystemTime;

/// evaluates If-Match precondition (None = invalid header, ignore this header)
pub(crate) fn if_match(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    strong_compare(etag_header, resource_etag)
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
    weak_compare(etag_header, resource_etag).map(|is_match| !is_match)
}

/// evaluates If-Modified-Since precondition (None = invalid header, ignore this header)
pub(crate) fn if_modified_since(
    header_modified_since: &HeaderValue,
    resource_modified_since: &SystemTime,
) -> Option<bool> {
    convert_to_datetime(header_modified_since, resource_modified_since)
        .map(|(header_date, resource_date)| header_date < resource_date)
}

/// evaluates If-Range precondition (None = invalid header, ignore this header)
pub(crate) fn if_range(
    if_range_header: Option<&HeaderValue>,
    modified_since: &SystemTime,
    etag: &String,
) -> Option<bool> {
    // check if there is an If-Range header
    if let Some(if_range_some) = if_range_header {
        // convert to str
        if let Ok(header_str) = if_range_some.to_str() {
            // check if etag or date
            if header_str.len() >= 3 && (0..3).any(|i| &header_str[i..i+1] == "\"") {
                 // check if etag matches
                
            } else {
                // check if date matches
            }
        } else {
            return None
        }
    } else {
        // If if_range_header is None
        return Some(true)
    }
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

/// does a strong comparison (returns true if there is at least 1 match)
fn strong_compare(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    if let Ok(etag_str) = etag_header.to_str() {
        // if tag is weak, return None
        if etag_str.starts_with("W/") || resource_etag.starts_with("W/") {
            return None
        } else {
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
        }
    } else {
        None
    }
}

/// does a weak comparison (returns true if there is at least 1 match)
fn weak_compare(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    if let Ok(etag_str) = etag_header.to_str() {
        let clean_header_etag = etag_str.strip_prefix("W/").unwrap_or(etag_str);
        let clean_resource_etag = resource_etag.strip_prefix("W/").unwrap_or(resource_etag);
        // convert split to vector
        let etags: Vec<&str> = clean_header_etag.split(", ").collect();

        if etags.len() > 1 && etags.contains(&"*") {
            None
        } else {
            Some(
                etags
                    .iter()
                    .any(|&etag| etag == "*" || etag == clean_resource_etag),
            )
        }
    } else {
        None
    }
}

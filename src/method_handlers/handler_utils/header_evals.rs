use std::time::SystemTime;

use chrono::{DateTime, Duration, Utc};
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::HeaderMap;

const MAX_RANGE_COUNT: usize = 100;

/// evaluates If-Match precondition (Err = invalid header, ignore this header)
pub(crate) fn if_match(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    strong_compare(etag_header, resource_etag)
}

/// evaluates If-Unmodified-Since precondition (Err = invalid header, ignore this header)
pub(crate) fn if_unmodified_since(
    header_modified_since: &HeaderValue,
    resource_modified_since: &SystemTime,
) -> Option<bool> {
    header_to_date(header_modified_since)
        .map(|header_date| header_date >= DateTime::<Utc>::from(*resource_modified_since))
}

/// evaluates If-None-Match precondition (Err = invalid header, ignore this header)
pub(crate) fn if_none_match(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    weak_compare(etag_header, resource_etag).map(|is_match| !is_match)
}

/// evaluates If-Modified-Since precondition (Err = invalid header, ignore this header)
pub(crate) fn if_modified_since(
    header_modified_since: &HeaderValue,
    resource_modified_since: &SystemTime,
) -> Option<bool> {
    header_to_date(header_modified_since)
        .map(|header_date| header_date < DateTime::<Utc>::from(*resource_modified_since))
}

/// evaluates If-Range precondition (Err = invalid header, ignore this header)
pub(crate) fn if_range(
    if_range_header: Option<&HeaderValue>,
    modified_since: &SystemTime,
    etag: &str,
    date_validator: Option<&HeaderValue>,
) -> Option<bool> {
    // If there's no If-Range header, there's no condition
    let if_range_some = match if_range_header {
        Some(header) => header,
        None => return Some(true),
    };

    // Try parsing the If-Range header as a date
    match header_to_date(if_range_some) {
        Some(if_range_date) => {
            // If there's no date header, the date is weak
            let date_val_header = match date_validator {
                Some(header) => header,
                None => return Some(false),
            };

            // Convert the date header to a date
            let date_val = match header_to_date(date_val_header) {
                Some(date) => date,
                None => return None,
            };

            // Compare the dates with a 1-second tolerance
            let resource_mod_time: DateTime<Utc> = DateTime::from(*modified_since);
            if (date_val - resource_mod_time) > Duration::seconds(1) {
                // Strong date: check if resource and client If-Range condition match
                Some(if_range_date == resource_mod_time)
            } else {
                // Weak date: condition failed
                Some(false)
            }
        }
        None => {
            // Assume the If-Range header is an ETag if parsing as a date failed
            strong_compare(if_range_some, etag)
        }
    }
}

/// returns bytes of the requested range
pub(crate) fn range(content: &Bytes, range_header: &HeaderValue) -> Option<Vec<(Bytes, u64, u64)>> {
    // content length for checking ranges
    let content_length = content.len() as u64;

    let range_str = match range_header.to_str() {
        Ok(range_str) => range_str,
        Err(_) => return None,
    };

    let stripped_str = match range_str.strip_prefix("bytes=") {
        Some(stripped_str) => stripped_str,
        None => return None,
    };

    let range_pairs: Vec<&str> = stripped_str.split(',').collect();
    let mut ranges: Vec<(u64, u64)> = Vec::new();

    // check if max range count exceeded
    if range_pairs.len() > MAX_RANGE_COUNT {
        return None;
    }

    // get the ranges from the string
    for pair in range_pairs {
        let parts: Vec<&str> = pair.split('-').collect();
        ranges.push(try_get_range(&parts, content_length)?);
    }

    // check if ranges is ascending (if only 1 range true by default)
    let is_ascending = if ranges.len() == 1 {
        true
    } else {
        ranges
            .iter()
            .zip(ranges.iter().skip(1))
            .all(|((start1, _), (start2, _))| start1 <= start2)
    };

    if !is_ascending {
        return None;
    }

    // check if ranges overlaps more than once (if only 1 range false by default)
    let many_overlaps = if ranges.len() == 1 {
        false
    } else {
        let mut overlap_count = 0;
        ranges
            .iter()
            .zip(ranges.iter().skip(1))
            .any(|((_, end1), (start2, _))| {
                if end1 >= start2 {
                    overlap_count += 1;
                    overlap_count == 2
                } else {
                    false
                }
            })
    };

    if many_overlaps {
        return None;
    }

    let mut sliced_content: Vec<(Bytes, u64, u64)> = Vec::new();
    // if in ascending order and there is not more than 1 overlap, slice content
    for &(start, end) in ranges.iter() {
        sliced_content.push((slice_with_range(start, end, content)?, start, end))
    }
    Some(sliced_content)
}

/// if range is valid, return range start and end in u64
fn try_get_range(range_slice: &[&str], content_length: u64) -> Option<(u64, u64)> {
    match (range_slice[0].is_empty(), range_slice[1].is_empty()) {
        (false, false) => {
            // range x-y
            let (start, end) = match (range_slice[0].parse::<u64>(), range_slice[1].parse::<u64>())
            {
                (Ok(start), Ok(end)) => (start, end),
                _ => return None,
            };

            if start >= content_length || start > end {
                return None;
            }

            if end >= content_length {
                return None;
            }

            Some((start, end))
        }
        (false, true) => {
            // range x-
            let from_start = match range_slice[0].parse::<u64>() {
                Ok(from_start) => from_start,
                Err(_) => return None,
            };

            if from_start < content_length {
                Some((from_start, content_length - 1))
            } else {
                None
            }
        }
        (true, false) => {
            // range -y
            let from_end = match range_slice[1].parse::<u64>() {
                Ok(from_end) => from_end,
                Err(_) => return None,
            };

            if from_end < content_length {
                Some(((content_length - 1) - from_end, content_length - 1))
            } else {
                Some((0, content_length - 1))
            }
        }
        _ => None,
    }
}

/// slices content according to range
fn slice_with_range(start: u64, end: u64, content: &Bytes) -> Option<Bytes> {
    let start_index = match usize::try_from(start) {
        Ok(index) => index,
        Err(_) => return None,
    };

    let end_index = match usize::try_from(end) {
        Ok(index) => index,
        Err(_) => return None,
    };

    Some(content.slice(start_index..end_index + 1))
}

/// returns true if according to http spec the cache can be checked based on request headers
pub(crate) fn can_check_cache(header_value: &HeaderMap) -> bool {
    if header_value.get("If-Match").is_some() || header_value.get("If-Unmodified-Since").is_some() {
        return false;
    }
    true
}

/// converts a HeaderValue to a utc date. Returns None if the header is in a bad date format.
fn header_to_date(header_date: &HeaderValue) -> Option<DateTime<Utc>> {
    let header_str = match header_date.to_str() {
        Ok(string) => string,
        Err(_) => return None,
    };

    DateTime::parse_from_rfc2822(header_str)
        .map(|no_timezone| Some(no_timezone.with_timezone(&Utc)))
        .unwrap_or(None)
}

/// does a strong comparison (returns true if there is at least 1 match).
fn strong_compare(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    let etag_str = match etag_header.to_str() {
        Ok(string) => string,
        Err(_) => return None,
    };

    if etag_str.starts_with("W/") || resource_etag.starts_with("W/") {
        None
    } else {
        // convert split to vector
        let etags: Vec<&str> = etag_str.split(", ").collect();

        compare_etag_vec(&etags, resource_etag)
    }
}

/// does a weak comparison (returns true if there is at least 1 match)
fn weak_compare(etag_header: &HeaderValue, resource_etag: &str) -> Option<bool> {
    if let Ok(etag_str) = etag_header.to_str() {
        let clean_header_etag = etag_str.strip_prefix("W/").unwrap_or(etag_str);
        let clean_resource_etag = resource_etag.strip_prefix("W/").unwrap_or(resource_etag);
        // convert split to vector
        let etags: Vec<&str> = clean_header_etag.split(", ").collect();

        compare_etag_vec(&etags, clean_resource_etag)
    } else {
        None
    }
}

// compares 2 etag vectors for matches or *. Invalid vectors return None
fn compare_etag_vec(header_slice: &[&str], resource_etag: &str) -> Option<bool> {
    if header_slice.len() > 1 && header_slice.contains(&"*") {
        None
    } else {
        Some(
            header_slice
                .iter()
                .any(|&etag| etag == "*" || etag == resource_etag),
        )
    }
}

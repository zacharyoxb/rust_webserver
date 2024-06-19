use crate::method_handlers::handler_utils::error::HeaderError;
use chrono::{DateTime, Duration, Utc};
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use std::time::SystemTime;

const MAX_RANGE_COUNT: usize = 100;

/// evaluates If-Match precondition (Err = invalid header, ignore this header)
pub(crate) fn if_match(
    etag_header: &HeaderValue,
    resource_etag: &str,
) -> Result<bool, HeaderError> {
    strong_compare(etag_header, resource_etag)
}

/// evaluates If-Unmodified-Since precondition (Err = invalid header, ignore this header)
pub(crate) fn if_unmodified_since(
    header_modified_since: &HeaderValue,
    resource_modified_since: &SystemTime,
) -> Result<bool, HeaderError> {
    header_to_date(header_modified_since)
        .map(|header_date| header_date >= DateTime::<Utc>::from(*resource_modified_since))
}

/// evaluates If-None-Match precondition (Err = invalid header, ignore this header)
pub(crate) fn if_none_match(
    etag_header: &HeaderValue,
    resource_etag: &str,
) -> Result<bool, HeaderError> {
    weak_compare(etag_header, resource_etag).map(|is_match| !is_match)
}

/// evaluates If-Modified-Since precondition (Err = invalid header, ignore this header)
pub(crate) fn if_modified_since(
    header_modified_since: &HeaderValue,
    resource_modified_since: &SystemTime,
) -> Result<bool, HeaderError> {
    header_to_date(header_modified_since)
        .map(|header_date| header_date < DateTime::<Utc>::from(*resource_modified_since))
}

/// evaluates If-Range precondition (Err = invalid header, ignore this header)
pub(crate) fn if_range(
    if_range_header: Option<&HeaderValue>,
    modified_since: &SystemTime,
    etag: &str,
    date_validator: Option<&HeaderValue>,
) -> Result<bool, HeaderError> {
    // check if there is an If-Range header
    if let Some(if_range_some) = if_range_header {
        // check if the if_range condition is a date by attempting parse
        if let Ok(if_range_date) = header_to_date(if_range_some) {
            // check if date header is present
            if let Some(date_val_header) = date_validator {
                // convert date header to date
                if let Ok(date_val) = header_to_date(date_val_header) {
                    let resource_mod_time: DateTime<Utc> = DateTime::from(*modified_since);
                    if (date_val - resource_mod_time) > Duration::seconds(1) {
                        // strong date: see if resource and client if_range condition match
                        Ok(if_range_date == resource_mod_time)
                    } else {
                        // weak date: condition failed
                        Ok(false)
                    }
                } else {
                    // conversion failed: bad format
                    Err(HeaderError::BadFormat)
                }
            } else {
                // if no date header present, the date is weak
                Ok(false)
            }
        } else {
            // assume is etag on parse fail
            strong_compare(if_range_some, etag)
        }
    } else {
        // If if_range_header is None - no condition
        Ok(true)
    }
}

/// returns bytes of the requested range
pub(crate) fn range(
    content: &Bytes,
    range_header: &HeaderValue,
) -> Result<Vec<(Bytes, u64, u64)>, HeaderError> {
    // content length for checking ranges
    let content_length = content.len() as u64;

    // if any of these fail, indicates invalid range
    if let Ok(range_str) = range_header.to_str() {
        if let Some(stripped_str) = range_str.strip_prefix("bytes=") {
            // ignores the "bytes" part
            let range_pairs: Vec<&str> = stripped_str.split(',').collect();
            let mut ranges: Vec<(u64, u64)> = Vec::new();

            // check if max range count exceeded
            if range_pairs.len() > MAX_RANGE_COUNT {
                return Err(HeaderError::BadFormat);
            }

            // get the ranges from the string
            for pair in range_pairs {
                let parts: Vec<&str> = pair.split('-').collect();
                ranges.push(try_get_range(parts, content_length)?);
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
                return Err(HeaderError::BadFormat);
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
                return Err(HeaderError::BadFormat);
            }

            let mut sliced_content: Vec<(Bytes, u64, u64)> = Vec::new();
            // if in ascending order and there is not more than 1 overlap, slice content
            for &(start, end) in ranges.iter() {
                sliced_content.push((slice_with_range(start, end, content)?, start, end))
            }
            Ok(sliced_content)
        } else {
            Err(HeaderError::BadFormat)
        }
    } else {
        Err(HeaderError::BadFormat)
    }
}

/// if range is valid, return range start and end in u64
fn try_get_range(range_vec: Vec<&str>, content_length: u64) -> Result<(u64, u64), HeaderError> {
    match (range_vec[0].is_empty(), range_vec[1].is_empty()) {
        (false, false) => {
            // range x-y
            if let (Ok(start), Ok(end)) = (range_vec[0].parse::<u64>(), range_vec[1].parse::<u64>())
            {
                if start < content_length && start <= end {
                    if end < content_length {
                        Ok((start, end))
                    } else {
                        Err(HeaderError::SuffixExceedsLength)
                    }
                } else {
                    Err(HeaderError::InvalidRange)
                }
            } else {
                Err(HeaderError::BadFormat)
            }
        }
        (false, true) => {
            // range x-
            if let Ok(from_start) = range_vec[0].parse::<u64>() {
                if from_start < content_length {
                    Ok((from_start, content_length - 1))
                } else {
                    Err(HeaderError::InvalidRange)
                }
            } else {
                Err(HeaderError::BadFormat)
            }
        }
        (true, false) => {
            // range -y
            if let Ok(from_end) = range_vec[1].parse::<u64>() {
                if from_end < content_length {
                    Ok(((content_length - 1) - from_end, content_length - 1))
                } else {
                    Ok((0, content_length - 1))
                }
            } else {
                Err(HeaderError::BadFormat)
            }
        }
        _ => Err(HeaderError::BadFormat),
    }
}

/// slices content according to range
fn slice_with_range(start: u64, end: u64, content: &Bytes) -> Result<Bytes, HeaderError> {
    let start_index = usize::try_from(start).map_err(|_| HeaderError::InvalidRange)?;
    let end_index = usize::try_from(end).map_err(|_| HeaderError::InvalidRange)?;
    Ok(content.slice(start_index..end_index + 1))
}

/// returns true if according to http spec the cache can be checked based on request headers
pub(crate) fn can_check_cache(header_value: &HeaderMap) -> bool {
    if header_value.get("If-Match").is_some() || header_value.get("If-Unmodified-Since").is_some() {
        return false;
    }
    true
}

/// converts a HeaderValue to a utc date
fn header_to_date(header_date: &HeaderValue) -> Result<DateTime<Utc>, HeaderError> {
    let header_str = header_date.to_str().map_err(|_| HeaderError::BadFormat)?;
    DateTime::parse_from_rfc2822(header_str)
        .map_err(|_| HeaderError::BadFormat)
        .map(|no_timezone| no_timezone.with_timezone(&Utc))
}

/// does a strong comparison (returns true if there is at least 1 match)
fn strong_compare(etag_header: &HeaderValue, resource_etag: &str) -> Result<bool, HeaderError> {
    if let Ok(etag_str) = etag_header.to_str() {
        // if tag is weak, return None
        if etag_str.starts_with("W/") || resource_etag.starts_with("W/") {
            Err(HeaderError::BadFormat)
        } else {
            // convert split to vector
            let etags: Vec<&str> = etag_str.split(", ").collect();

            compare_etag_vec(etags, resource_etag)
        }
    } else {
        Err(HeaderError::BadFormat)
    }
}

/// does a weak comparison (returns true if there is at least 1 match)
fn weak_compare(etag_header: &HeaderValue, resource_etag: &str) -> Result<bool, HeaderError> {
    if let Ok(etag_str) = etag_header.to_str() {
        let clean_header_etag = etag_str.strip_prefix("W/").unwrap_or(etag_str);
        let clean_resource_etag = resource_etag.strip_prefix("W/").unwrap_or(resource_etag);
        // convert split to vector
        let etags: Vec<&str> = clean_header_etag.split(", ").collect();

        compare_etag_vec(etags, clean_resource_etag)
    } else {
        Err(HeaderError::BadFormat)
    }
}

// compares 2 etag vectors for matches / *
fn compare_etag_vec(header_vec: Vec<&str>, resource_etag: &str) -> Result<bool, HeaderError> {
    if header_vec.len() > 1 && header_vec.contains(&"*") {
        Err(HeaderError::BadFormat)
    } else {
        Ok(header_vec
            .iter()
            .any(|&etag| etag == "*" || etag == resource_etag))
    }
}

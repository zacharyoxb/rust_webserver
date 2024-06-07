use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::HeaderMap;

// evaluates If-Match precondition
pub(crate) fn if_match(header: &HeaderValue) -> bool {
    todo!()
}

// evaluates If-Unmodified-Since precondition
pub(crate) fn if_unmodified_since(header: &HeaderValue) -> bool {
    todo!()
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

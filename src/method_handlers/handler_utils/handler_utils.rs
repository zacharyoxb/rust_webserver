// Standard library imports
use std::io;
use std::io::ErrorKind;
use std::time::SystemTime;
use chrono::{DateTime, Days, Utc};
// External crate imports
use hyper::header::HeaderValue;

// checks if the request is out of date or not, returning true if the page has been modified since
// takes a HeaderValue and a SystemTime value
// TODO: This is a temporary file, these methods should be moved eventually
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
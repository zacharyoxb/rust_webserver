use std::fmt;
use std::fmt::write;

type Result<T> = std::result::Result<T, HeaderError>;

#[derive(Debug, Clone)]
pub enum HeaderError {
    BadFormat,
    ParseError,
    InvalidRange
}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HeaderError::BadFormat => write!(f, "Client packet has bad header format"),
            HeaderError::ParseError => write!(f, "Error in parsing"),
            HeaderError::InvalidRange => write!(f, "Invalid Range supplied")
        }
    }
}
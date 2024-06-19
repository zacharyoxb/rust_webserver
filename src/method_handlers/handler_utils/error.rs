use std::fmt;

#[derive(Debug, Clone)]
pub enum HeaderError {
    BadFormat,
    InvalidRange,
    SuffixExceedsLength,
}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HeaderError::BadFormat => write!(f, "Client packet has bad header format"),
            HeaderError::InvalidRange => write!(f, "Invalid Range(s) supplied"),
            HeaderError::SuffixExceedsLength => write!(
                f,
                "Suffix has exceeded length: entire resource should be sent back."
            ),
        }
    }
}

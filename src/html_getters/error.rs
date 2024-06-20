use std::fmt;

#[derive(Debug, Clone)]
pub struct RetrievalError;

impl fmt::Display for RetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error in retrieving web content")
    }
}

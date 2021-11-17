use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ParseError {
    pub lineno: usize,
    pub message: String,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.lineno, self.message)
    }
}

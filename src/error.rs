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

#[derive(Debug)]
pub struct CLIError(pub String);

impl Error for CLIError {}

impl fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct AssemblerError {
    reason: String,
    line: usize
}

impl AssemblerError {
    pub fn new(reason: String, line: usize) -> Self {
        Self { reason, line }
    }
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if  self.line == 0 {
            write!(f, "{}", self.reason)
        } else {
            write!(f, "[Line {}] {}", self.line, self.reason)
        }
    }
}

impl Error for AssemblerError {}
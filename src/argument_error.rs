use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ArgumentError {
    reason: String
}

impl ArgumentError {
    pub fn new(reason: String) -> Self {
        Self { reason }
    }
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl Error for ArgumentError {}
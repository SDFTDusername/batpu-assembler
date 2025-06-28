use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblerError {
    pub description: String,
    pub line: usize
}

impl AssemblerError {
    pub fn new(description: String) -> Self {
        Self { description, line: 0 }
    }

    pub fn new_line(description: String, line: usize) -> Self {
        Self { description, line }
    }
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.description)
        } else {
            write!(f, "[Line {}] {}", self.line, self.description)
        }
    }
}

impl PartialOrd for AssemblerError {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.line.partial_cmp(&other.line)
    }
}

impl Ord for AssemblerError {
    fn cmp(&self, other: &Self) -> Ordering {
        self.line.cmp(&other.line)
    }
}

impl Error for AssemblerError {}
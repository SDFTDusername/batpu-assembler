use batpu_assembly::assembly_error::AssemblyError;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblerError {
    pub description: String,
    pub line: Option<u32>
}

impl AssemblerError {
    pub fn new(description: String) -> Self {
        Self {
            description,
            line: None
        }
    }

    pub fn new_line(description: String, line: u32) -> Self {
        Self {
            description,
            line: Some(line)
        }
    }

    pub fn from_assembly_error(error: &AssemblyError) -> Self {
        Self {
            description: error.description.clone(),
            line: None
        }
    }

    pub fn from_assembly_error_line(error: &AssemblyError, line: u32) -> Self {
        Self {
            description: error.description.clone(),
            line: Some(line)
        }
    }
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(f, "[Line {}] {}", line, self.description),
            None => write!(f, "{}", self.description)
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
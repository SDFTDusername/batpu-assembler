use crate::assembler::assembler_error::AssemblerError;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum Location {
    Address(usize),
    Label(String)
}

impl Location {
    pub fn get_address(&self, line: usize, labels: &HashMap<String, usize>) -> Result<usize, Box<dyn Error>> {
        match self {
            Location::Address(address) => Ok(*address),
            Location::Label(label) => {
                let result = labels.get(label);
                match result {
                    Some(value) => Ok(*value),
                    None => Err(AssemblerError::new(format!("Unknown label \"{}\"", label), line).into())
                }
            }
        }
    }
}
use crate::assembly::condition::Condition;
use crate::assembly::immediate::Immediate;
use crate::assembly::location::Location;
use crate::assembly::offset::Offset;
use crate::assembly::register::Register;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum Instruction {
    NoOperation,
    Halt,
    Addition(Register, Register, Register),
    Subtraction(Register, Register, Register),
    BitwiseNOR(Register, Register, Register),
    BitwiseAND(Register, Register, Register),
    BitwiseXOR(Register, Register, Register),
    RightShift(Register, Register),
    LoadImmediate(Register, Immediate),
    AddImmediate(Register, Immediate),
    Jump(Location),
    Branch(Condition, Location),
    Call(Location),
    Return,
    MemoryLoad(Register, Register, Offset),
    MemoryStore(Register, Register, Offset)
}

impl Instruction {
    pub fn index(&self) -> u8 {
        match self {
            Instruction::NoOperation            => 0,
            Instruction::Halt                   => 1,
            Instruction::Addition(_, _, _)      => 2,
            Instruction::Subtraction(_, _, _)   => 3,
            Instruction::BitwiseNOR(_, _, _)    => 4,
            Instruction::BitwiseAND(_, _, _)    => 5,
            Instruction::BitwiseXOR(_, _, _)    => 6,
            Instruction::RightShift(_, _)       => 7,
            Instruction::LoadImmediate(_, _)    => 8,
            Instruction::AddImmediate(_, _)     => 9,
            Instruction::Jump(_)                => 10,
            Instruction::Branch(_, _)           => 11,
            Instruction::Call(_)                => 12,
            Instruction::Return                 => 13,
            Instruction::MemoryLoad(_, _, _)    => 14,
            Instruction::MemoryStore(_, _, _)   => 15
        }
    }
    
    pub fn binary(&self, line: usize, labels: &HashMap<String, usize>) -> Result<u16, Box<dyn Error>> {
        let mut binary: u16 = 0;
        binary |= (self.index() as u16 & 0b1111) << 12;

        match self {
            Instruction::Addition(a, b, c) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= (b.register() as u16 & 0b1111) << 4;
                binary |= c.register() as u16 & 0b1111;
            },
            Instruction::Subtraction(a, b, c) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= (b.register() as u16 & 0b1111) << 4;
                binary |= c.register() as u16 & 0b1111;
            },
            Instruction::BitwiseNOR(a, b, c) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= (b.register() as u16 & 0b1111) << 4;
                binary |= c.register() as u16 & 0b1111;
            },
            Instruction::BitwiseAND(a, b, c) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= (b.register() as u16 & 0b1111) << 4;
                binary |= c.register() as u16 & 0b1111;
            },
            Instruction::BitwiseXOR(a, b, c) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= (b.register() as u16 & 0b1111) << 4;
                binary |= c.register() as u16 & 0b1111;
            },
            Instruction::RightShift(a, c) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= c.register() as u16 & 0b1111;
            },
            Instruction::LoadImmediate(a, immediate) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= immediate.immediate() as u16 & 0b1111_1111;
            },
            Instruction::AddImmediate(a, immediate) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= immediate.immediate() as u16 & 0b1111_1111;
            },
            Instruction::Jump(label) => {
                let address = label.get_address(line, labels)?;
                binary |= address as u16 & 0b11_1111_1111;
            },
            Instruction::Branch(condition, label) => {
                binary |= (condition.index() as u16 & 0b11) << 10;

                let address = label.get_address(line, labels)?;
                binary |= address as u16 & 0b11_1111_1111;
            },
            Instruction::Call(label) => {
                let address = label.get_address(line, labels)?;
                binary |= address as u16 & 0b11_1111_1111;
            },
            Instruction::MemoryLoad(a, b, offset) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= (b.register() as u16 & 0b1111) << 4;
                binary |= offset.offset() as u16 & 0b1111;
            },
            Instruction::MemoryStore(a, b, offset) => {
                binary |= (a.register() as u16 & 0b1111) << 8;
                binary |= (b.register() as u16 & 0b1111) << 4;
                binary |= offset.offset() as u16 & 0b1111;
            },
            _ => {}
        };
        
        Ok(binary)
    }
}
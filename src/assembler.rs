use crate::assembler_config::AssemblerConfig;
use crate::assembler_error::AssemblerError;
use batpu_assembly::components::address;
use batpu_assembly::components::address::Address;
use batpu_assembly::components::condition::Condition;
use batpu_assembly::components::immediate::Immediate;
use batpu_assembly::components::location::Location;
use batpu_assembly::components::offset::Offset;
use batpu_assembly::components::register::Register;
use batpu_assembly::instruction::Instruction;
use batpu_assembly::Labels;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::iter::Iterator;

const CHARACTERS: &[char] = &[' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '.', '!', '?'];

pub struct Assembler {
    pub config: AssemblerConfig,
    
    instructions: Vec<(Instruction, u32)>,
    labels: Labels,
    defines: HashMap<String, String>,

    line: u32
}

impl Assembler {
    pub fn new(config: AssemblerConfig) -> Self {
        let mut defines = HashMap::new();

        if config.default_defines {
            // Screen

            defines.insert("SCR_PIX_X".to_string(), "240".to_string());
            defines.insert("SCR_PIX_Y".to_string(), "241".to_string());

            defines.insert("SCR_DRAW_PIX".to_string(), "242".to_string());
            defines.insert("SCR_CLR_PIX".to_string(), "243".to_string());
            defines.insert("SCR_GET_PIX".to_string(), "244".to_string());

            defines.insert("SCR_PUSH".to_string(), "245".to_string());
            defines.insert("SCR_CLR".to_string(), "246".to_string());

            // Character Display

            defines.insert("CHAR_DISP_PUSH".to_string(), "247".to_string());

            defines.insert("CHAR_DISP_DRAW".to_string(), "248".to_string());
            defines.insert("CHAR_DISP_CLR".to_string(), "249".to_string());

            // Number Display

            defines.insert("NUM_DISP_SHOW".to_string(), "250".to_string());
            defines.insert("NUM_DISP_CLR".to_string(), "251".to_string());

            defines.insert("NUM_DISP_SIGNED".to_string(), "252".to_string());
            defines.insert("NUM_DISP_UNSIGNED".to_string(), "253".to_string());

            // Random Number Generator
            defines.insert("RNG".to_string(), "254".to_string());

            // Controller
            defines.insert("CONTROLLER".to_string(), "255".to_string());
        }

        Self {
            config,
            
            instructions: Vec::new(),
            labels: HashMap::new(),
            defines,

            line: 0
        }
    }

    fn check_arguments(&self, mut actual_len: usize, expected: &[&str]) -> Result<(), AssemblerError> {
        actual_len -= 1;
        
        if actual_len != expected.len() {
            return Err(AssemblerError::new_line(format!(
                "Expected {}, got {} instead",
                if expected.len() == 0 {
                    "no arguments".to_string()
                } else {
                    format!(
                        "{} ({} argument{})",
                        Self::join_with_and(expected),
                        expected.len(),
                        if expected.len() == 1 { "" } else { "s" }
                    )
                },
                actual_len
            ), self.line));
        }

        Ok(())
    }

    fn parse_piece(&mut self, piece: &str) -> Result<Option<Instruction>, Box<dyn Error>> {
        let mut args: Vec<&str> = piece
            .split_whitespace()
            .collect();

        let name = args[0];

        if name.ends_with(':') {
            self.check_arguments(args.len(), &[])?;

            let label_name = name[..name.len() - 1].to_string();

            if self.labels.contains_key(&label_name) {
                return Err(AssemblerError::new_line(format!("Label \"{}\" was already defined", label_name), self.line).into());
            }

            self.labels.insert(label_name, Immediate::new(self.instructions.len() as u32));
            return Ok(None);
        }

        if name.eq("#define") {
            self.check_arguments(args.len(), &["Name", "Value"])?;

            let define_name = args[1];

            if self.defines.contains_key(define_name) {
                return Err(AssemblerError::new_line(format!("Definition of \"{}\" already exists", define_name), self.line).into());
            }

            let define_value = args[2];

            self.defines.insert(define_name.to_string(), define_value.to_string());
            return Ok(None);
        }

        for i in 0..args.len() {
            let result = self.defines.get(args[i]);
            if let Some(definition) = result {
                args[i] = definition;
            }
        }

        let instruction = match name {
            "nop" => {
                self.check_arguments(args.len(), &[])?;
                Instruction::NoOperation
            },
            "hlt" => {
                self.check_arguments(args.len(), &[])?;
                Instruction::Halt
            },
            "add" => {
                self.check_arguments(args.len(), &["RegA", "RegB", "RegC"])?;
                Instruction::Addition(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "sub" => {
                self.check_arguments(args.len(), &["RegA", "RegB", "RegC"])?;
                Instruction::Subtraction(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "nor" => {
                self.check_arguments(args.len(), &["RegA", "RegB", "RegC"])?;
                Instruction::BitwiseNOR(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "and" => {
                self.check_arguments(args.len(), &["RegA", "RegB", "RegC"])?;
                Instruction::BitwiseAND(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "xor" => {
                self.check_arguments(args.len(), &["RegA", "RegB", "RegC"])?;
                Instruction::BitwiseXOR(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "rsh" => {
                self.check_arguments(args.len(), &["RegA", "RegC"])?;
                Instruction::RightShift(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?
                )
            },
            "ldi" => {
                self.check_arguments(args.len(), &["RegA", "Immediate"])?;
                Instruction::LoadImmediate(
                    self.get_register(args[1])?,
                    self.get_immediate(args[2])?
                )
            },
            "adi" => {
                self.check_arguments(args.len(), &["RegA", "Immediate"])?;
                Instruction::AddImmediate(
                    self.get_register(args[1])?,
                    self.get_immediate(args[2])?
                )
            },
            "jmp" => {
                self.check_arguments(args.len(), &["Label/Address"])?;
                Instruction::Jump(
                    self.get_location(args[1])?
                )
            },
            "brh" => {
                self.check_arguments(args.len(), &["Condition", "Label/Address"])?;
                Instruction::Branch(
                    self.get_condition(args[1])?,
                    self.get_location(args[2])?
                )
            },
            "cal" => {
                self.check_arguments(args.len(), &["Label/Address"])?;
                Instruction::Call(
                    self.get_location(args[1])?
                )
            },
            "ret" => {
                self.check_arguments(args.len(), &[])?;
                Instruction::Return
            },
            "lod" => {
                self.check_arguments(args.len(), &["RegA", "RegB", "Offset"])?;
                Instruction::MemoryLoad(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_offset(args[3])?
                )
            },
            "str" => {
                self.check_arguments(args.len(), &["RegA", "RegB", "Offset"])?;
                Instruction::MemoryStore(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_offset(args[3])?
                )
            },
            "cmp" => {
                self.check_arguments(args.len(), &["RegA", "RegB"])?;
                Instruction::Subtraction(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    Register::new(0)?
                )
            },
            "mov" => {
                self.check_arguments(args.len(), &["RegA", "RegC"])?;
                Instruction::Addition(
                    self.get_register(args[1])?,
                    Register::new(0)?,
                    self.get_register(args[2])?
                )
            },
            "lsh" => {
                self.check_arguments(args.len(), &["RegA", "RegC"])?;
                let a = self.get_register(args[1])?;
                Instruction::Addition(
                    a,
                    a,
                    self.get_register(args[2])?
                )
            },
            "inc" => {
                self.check_arguments(args.len(), &["RegA"])?;
                Instruction::AddImmediate(
                    self.get_register(args[1])?,
                    Immediate::new(1)
                )
            },
            "dec" => {
                self.check_arguments(args.len(), &["RegA"])?;
                Instruction::AddImmediate(
                    self.get_register(args[1])?,
                    Immediate::new_signed(-1)
                )
            },
            "not" => {
                self.check_arguments(args.len(), &["RegA", "RegC"])?;
                Instruction::BitwiseNOR(
                    self.get_register(args[1])?,
                    Register::new(0)?,
                    self.get_register(args[2])?
                )
            },
            "neg" => {
                self.check_arguments(args.len(), &["RegA", "RegC"])?;
                Instruction::Subtraction(
                    Register::new(0)?,
                    self.get_register(args[1])?,
                    self.get_register(args[2])?
                )
            },
            _ => {
                return Err(AssemblerError::new_line(format!("Unknown opcode: {}", name), self.line).into());
            }
        };

        Ok(Some(instruction))
    }

    fn parse_line(&mut self, mut line: &str) -> Result<Vec<(Instruction, u32)>, Vec<Box<dyn Error>>> {
        let mut errors = Vec::new();
        let mut instructions = Vec::new();

        let comment_index = line.find("//");

        match comment_index {
            Some(index) => {
                line = &line[..index];
            }
            None => {}
        }

        line = line.trim();

        if line.is_empty() {
            return Ok(instructions);
        }

        let pieces: Vec<&str> = line
            .split(';')
            .map(|piece| piece.trim())
            .collect();

        for piece in pieces {
            if piece.is_empty() {
                errors.push(AssemblerError::new_line("Useless semicolon".to_string(), self.line).into());
                continue;
            }
            
            let result = self.parse_piece(piece);
            match result {
                Ok(instruction) => {
                    if let Some(instruction) = instruction {
                        instructions.push((instruction, self.line));
                    }
                },
                Err(error) => {
                    errors.push(error);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(instructions)
    }

    pub fn parse(&mut self, input: &str) -> Result<(), Vec<Box<dyn Error>>> {
        let mut errors: Vec<Box<dyn Error>> = Vec::new();

        for (i, line) in input.lines().into_iter().enumerate() {
            self.line = i as u32 + 1;
            
            let result = self.parse_line(line);

            match result {
                Ok(mut result) => {
                    self.instructions.append(&mut result);
                },
                Err(mut parse_errors) => {
                    errors.append(&mut parse_errors);
                }
            }
        }

        if self.instructions.len() > address::MAX_VALUE as usize {
            errors.push(AssemblerError::new(format!("Program reached maximum size ({} instructions)", address::MAX_POSSIBLE_COUNT)).into());
            return Err(errors);
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }

    pub fn parse_file(&mut self, path: &str) -> Result<(), Vec<Box<dyn Error>>> {
        let result = fs::read_to_string(path);

        match result {
            Ok(file) => self.parse(file.as_str()),
            Err(error) => Err(vec![Box::new(error)])
        }
    }

    pub fn assemble(&self) -> Result<Vec<u16>, Vec<AssemblerError>> {
        let mut errors: Vec<AssemblerError> = Vec::new();

        let binary = self.instructions
            .iter()
            .enumerate()
            .map(|(address, (instruction, line))| {
                let result = instruction.binary(address as u32, &self.labels);
                match result {
                    Ok(binary) => binary,
                    Err(error) => {
                        errors.push(AssemblerError::from_assembly_error_line(&error, *line));
                        0
                    }
                }
            })
            .collect();

        if !errors.is_empty() {
            return Err(errors);
        }

        if self.config.print_info {
            println!(
                "{} out of {} instructions used ({:.1}%)",
                self.instructions.len(),
                address::MAX_POSSIBLE_COUNT,
                self.instructions.len() as f32 * 100.0 / address::MAX_POSSIBLE_COUNT as f32
            );
        }
        
        Ok(binary)
    }
    
    pub fn assemble_to_file(&mut self, path: &str) -> Result<(), Vec<Box<dyn Error>>> {
        let assemble_result = self.assemble();
        match assemble_result {
            Ok(machine_code) => {
                let file_result = File::create(path);
                match file_result {
                    Ok(file) => {
                        let mut output_writer = BufWriter::new(file);

                        if self.config.text_output {
                            for (i, &instruction) in machine_code.iter().enumerate() {
                                let line = format!("{:016b}", instruction);

                                let instruction_write = output_writer.write_all(line.as_bytes());
                                if let Err(error) = instruction_write {
                                    return Err(vec![error.into()]);
                                }

                                if i < machine_code.len() - 1 {
                                    let line_write = output_writer.write_all(&[b'\n']);
                                    if let Err(error) = line_write {
                                        return Err(vec![error.into()]);
                                    }
                                }
                            }
                        } else {
                            for &instruction in &machine_code {
                                let bytes = instruction.to_be_bytes();

                                let instruction_write = output_writer.write_all(&bytes);
                                if let Err(error) = instruction_write {
                                    return Err(vec![error.into()]);
                                }
                            }
                        }

                        Ok(())
                    },
                    Err(error) => {
                        Err(vec![error.into()])
                    }
                }
            },
            Err(errors) => {
                let errors = errors
                    .iter()
                    .map(|error| error.clone().into())
                    .collect();
                
                Err(errors)
            }
        }
    }

    fn parse_u32(str: &str) -> Result<u32, Box<dyn Error>> {
        let str = str.replace('_', "");

        if str.starts_with("0x") {
            Ok(u32::from_str_radix(&str[2..], 16)?)
        } else if str.starts_with("0b") {
            Ok(u32::from_str_radix(&str[2..], 2)?)
        } else {
            Ok(str.parse()?)
        }
    }

    fn parse_i32(str: &str) -> Result<i32, Box<dyn Error>> {
        let str = str.replace('_', "");

        if str.starts_with("0x") {
            Ok(i32::from_str_radix(&str[2..], 16)?)
        } else if str.starts_with("0b") {
            Ok(i32::from_str_radix(&str[2..], 2)?)
        } else {
            Ok(str.parse()?)
        }
    }

    fn get_register(&self, register: &str) -> Result<Register, Box<dyn Error>> {
        if !register.starts_with('r') {
            return Err(AssemblerError::new_line(format!("Register \"{}\" must start with a lowercase 'r'", register), self.line).into());
        }

        let register = &register[1..];
        let result = register.parse::<u32>();

        match result {
            Ok(num) => {
                let result = Register::new(num);
                match result {
                    Ok(register) => Ok(register),
                    Err(error) => {
                        Err(AssemblerError::from_assembly_error_line(&error, self.line).into())
                    }
                }
            },
            Err(error) => {
                Err(AssemblerError::new_line(format!("Failed to parse register \"{}\": {}", register, error), self.line).into())
            }
        }
    }

    fn get_immediate(&self, immediate: &str) -> Result<Immediate, Box<dyn Error>> {
        if immediate.starts_with("'") {
            if !immediate.ends_with("'") {
                return Err(AssemblerError::new_line(format!("Immediate \"{}\" must end with ''", immediate), self.line).into());
            }

            let immediate = &immediate[1..immediate.len() - 1];

            if immediate.len() != 1 {
                return Err(AssemblerError::new_line(format!("Immediate \"{}\" must only contain a single character", immediate), self.line).into());
            }

            let char = immediate.chars().next().unwrap();
            let char_index = CHARACTERS.iter().position(|&c| c == char);

            return match char_index {
                Some(index) => {
                    Ok(Immediate::new(index as u32))
                }
                None => {
                    Err(AssemblerError::new_line(format!("Character \"{}\" is not supported, you can only use ones in \"{}\"", char, CHARACTERS.iter().collect::<String>()), self.line).into())
                }
            }
        }

        let result = Self::parse_i32(immediate);

        match result {
            Ok(num) => Ok(Immediate::new_signed(num)),
            Err(error) => {
                Err(AssemblerError::new_line(format!("Failed to parse immediate \"{}\": {}", immediate, error), self.line).into())
            }
        }
    }

    fn get_location(&self, location: &str) -> Result<Location, Box<dyn Error>> {
        let add = location.starts_with('+');
        let sub = location.starts_with('-');

        if add || sub {
            let result = Self::parse_u32(&location[1..]);
            return match result {
                Ok(num) => {
                    let num = if add {
                        num as i32
                    } else if sub {
                        -(num as i32)
                    } else {
                        return Err(AssemblerError::new_line(format!("Unknown location \"{}\"", location),  self.line).into());
                    };
                    
                    Ok(Location::Offset(Offset::new(num)?))
                },
                Err(error) => {
                    Err(AssemblerError::new_line(format!("Failed to parse address offset \"{}\": {}", location, error), self.line).into())
                }
            }
        }

        let result = Self::parse_u32(location);
        match result {
            Ok(num) => {
                let result = Address::new(num);
                match result {
                    Ok(address) => Ok(Location::Address(address)),
                    Err(error) => {
                        Err(AssemblerError::from_assembly_error_line(&error, self.line).into())
                    }
                }
            }
            Err(_) => {
                Ok(Location::Label(location.to_string()))
            }
        }
    }

    fn get_condition(&self, condition: &str) -> Result<Condition, Box<dyn Error>> {
        match condition {
            "zero"     =>  Ok(Condition::Zero),
            "notzero"  =>  Ok(Condition::NotZero),
            "carry"    =>  Ok(Condition::Carry),
            "notcarry" =>  Ok(Condition::NotCarry),
            _ => Err(AssemblerError::new_line(format!("Unknown condition: \"{}\"", condition), self.line).into())
        }
    }

    fn get_offset(&self, offset: &str) -> Result<Offset, Box<dyn Error>> {
        let result = Self::parse_i32(offset);
        match result {
            Ok(num) => {
                let result = Offset::new(num);
                match result {
                    Ok(offset) => Ok(offset),
                    Err(error) => {
                        Err(AssemblerError::from_assembly_error_line(&error, self.line).into())
                    }
                }
            },
            Err(error) => {
                Err(AssemblerError::new_line(format!("Failed to parse offset \"{}\": {}", offset, error), self.line).into())
            }
        }
    }
    
    fn join_with_and(items: &[&str]) -> String {
        match items.len() {
            0 => String::new(),
            1 => items[0].to_string(),
            2 => format!("{} and {}", items[0], items[1]),
            _ => {
                let all_but_last = &items[..items.len() - 1];
                let last = items[items.len() - 1];
                
                format!("{} and {}", all_but_last.join(", "), last)
            }
        }
    }
}
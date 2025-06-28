use crate::assembler::assembler_config::AssemblerConfig;
use crate::assembler::assembler_error::AssemblerError;
use crate::assembly::condition::Condition;
use crate::assembly::immediate::Immediate;
use crate::assembly::instruction::Instruction;
use crate::assembly::location::Location;
use crate::assembly::location::Location::{Address, Label};
use crate::assembly::offset::Offset;
use crate::assembly::register::Register;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::iter::Iterator;

const CHARACTERS: &[char] = &[' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '.', '!', '?'];

pub struct Assembler {
    pub config: AssemblerConfig,
    
    instructions: Vec<Instruction>,
    labels: HashMap<String, usize>,
    defines: HashMap<String, String>,

    line: usize
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
            defines.insert("SCR_LOAD_PIX".to_string(), "244".to_string());

            defines.insert("SCR_DRAW".to_string(), "245".to_string());
            defines.insert("SCR_CLR".to_string(), "246".to_string());

            // Character Display

            defines.insert("CHAR_DISP_WRITE".to_string(), "247".to_string());

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
    
    pub fn parse_line(&mut self, mut line: &str) -> Result<Option<Instruction>, Box<dyn Error>> {
        let comment_index = line.find("//");

        match comment_index {
            Some(index) => {
                line = &line[..index];
            }
            None => {}
        }

        line = line.trim();

        if line.is_empty() {
            return Ok(None);
        }

        let mut args: Vec<&str> = line
            .split_whitespace()
            .collect();

        let name = args[0];

        if name.ends_with(':') {
            let label_name = name[..name.len() - 1].to_string();

            if self.labels.contains_key(&label_name) {
                return Err(AssemblerError::new(format!("Label \"{}\" was already defined", label_name), self.line).into());
            }

            self.labels.insert(label_name, self.instructions.len());
            return Ok(None);
        }

        if name.eq("#define") {
            assert_eq!(args.len(), 3, "Expected name and value for define, got {}", args.len() - 1);

            let define_name = args[1];

            if self.defines.contains_key(define_name) {
                return Err(AssemblerError::new(format!("Definition of \"{}\" already exists", define_name), self.line).into());
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
                Instruction::NoOperation
            },
            "hlt" => {
                Instruction::Halt
            },
            "add" => {
                assert_eq!(args.len(), 4, "Expected 3 arguments, got {}", args.len() - 1);
                Instruction::Addition(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "sub" => {
                assert_eq!(args.len(), 4, "Expected 3 arguments, got {}", args.len() - 1);
                Instruction::Subtraction(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "nor" => {
                assert_eq!(args.len(), 4, "Expected 3 arguments, got {}", args.len() - 1);
                Instruction::BitwiseNOR(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "and" => {
                assert_eq!(args.len(), 4, "Expected 3 arguments, got {}", args.len() - 1);
                Instruction::BitwiseAND(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "xor" => {
                assert_eq!(args.len(), 4, "Expected 3 arguments, got {}", args.len() - 1);
                Instruction::BitwiseXOR(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_register(args[3])?
                )
            },
            "rsh" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                Instruction::RightShift(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?
                )
            },
            "ldi" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                Instruction::LoadImmediate(
                    self.get_register(args[1])?,
                    self.get_immediate(args[2])?
                )
            },
            "adi" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                Instruction::AddImmediate(
                    self.get_register(args[1])?,
                    self.get_immediate(args[2])?
                )
            },
            "jmp" => {
                assert_eq!(args.len(), 2, "Expected 1 argument, got {}", args.len() - 1);
                Instruction::Jump(
                    self.get_location(args[1])?
                )
            },
            "brh" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                Instruction::Branch(
                    self.get_condition(args[1])?,
                    self.get_location(args[2])?
                )
            },
            "cal" => {
                assert_eq!(args.len(), 2, "Expected 1 argument, got {}", args.len() - 1);
                Instruction::Call(
                    self.get_location(args[1])?
                )
            },
            "ret" => {
                assert_eq!(args.len(), 1, "Expected 0 arguments, got {}", args.len() - 1);
                Instruction::Return
            },
            "lod" => {
                assert_eq!(args.len(), 4, "Expected 3 arguments, got {}", args.len() - 1);
                Instruction::MemoryLoad(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_offset(args[3])?
                )
            },
            "str" => {
                assert_eq!(args.len(), 4, "Expected 3 arguments, got {}", args.len() - 1);
                Instruction::MemoryStore(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    self.get_offset(args[3])?
                )
            },
            "cmp" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                Instruction::Subtraction(
                    self.get_register(args[1])?,
                    self.get_register(args[2])?,
                    Register::new(0)
                )
            },
            "mov" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                Instruction::Addition(
                    self.get_register(args[1])?,
                    Register::new(0),
                    self.get_register(args[2])?
                )
            },
            "lsh" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                let a = self.get_register(args[1])?;
                Instruction::Addition(
                    a,
                    a,
                    self.get_register(args[2])?
                )
            },
            "inc" => {
                assert_eq!(args.len(), 2, "Expected 1 argument, got {}", args.len() - 1);
                Instruction::AddImmediate(
                    self.get_register(args[1])?,
                    Immediate::new(1)
                )
            },
            "dec" => {
                assert_eq!(args.len(), 2, "Expected 1 argument, got {}", args.len() - 1);
                Instruction::AddImmediate(
                    self.get_register(args[1])?,
                    Immediate::new_signed(-1)
                )
            },
            "not" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                Instruction::BitwiseNOR(
                    self.get_register(args[1])?,
                    Register::new(0),
                    self.get_register(args[2])?
                )
            },
            "neg" => {
                assert_eq!(args.len(), 3, "Expected 2 arguments, got {}", args.len() - 1);
                Instruction::Subtraction(
                    Register::new(0),
                    self.get_register(args[1])?,
                    self.get_register(args[2])?
                )
            },
            _ => {
                return Err(AssemblerError::new(format!("Unknown opcode: {}", name), self.line).into());
            }
        };

        Ok(Some(instruction))
    }

    pub fn parse(&mut self, input: &str) -> Result<(), Vec<Box<dyn Error>>> {
        let mut errors: Vec<Box<dyn Error>> = Vec::new();
        
        for (i, line) in input.lines().into_iter().enumerate() {
            self.line = i + 1;
            
            let result = self.parse_line(line);
            
            match result {
                Ok(result) => {
                    if let Some(instruction) = result {
                        self.instructions.push(instruction);
                    }
                },
                Err(error) => {
                    errors.push(error);
                }
            }
        }

        if self.instructions.len() > 4095 {
            errors.push(AssemblerError::new("Program reached maximum size (4096 instructions)".to_string(), 0).into());
            return Err(errors);
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        if self.config.print_info {
            println!("{} out of 4096 instructions used ({:.1}%)", self.instructions.len(), self.instructions.len() as f64 * 100.0 / 4096.0);
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

    pub fn assemble(&self) -> Result<Vec<u16>, Vec<Box<dyn Error>>> {
        let mut errors: Vec<Box<dyn Error>> = Vec::new();
        
        let binary = self.instructions
            .iter()
            .enumerate()
            .map(|(i, instruction)| {
                let result = instruction.binary(i + 1, &self.labels);
                match result {
                    Ok(binary) => binary,
                    Err(error) => {
                        errors.push(error);
                        0
                    }
                }
            })
            .collect();
        
        if !errors.is_empty() {
            return Err(errors);
        }
        
        Ok(binary)
    }
    
    pub fn assemble_to_file(&mut self, path: &str) -> Result<(), Vec<Box<dyn Error>>> {
        let machine_code = self.assemble()?;

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
    }

    fn parse_usize(str: &str) -> Result<usize, Box<dyn Error>> {
        let str = str.replace('_', "");

        if str.starts_with("0x") {
            Ok(usize::from_str_radix(&str[2..], 16)?)
        } else if str.starts_with("0b") {
            Ok(usize::from_str_radix(&str[2..], 2)?)
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
            return Err(AssemblerError::new(format!("Register \"{}\" must start with a lowercase 'r'", register), self.line).into());
        }

        let register = &register[1..];
        let result = register.parse::<u8>();

        match result {
            Ok(num) => {
                if num > 15 {
                    return Err(AssemblerError::new(format!("Register {} out of range, expected 0-15", register), self.line).into());
                }

                Ok(Register::new(num))
            },
            Err(error) => {
                Err(AssemblerError::new(format!("Failed to parse register \"{}\": {}", register, error), self.line).into())
            }
        }
    }

    fn get_immediate(&self, immediate: &str) -> Result<Immediate, Box<dyn Error>> {
        if immediate.starts_with("'") {
            if !immediate.ends_with("'") {
                return Err(AssemblerError::new(format!("Immediate \"{}\" must end with ''", immediate), self.line).into());
            }

            let immediate = &immediate[1..immediate.len() - 1];

            if immediate.len() != 1 {
                return Err(AssemblerError::new(format!("Immediate \"{}\" must only contain a single character", immediate), self.line).into());
            }

            let char = immediate.chars().next().unwrap();
            let char_index = CHARACTERS.iter().position(|&c| c == char);

            return match char_index {
                Some(index) => {
                    Ok(Immediate::new(index as u8))
                }
                None => {
                    Err(AssemblerError::new(format!("Character \"{}\" is not supported, you can only use ones in \"{}\"", char, CHARACTERS.iter().collect::<String>()), self.line).into())
                }
            }
        }

        let result = Self::parse_i32(immediate);

        match result {
            Ok(num) => {
                if num < -128 || num > 255 {
                    return Err(AssemblerError::new(format!("Immediate {} out of range, expected -128-255", immediate), self.line).into());
                }
                
                Ok(Immediate::new_signed(num as i16))
            },
            Err(error) => {
                Err(AssemblerError::new(format!("Failed to parse immediate \"{}\": {}", immediate, error), self.line).into())
            }
        }
    }

    fn get_location(&self, location: &str) -> Result<Location, Box<dyn Error>> {
        let result = Self::parse_usize(location);
        match result {
            Ok(num) => {
                if num > 4095 {
                    return Err(AssemblerError::new(format!("Address {} out of range, expected 0-4095", num), self.line).into());
                }

                Ok(Address(num))
            }
            Err(_) => {
                Ok(Label(location.to_string()))
            }
        }
    }

    fn get_condition(&self, condition: &str) -> Result<Condition, Box<dyn Error>> {
        match condition {
            "zero"     =>  Ok(Condition::Zero),
            "notzero"  =>  Ok(Condition::NotZero),
            "carry"    =>  Ok(Condition::Carry),
            "notcarry" =>  Ok(Condition::NotCarry),
            _ => Err(AssemblerError::new(format!("Unknown condition: \"{}\"", condition), self.line).into())
        }
    }

    fn get_offset(&self, offset: &str) -> Result<Offset, Box<dyn Error>> {
        let result = Self::parse_i32(offset);
        match result {
            Ok(num) => {
                if num < -8 || num > 7 {
                    return Err(AssemblerError::new(format!("Offset {} out of range, expected -8-7", offset), self.line).into());
                }

                Ok(Offset::new(num as i8))
            },
            Err(error) => {
                Err(AssemblerError::new(format!("Failed to parse offset \"{}\": {}", offset, error), self.line).into())
            }
        }
    }
}
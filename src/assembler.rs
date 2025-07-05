use crate::assembler_config::AssemblerConfig;
use crate::assembler_error::AssemblerError;
use batpu_assembly::assembly_error::AssemblyError;
use batpu_assembly::components::address::Address;
use batpu_assembly::components::condition::Condition;
use batpu_assembly::components::immediate::Immediate;
use batpu_assembly::components::location::Location;
use batpu_assembly::components::offset::Offset;
use batpu_assembly::components::register::Register;
use batpu_assembly::instruction::Instruction;
use batpu_assembly::{instructions_to_binary, InstructionVec, Labels};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::iter::Iterator;

const CHARACTERS: &[char] = &[' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '.', '!', '?'];

pub struct Assembler {
    pub config: AssemblerConfig,
    
    instructions: InstructionVec,
    labels: Labels,
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

            self.labels.insert(label_name, self.instructions.len());
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
                    Register::new(0)
                )
            },
            "mov" => {
                self.check_arguments(args.len(), &["RegA", "RegC"])?;
                Instruction::Addition(
                    self.get_register(args[1])?,
                    Register::new(0),
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
                    Register::new(0),
                    self.get_register(args[2])?
                )
            },
            "neg" => {
                self.check_arguments(args.len(), &["RegA", "RegC"])?;
                Instruction::Subtraction(
                    Register::new(0),
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

    fn parse_line(&mut self, mut line: &str) -> Result<Vec<Instruction>, Vec<Box<dyn Error>>> {
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

        if line.ends_with(';') {
            errors.push(AssemblerError::new_line("Semicolons at the end of lines are useless".to_string(), self.line).into());
        }

        let pieces: Vec<&str> = line
            .split(';')
            .filter(|piece| !piece.trim().is_empty())
            .collect();

        for piece in pieces {
            let result = self.parse_piece(piece);
            match result {
                Ok(instruction) => {
                    if let Some(instruction) = instruction {
                        instructions.push(instruction);
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
            self.line = i + 1;
            
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

        if self.instructions.len() > 1023 {
            errors.push(AssemblerError::new("Program reached maximum size (1024 instructions)".to_string()).into());
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

    pub fn assemble(&self) -> Result<Vec<u16>, Vec<AssemblyError>> {
        let result = instructions_to_binary(&self.instructions, &self.labels);
        
        if result.is_err() {
            return result;
        }

        if self.config.print_info {
            println!(
                "{} out of 1024 instructions used ({:.1}%)",
                self.instructions.len(),
                self.instructions.len() as f64 * 100.0 / 1024.0
            );
        }
        
        result
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
            return Err(AssemblerError::new_line(format!("Register \"{}\" must start with a lowercase 'r'", register), self.line).into());
        }

        let register = &register[1..];
        let result = register.parse::<u8>();

        match result {
            Ok(num) => {
                if num > 15 {
                    return Err(AssemblerError::new_line(format!("Register {} out of range, expected 0-15", register), self.line).into());
                }

                Ok(Register::new(num))
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
                    Ok(Immediate::new(index as u8))
                }
                None => {
                    Err(AssemblerError::new_line(format!("Character \"{}\" is not supported, you can only use ones in \"{}\"", char, CHARACTERS.iter().collect::<String>()), self.line).into())
                }
            }
        }

        let result = Self::parse_i32(immediate);

        match result {
            Ok(num) => {
                if num < -128 || num > 255 {
                    return Err(AssemblerError::new_line(format!("Immediate {} out of range, expected -128-255", immediate), self.line).into());
                }
                
                Ok(Immediate::new_signed(num as i16))
            },
            Err(error) => {
                Err(AssemblerError::new_line(format!("Failed to parse immediate \"{}\": {}", immediate, error), self.line).into())
            }
        }
    }

    fn get_location(&self, location: &str) -> Result<Location, Box<dyn Error>> {
        let add = location.starts_with('+');
        let sub = location.starts_with('-');

        if add || sub {
            let result = Self::parse_usize(&location[1..]);
            return match result {
                Ok(num) => {
                    if add {
                        Ok(Location::Offset(num as isize))
                    } else if sub {
                        Ok(Location::Offset(-(num as isize)))
                    } else {
                        panic!("Unknown location \"{}\"", location);
                    }
                },
                Err(error) => {
                    Err(AssemblerError::new_line(format!("Failed to parse address offset \"{}\": {}", location, error), self.line).into())
                }
            }
        }

        let result = Self::parse_usize(location);
        match result {
            Ok(num) => {
                if num > 1023 {
                    return Err(AssemblerError::new_line(format!("Address {} out of range, expected 0-1023", num), self.line).into());
                }

                Ok(Location::Address(Address::new(num as u16)))
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
                if num < -8 || num > 7 {
                    return Err(AssemblerError::new_line(format!("Offset {} out of range, expected -8-7", offset), self.line).into());
                }

                Ok(Offset::new(num as i8))
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
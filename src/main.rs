mod argument_error;

use crate::argument_error::ArgumentError;
use batpu_assembler::assembler::assembler::Assembler;
use batpu_assembler::assembler::assembler_config::AssemblerConfig;
use std::env;
use std::error::Error;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut config = AssemblerConfig::default();
    config.print_info = true;
    
    let args: Vec<String> = env::args().collect();

    let mut values: Vec<&str> = Vec::new();
    let mut options: Vec<&str> = Vec::new();

    let mut arg_errors: Vec<Box<dyn Error>> = Vec::new();
    let mut help = false;

    for arg in args.iter().skip(1) {
        if !arg.starts_with("-") {
            values.push(arg);
            continue;
        }

        if options.contains(&arg.as_str()) {
            arg_errors.push(ArgumentError::new(format!("Option \"{}\" was already specified", arg)).into());
            continue;
        }
        
        match arg.as_str() {
            "-d" | "--no-default-defines" => {
                config.default_defines = false;
            },
            "-p" | "--no-print-info" => {
                config.print_info = false;
            },
            "-t" | "--text-output" => {
                config.text_output = true;
            },
            "-h" |  "--help" => {
                help = true;
            }
            _ => {
                arg_errors.push(ArgumentError::new(format!("Unknown option \"{}\"", arg)).into());
                continue;
            }
        }
        
        options.push(arg);
    }

    if !arg_errors.is_empty() {
        for error in arg_errors {
            eprintln!("{}", error);
        }
        
        return ExitCode::FAILURE;
    }
    
    if help || values.is_empty() {
        println!("Usage: batpu-assembler [INPUT] [OUTPUT]
-d, --disable-default-defines - Disables built-in defines, such as SCR_PIX_X
-p, --no-print-info           - Do not print assembler info
-t, --text-output             - Assemble to text file with binary representation");
        return ExitCode::SUCCESS;
    }
    
    if values.len() != 2 {
        eprintln!("Expected input and output files, got {} value(s)", values.len());
        return ExitCode::FAILURE;
    }
    
    let input_path = &values[0];
    let output_path = &values[1];

    let mut assembler = Assembler::new(config);
    
    let parse_result = assembler.parse_file(input_path);
    if let Err(errors) = parse_result {
        eprintln!("Failed to parse \"{}\", {} error(s):", input_path, errors.len());
        for error in errors {
            eprintln!("{}", error);
        }
        
        return ExitCode::FAILURE;
    }

    let assemble_result = assembler.assemble_to_file(output_path);
    if let Err(errors) = assemble_result {
        eprintln!("Failed to assemble \"{}\", {} error(s):", input_path, errors.len());
        for error in errors {
            eprintln!("{}", error);
        }

        return ExitCode::FAILURE;
    }

    if config.print_info {
        println!("Assembled \"{}\" to \"{}\"", input_path, output_path);
    }
    
    ExitCode::SUCCESS
}
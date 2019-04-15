use std::fs::File;
use std::io::{BufRead, BufReader, Write};
mod compile;
mod op_codes;
mod parser;
mod token;
use compile::compile;
use parser::*;
use token::*;
extern crate nom;

fn main() {
    // Handle program arguments
    let mut args = std::env::args();
    let program_name = args.next().unwrap();
    if args.len() < 2 {
        println!("Usage: {} <assembly.s> <output.bs>", program_name);
        return;
    }
    let assembly_dir = args.next().unwrap();
    let output_dir = args.next().unwrap();

    // Load assembly
    let assembly_file = File::open(assembly_dir).expect("Failed to open assembly_file file");
    let lines: Vec<String> = BufReader::new(assembly_file)
        .lines()
        .map(|x| x.unwrap().clone())
        .collect();

    // Parse assembly
    let mut program: Vec<Token> = Vec::new();
    for (number, line) in lines.iter().enumerate() {
        match parse_token(line) {
            Ok((_, token)) => program.extend_from_slice(&token),
            Err(err) => panic!("line {}: {:?}", number, err),
        }
    }

    // Compile
    let mut translated_program = compile(&program);
    while translated_program.len() < 0x100 {
        translated_program.push(0x00);
    }

    // Save
    let mut file = File::create(output_dir).unwrap();
    file.write_all(&translated_program).unwrap();
    file.flush().unwrap();
}

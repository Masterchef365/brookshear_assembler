use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::str::FromStr;
mod compile;
mod op_codes;
mod parser;
mod token;
use compile::compile;
use parser::*;
use token::*;
extern crate nom;

fn main() {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();
    if args.len() < 2 {
        println!("Usage: {} <assembly.s> <output.bs>", program_name);
        return;
    }
    let assembly_dir = args.next().unwrap();
    let output_dir = args.next().unwrap();

    let assembly = File::open(assembly_dir).expect("Failed to open assembly file");

    //for line in BufReader::new(assembly).lines() {
    
    let mut program: Vec<Token> = Vec::new();

    let lines = [
        "LCON 1 01 ;",
        "LMEM 0 \"string_end\" ;",
        "LMEM 3 \"print_pointer\" ;",
        "LMEM 2 \"string_start\" : \"print_start\" : \"print_pointer\" ;",
        "CHAR 2 ;",
        "ADDR 3 31 ;",
        "STOR 3 \"print_pointer\" ;",
        "JUMP 3 \"halt\" ;",
        "JUMP 0 \"print_start\" ;",
        "DATA \"Hello, world!\" : \"string_start\" : \"string_end\" ;",
        "HALT 00 : \"halt\" ;",
    ];

    for (number, line) in lines.iter().enumerate() {
        match parse_token(line) {
            Ok((_, token)) => program.extend_from_slice(&token),
            Err(a) => println!("{:?}", a),
        }
    }

    let translated_program = compile(&program);
    for (position, chunk) in translated_program.chunks(2).enumerate() {
        println!("{:02X}: {:02X}{:02X} ", position * 2, chunk[0], chunk[1]);
    }

    let mut file = File::create(output_dir).unwrap();
    file.write_all(&translated_program).unwrap();
    file.flush().unwrap();
}

/*
   fn concat_nibbles(hi: u8, lo: u8) -> u8 {
   (hi * 0x10) + lo
   }

   fn data_segment<'a>(
   data: &[u8],
   start_label: Option<&'a str>,
   end_label: Option<&'a str>,
   ) -> Vec<Token<'a>> {
   let mut output: Vec<Token> = data.iter().map(|value| Token::from_const(*value)).collect();

// Add labels
output.first_mut().unwrap().label = start_label;
output.last_mut().unwrap().label = end_label;

// Preserve alignment, don't assign last after alignment because it's not part of the data
if output.len() & 1 == 1 {
output.push(Token::from_const(0x00));
}

return output;
}

fn const_instruction<'a>(a: u8, b: u8, c: u8, d: u8, label: Option<&'a str>) -> [Token<'a>; 2] {
[
Token {
value: TokenValue::Constant(concat_nibbles(a, b)),
label,
},
Token::from_const(concat_nibbles(c, d)),
]
}

fn set_register<'a>(register: u8, value: u8, label: Option<&'a str>) -> [Token<'a>; 2] {
[
Token {
value: TokenValue::Constant(concat_nibbles(LCON, register)),
label,
},
Token::from_const(value),
]
}

fn label_as_arg<'a>(
opcode: u8,
condition_reg: u8,
label: &'a str,
rejump: Option<&'a str>,
) -> [Token<'a>; 2] {
[
Token {
value: TokenValue::Constant(concat_nibbles(opcode, condition_reg)),
label: rejump,
},
Token {
value: TokenValue::Label(label),
label: None,
},
]
}

fn bain() {
let mut main: Vec<Token> = Vec::new();
main.extend_from_slice(&set_register(1, 0x01, None));
main.extend_from_slice(&label_as_arg(LMEM, 0, "string_end", None));
main.extend_from_slice(&label_as_arg(LMEM, 3, "print_pointer", None));
main.extend_from_slice(&[
Token {
value: TokenValue::Constant(concat_nibbles(LMEM, 2)),
label: Some("print_start"),
},
    Token {
        value: TokenValue::Label("string_start"),
        label: Some("print_pointer"),
    },
    ]);
main.extend_from_slice(&const_instruction(CHAR, 2, 0, 0, None));
main.extend_from_slice(&const_instruction(ADDR, 3, 3, 1, None));
main.extend_from_slice(&label_as_arg(STOR, 3, "print_pointer", None));
main.extend_from_slice(&label_as_arg(JUMP, 3, "halt", None));
main.extend_from_slice(&label_as_arg(JUMP, 0, "print_start", None));
main.extend(data_segment(
        b"bitchass",
        Some("string_start"),
        Some("string_end"),
));
main.extend_from_slice(&const_instruction(HALT, 0, 0, 0, Some("halt")));

let translated_program = compile(&mut main);
for (position, chunk) in translated_program.chunks(2).enumerate() {
    println!("{:02X}: {:02X}{:02X} ", position * 2, chunk[0], chunk[1]);
}

let mut file = File::create("Bitchass.sma").unwrap();
file.write_all(&translated_program).unwrap();
file.flush().unwrap();
}
*/

use std::collections::hash_map::HashMap;
use std::io::prelude::*;
use std::fs::File;

#[allow(dead_code)]
mod op_codes {
    pub const LMEM: u8 = 0x01;
    pub const LCON: u8 = 0x02;
    pub const STOR: u8 = 0x03;
    pub const MOVE: u8 = 0x04;
    pub const ADD: u8 = 0x05;
    pub const OR: u8 = 0x07;
    pub const AND: u8 = 0x08;
    pub const XOR: u8 = 0x09;
    pub const ROT: u8 = 0x0A;
    pub const JUMP: u8 = 0x0B;
    pub const HALT: u8 = 0x0C;
    pub const CHAR: u8 = 0x0D;
    pub const LESS: u8 = 0x0E;
    pub const DISP: u8 = 0x0F;
}

use op_codes::*;

fn concat_nibs(hi: u8, lo: u8) -> u8 {
    (hi * 0x10) + lo
}

#[derive(Debug, Clone)]
enum TokenValue {
    Constant(u8),
    LabelRef(String),
    DestructiveLabelRef(String),
}

#[derive(Debug, Clone)]
struct Token {
    value: TokenValue,
    label: Option<String>,
}

#[derive(Debug, Clone)]
enum Unit<'a> {
    Inline(&'a [Unit<'a>]),
    Token(Token),
}

fn count_program_length(program: &[Unit]) -> u8 {
    let mut count = 0;
    for unit in program {
        match unit {
            Unit::Inline(subprogram) => count += count_program_length(subprogram),
            Unit::Token(_) => count += 1,
        }
    }
    return count;
}

fn flatten(program: &[Unit]) -> Vec<Token> {
    let mut output: Vec<Token> = Vec::new();
    for unit in program {
        match unit {
            Unit::Token(token) => output.push(token.clone()),
            Unit::Inline(subprogram) => output.extend(flatten(subprogram)),
        }
    }
    return output;
}

fn translate(program: &[Token]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    let mut destructive_labelled_program = Vec::new();

    // Create label hashmap
    let mut label_hashmap = HashMap::new();
    for (address, token) in program.iter().enumerate() {
        if let Some(label) = &token.label {
            label_hashmap.insert(label, address as u8);
        }
        let mut new_token = token.clone();
        if let TokenValue::DestructiveLabelRef(label) = &token.value {
            if label_hashmap.contains_key(&label.clone()) {
                new_token.value = TokenValue::Constant(*label_hashmap.get(&label.clone()).unwrap());
                label_hashmap.remove_entry(&label.clone());
            }
            destructive_labelled_program.push(new_token);
        } else {
            destructive_labelled_program.push(token.clone());
        }
    }

    // Convert the program into memory
    for token in destructive_labelled_program {
        match &token.value {
            TokenValue::Constant(value) => output.push(*value),
            TokenValue::LabelRef(label) => {
                if let Some(value) = label_hashmap.get(&label) {
                    output.push(*value)
                } else {
                    panic!("Label not found: {}", label);
                }
            }
            _ => panic!(),
        };
    }
    return output;
}

fn data_segment(data: &[u8], label: String, end_label: Option<String>) -> Vec<Token> {
    let mut output = Vec::new();

    // Label the first token
    output.push(Token {
        label: Some(label),
        value: TokenValue::Constant(data[0]),
    });

    // Add the remaining data
    for character in data.iter().skip(1).take(data.len() - 2) {
        output.push(Token {
            label: None,
            value: TokenValue::Constant(*character),
        });
    }

    if let Some(end_label) = end_label {
        output.push(Token {
            label: Some(end_label),
            value: TokenValue::Constant(*data.last().unwrap()),
        });
    } else {
        output.push(Token {
            label: None,
            value: TokenValue::Constant(*data.last().unwrap()),
        });
    }

    // Preserve alignment
    if output.len() % 2 != 0 {
        output.push(Token {
            label: None,
            value: TokenValue::Constant(0x00),
        })
    }
    return output;
}

fn const_instruction(a: u8, b: u8, c: u8, d: u8, label: Option<String>) -> [Token; 2] {
    [
        Token {
            value: TokenValue::Constant(concat_nibs(a, b)),
            label,
        },
        Token {
            value: TokenValue::Constant(concat_nibs(c, d)),
            label: None,
        },
    ]
}

fn set_register(register: u8, value: u8, label: Option<String>) -> [Token; 2] {
    [
        Token {
            value: TokenValue::Constant(concat_nibs(LCON, register)),
            label,
        },
        Token {
            value: TokenValue::Constant(value),
            label: None,
        },
    ]
}

fn label_as_arg(
    opcode: u8,
    condition_reg: u8,
    label: String,
    rejump: Option<String>,
) -> [Token; 2] {
    [
        Token {
            value: TokenValue::Constant(concat_nibs(opcode, condition_reg)),
            label: rejump,
        },
        Token {
            value: TokenValue::LabelRef(label),
            label: None,
        },
    ]
}

fn main() {
    let mut main: Vec<Token> = Vec::new();
    main.extend_from_slice(&set_register(1, 0x01, None));
    main.extend_from_slice(&label_as_arg(LMEM, 0, "string_end".to_string(), None));
    main.extend_from_slice(&label_as_arg(LMEM, 3, "print_pointer".to_string(), None));
    main.extend_from_slice(&[
        Token {
            value: TokenValue::Constant(concat_nibs(LMEM, 2)),
            label: Some("print_start".to_string()),
        },
        Token {
            value: TokenValue::LabelRef("string_start".to_string()),
            label: Some("print_pointer".to_string()),
        },
    ]);
    main.extend_from_slice(&const_instruction(CHAR, 2, 0, 0, None));
    main.extend_from_slice(&const_instruction(ADD, 3, 3, 1, None));
    main.extend_from_slice(&label_as_arg(STOR, 3, "print_pointer".to_string(), None));
    main.extend_from_slice(&label_as_arg(JUMP, 3, "halt".to_string(), None));
    main.extend_from_slice(&label_as_arg(JUMP, 0, "print_start".to_string(), None));
    main.extend(data_segment(b"Hello, world!", "string_start".to_string(), Some("string_end".to_string())));
    main.extend_from_slice(&const_instruction(HALT, 0, 0, 0, Some("halt".to_string())));

    let translated_program = translate(&mut main);
    for (position, chunk) in translated_program.chunks(2).enumerate() {
        println!("{:02X}: {:02X}{:02X} ", position * 2, chunk[0], chunk[1]);
    }

    let mut file = File::create("Bitchass.sma").unwrap();
    file.write_all(&translated_program).unwrap();
    file.flush().unwrap();
}

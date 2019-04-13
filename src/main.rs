use std::collections::hash_map::HashMap;
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
                }
            }
            _ => panic!(),
        };
    }
    return output;
}

fn data_segment_string(text: String, label: String) -> Vec<Token> {
    let mut output = Vec::new();
    let text_ascii = text.into_bytes();
    output.push(Token {
        label: Some(label),
        value: TokenValue::Constant(text_ascii[0]),
    });

    for character in text_ascii[1:] {
        output.push(Token {
            label: None,
            value: TokenValue::Constant(character),
        });
    }

    return output;
}

fn main() {
    let main = Vec::new();

    let translated_program = translate(&mut main);
    for (position, chunk) in translated_program.chunks(2).enumerate() {
        println!("{:02X}: {:02X}{:02X} ", position * 2, chunk[0], chunk[1]);
    }
}

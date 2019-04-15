use std::collections::hash_map::HashMap;
use crate::token::*;

pub fn compile(program: &[Token]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();

    // Create label hashmap
    let mut label_hashmap = HashMap::new();
    for (address, token) in program.iter().enumerate() {
        if let Some(label) = &token.label {
            label_hashmap.insert(label, address as u8);
        }
    }

    // Convert the program into memory
    for token in program {
        match &token.value {
            TokenValue::Constant(value) => output.push(*value),
            TokenValue::Label(label) => {
                output.push(
                    *label_hashmap
                        .get(&label)
                        .expect(&format!("Label not found: {}", &label)),
                );
            }
        };
    }

    return output;
}

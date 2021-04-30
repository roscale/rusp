use std::collections::HashMap;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::jvm::constant_pool::ConstantPool;

#[derive(Debug)]
pub enum Instruction {
    Label(String),
    Bipush(u8),
    Istore(u8),
    Ldc(u8),
    Getstatic {
        class: String,
        field: String,
        field_type: String,
    },
    IfIcmpne(String),
    Invokevirtual {
        class: String,
        method: String,
        descriptor: String,
    },
    Return,
}

impl Instruction {
    pub fn len(&self) -> usize {
        use Instruction::*;
        match self {
            Label(_) => 0,
            Bipush(_) => 2,
            Istore(_) => 2,
            Ldc(_) => 2,
            Getstatic { .. } => 3,
            IfIcmpne(_) => 3,
            Invokevirtual { .. } => 3,
            Return => 1,
        }
    }
}

fn scan_for_labels(code: &Vec<Instruction>) -> HashMap<&str, usize> {
    let mut labels = HashMap::new();
    let mut i = 0;
    for instruction in code {
        match instruction {
            Instruction::Label(label) => {
                labels.insert(label.as_str(), i);
            }
            _ => {
                i += instruction.len()
            }
        }
    }
    labels
}

pub fn compile_instructions(code: &Vec<Instruction>, constant_pool: &mut ConstantPool) -> Vec<u8> {
    let labels = scan_for_labels(code);

    let mut final_instructions = Vec::new();

    let mut i = 0;
    for instruction in code {
        match instruction {
            Instruction::Label(_) => {}
            Instruction::Bipush(byte) => final_instructions.extend_from_slice(&[16, *byte]),
            Instruction::Istore(int) => final_instructions.extend_from_slice(&[54, *int]),
            Instruction::Ldc(index) => final_instructions.extend_from_slice(&[18, *index]),
            Instruction::Getstatic { class, field, field_type } => {
                let index = constant_pool.add_field(class.clone(), field.clone(), field_type.clone());
                final_instructions.push(178);
                final_instructions.write_u16::<BigEndian>(index).unwrap();
            }

            Instruction::IfIcmpne(label) => {
                final_instructions.push(160);
                let offset = {
                    let target = *labels.get(label.as_str()).expect(&format!("Label \"{}\" does not exist!", label)) as isize;
                    let here = i as isize;
                    (target - here) as i16
                };
                final_instructions.write_i16::<BigEndian>(offset).unwrap();
            }
            Instruction::Invokevirtual { class, method, descriptor } => {
                let index = constant_pool.add_method(class.clone(), method.clone(), descriptor.clone());
                final_instructions.push(182);
                final_instructions.write_u16::<BigEndian>(index).unwrap();
            }
            Instruction::Return => final_instructions.extend_from_slice(&[177]),
        };
        i += instruction.len();
    }
    final_instructions
}
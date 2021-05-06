use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::jvm::constant_pool::ConstantPool;
use crate::jvm::variable_stack::VariableStack;

#[derive(Debug)]
pub enum Instruction {
    Label(String),
    Goto(String),
    Bipush(u8),
    Istore(u8),
    Ldc(i32),
    Iadd,
    Iload(u8),
    Getstatic {
        class: String,
        field: String,
        field_type: String,
    },
    IfIcmpne(String),
    Ifne(String),
    Ifeq(String),
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
            Goto(_) => 3,
            Bipush(_) => 2,
            Istore(_) => 2,
            Ldc(_) => 2,
            Iadd => 1,
            Iload(_) => 2,
            Getstatic { .. } => 3,
            IfIcmpne(_) => 3,
            Ifne(_) => 3,
            Ifeq(_) => 3,
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

    let mut bytecode = Vec::new();

    let mut i = 0;
    for instruction in code {
        match instruction {
            Instruction::Label(_) => {}
            Instruction::Goto(label) => {
                bytecode.push(167);
                let offset = {
                    let target = *labels.get(label.as_str()).expect(&format!("Label \"{}\" does not exist!", label)) as isize;
                    let here = i as isize;
                    (target - here) as i16
                };
                bytecode.write_i16::<BigEndian>(offset).unwrap();
            }
            Instruction::Bipush(byte) => bytecode.extend_from_slice(&[16, *byte]),
            Instruction::Istore(index) => bytecode.extend_from_slice(&[54, *index]),
            Instruction::Ldc(integer) => {
                let index = constant_pool.add_integer(*integer);
                match index.try_into() {
                    Ok(byte_index) => { // ldc
                        bytecode.extend_from_slice(&[18, byte_index])
                    }
                    Err(_) => { // ldc_w
                        bytecode.push(19);
                        bytecode.write_u16::<BigEndian>(index).unwrap();
                    }
                }
            }
            Instruction::Iadd => bytecode.push(96),
            Instruction::Iload(index) => bytecode.extend_from_slice(&[21, *index]),
            Instruction::Getstatic { class, field, field_type } => {
                let index = constant_pool.add_field(class.clone(), field.clone(), field_type.clone());
                bytecode.push(178);
                bytecode.write_u16::<BigEndian>(index).unwrap();
            }
            Instruction::IfIcmpne(label) => {
                bytecode.push(160);
                let offset = {
                    let target = *labels.get(label.as_str()).expect(&format!("Label \"{}\" does not exist!", label)) as isize;
                    let here = i as isize;
                    (target - here) as i16
                };
                bytecode.write_i16::<BigEndian>(offset).unwrap();
            }
            Instruction::Ifne(label) => {
                bytecode.push(154);
                let offset = {
                    let target = *labels.get(label.as_str()).expect(&format!("Label \"{}\" does not exist!", label)) as isize;
                    let here = i as isize;
                    (target - here) as i16
                };
                bytecode.write_i16::<BigEndian>(offset).unwrap();
            }
            Instruction::Ifeq(label) => {
                bytecode.push(153);
                let offset = {
                    let target = *labels.get(label.as_str()).expect(&format!("Label \"{}\" does not exist!", label)) as isize;
                    let here = i as isize;
                    (target - here) as i16
                };
                bytecode.write_i16::<BigEndian>(offset).unwrap();
            }
            Instruction::Invokevirtual { class, method, descriptor } => {
                let index = constant_pool.add_method(class.clone(), method.clone(), descriptor.clone());
                bytecode.push(182);
                bytecode.write_u16::<BigEndian>(index).unwrap();
            }
            Instruction::Return => bytecode.extend_from_slice(&[177]),
        };
        i += instruction.len();
    }
    bytecode
}
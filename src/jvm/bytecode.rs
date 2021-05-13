use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::jvm::constant_pool::ConstantPool;
use crate::jvm::variable_stack::VariableStack;

pub type Label = u64;
pub type PoolIndex = u8;
pub type WidePoolIndex = u16;

#[derive(Debug)]
pub enum Instruction {
    Label(Label),
    Goto(Label),
    Bipush(u8),
    Ldc(PoolIndex),
    Istore(PoolIndex),
    Astore(PoolIndex),
    Iadd,
    Iload(PoolIndex),
    Aload(PoolIndex),
    Getstatic(WidePoolIndex),
    IfIcmpeq(Label),
    IfIcmpne(Label),
    Ifne(Label),
    Ifeq(Label),
    Invokevirtual(WidePoolIndex),
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
            Astore(_) => 2,
            Ldc(_) => 2,
            Iadd => 1,
            Iload(_) => 2,
            Aload(_) => 2,
            Getstatic(_) => 3,
            IfIcmpeq(_) => 3,
            IfIcmpne(_) => 3,
            Ifeq(_) => 3,
            Ifne(_) => 3,
            Invokevirtual(_) => 3,
            Return => 1,
        }
    }
}

fn scan_for_labels(code: &Vec<Instruction>) -> HashMap<Label, usize> {
    let mut labels = HashMap::new();
    let mut i = 0;
    for instruction in code {
        match instruction {
            Instruction::Label(label) => {
                labels.insert(*label, i);
            }
            _ => {
                i += instruction.len()
            }
        }
    }
    labels
}

pub fn compile_instructions(code: &Vec<Instruction>) -> Vec<u8> {
    let labels = scan_for_labels(code);
    let mut bytecode = Vec::new();

    let mut i = 0;

    for instruction in code {
        let get_target_offset = |label: &Label| {
            let target = *labels.get(label).expect(&format!("Label \"{}\" does not exist!", label)) as isize;
            let here = i as isize;
            (target - here) as i16
        };

        match instruction {
            Instruction::Label(_) => {}
            Instruction::Goto(label) => {
                bytecode.push(167);
                bytecode.write_i16::<BigEndian>(get_target_offset(label)).unwrap();
            }
            Instruction::Bipush(byte) => {
                bytecode.extend_from_slice(&[16, *byte])
            }
            Instruction::Istore(index) => {
                bytecode.extend_from_slice(&[54, *index])
            }
            Instruction::Astore(index) => {
                bytecode.extend_from_slice(&[58, *index])
            }
            Instruction::Ldc(index) => {
                bytecode.extend_from_slice(&[18, *index])
            }
            Instruction::Iadd => {
                bytecode.push(96)
            }
            Instruction::Iload(index) => {
                bytecode.extend_from_slice(&[21, *index])
            }
            Instruction::Aload(index) => {
                bytecode.extend_from_slice(&[25, *index])
            }
            Instruction::Getstatic(index) => {
                bytecode.push(178);
                bytecode.write_u16::<BigEndian>(*index).unwrap();
            }
            Instruction::IfIcmpeq(label) => {
                bytecode.push(159);
                bytecode.write_i16::<BigEndian>(get_target_offset(label)).unwrap();
            }
            Instruction::IfIcmpne(label) => {
                bytecode.push(160);
                bytecode.write_i16::<BigEndian>(get_target_offset(label)).unwrap();
            }
            Instruction::Ifne(label) => {
                bytecode.push(154);
                bytecode.write_i16::<BigEndian>(get_target_offset(label)).unwrap();
            }
            Instruction::Ifeq(label) => {
                bytecode.push(153);
                bytecode.write_i16::<BigEndian>(get_target_offset(label)).unwrap();
            }
            Instruction::Invokevirtual(index) => {
                bytecode.push(182);
                bytecode.write_u16::<BigEndian>(*index).unwrap();
            }
            Instruction::Return => {
                bytecode.extend_from_slice(&[177])
            }
        };
        i += instruction.len();
    }
    bytecode
}
use std::collections::HashMap;
use std::convert::TryInto;

use byteorder::BigEndian;

use crate::jvm::bytecode::{Instruction, Label};
use crate::jvm::constant_pool::ConstantPool;
use crate::jvm::jvm_type::{JvmType, PushLiteral};
use crate::jvm::label_generator::LabelGenerator;
use crate::jvm::variable_stack::VariableStack;

#[derive(Debug)]
pub enum PseudoInstruction {
    Label(Label),
    Goto(Label),
    Push(PushLiteral),
    Load(String),
    Store(String, bool),
    Drop(String),
    Add,
    Cmpeq,
    Cmpne,
    Ifeq(Label),
    Ifne(Label),
    Getstatic {
        class: String,
        field: String,
        field_type: String,
    },
    Invokevirtual {
        class: String,
        method: String,
        descriptor: String,
    },
    Return,
}

pub fn compile_to_jvm_instructions(
    pseudo_instructions: Vec<PseudoInstruction>,
    label_generator: &mut LabelGenerator,
    constant_pool: &mut ConstantPool,
) -> Vec<Instruction> {
    let mut instructions = vec![];
    let mut variable_stack = VariableStack::new();
    let mut operand_stack = vec![];

    for instruction in pseudo_instructions {
        match instruction {
            PseudoInstruction::Label(label) => instructions.push(Instruction::Label(label)),
            PseudoInstruction::Goto(label) => instructions.push(Instruction::Goto(label)),
            PseudoInstruction::Push(value) => {
                match value {
                    PushLiteral::Boolean(bool) => {
                        operand_stack.push(JvmType::Boolean);
                        instructions.push(Instruction::Bipush(bool as u8))
                    }
                    PushLiteral::Byte(byte) => {
                        operand_stack.push(JvmType::Byte);
                        instructions.push(Instruction::Bipush(byte))
                    }
                    PushLiteral::Char(char) => todo!(),
                    PushLiteral::Short(_) => todo!(),
                    PushLiteral::Int(int) => {
                        operand_stack.push(JvmType::Int);

                        let index = constant_pool.add_integer(int);
                        match index.try_into() {
                            Ok(byte_index) => { // ldc
                                instructions.push(Instruction::Ldc(byte_index));
                            }
                            Err(_) => { // ldc_w
                                todo!();
                            }
                        }
                    }
                    PushLiteral::Long(_) => todo!(),
                    PushLiteral::Float(_) => todo!(),
                    PushLiteral::Double(_) => todo!(),
                    PushLiteral::String(string) => {
                        operand_stack.push(JvmType::Reference);
                        let index = constant_pool.add_string(string);
                        match index.try_into() {
                            Ok(byte_index) => { // ldc
                                instructions.push(Instruction::Ldc(byte_index));
                            }
                            Err(_) => { // ldc_w
                                todo!();
                            }
                        }
                    }
                }
            }
            PseudoInstruction::Load(var) => {
                match variable_stack.get(&var) {
                    None => todo!(),
                    Some((index, jvm_type)) => {
                        operand_stack.push(jvm_type);
                        match jvm_type {
                            JvmType::Boolean => instructions.push(Instruction::Iload(index)),
                            JvmType::Byte => todo!(),
                            JvmType::Char => todo!(),
                            JvmType::Short => todo!(),
                            JvmType::Int => instructions.push(Instruction::Iload(index)),
                            JvmType::Long => todo!(),
                            JvmType::Float => todo!(),
                            JvmType::Double => todo!(),
                            JvmType::Reference => instructions.push(Instruction::Aload(index)),
                        }
                    }
                }
            }
            PseudoInstruction::Store(var, create) => {
                match operand_stack.pop() {
                    None => todo!(),
                    Some(jvm_type) => {
                        let index = match create {
                            true => variable_stack.create(var, jvm_type),
                            false => match variable_stack.get(&var) {
                                None => todo!(),
                                Some((index, _)) => index,
                            }
                        };
                        match jvm_type {
                            JvmType::Boolean => todo!(),
                            JvmType::Byte => todo!(),
                            JvmType::Char => todo!(),
                            JvmType::Short => todo!(),
                            JvmType::Int => instructions.push(Instruction::Istore(index)),
                            JvmType::Long => todo!(),
                            JvmType::Float => todo!(),
                            JvmType::Double => todo!(),
                            JvmType::Reference => instructions.push(Instruction::Astore(index)),
                        }
                    }
                }
            }
            PseudoInstruction::Drop(var) => {
                variable_stack.drop(&var);
            }
            PseudoInstruction::Add => {
                use JvmType::*;
                match (operand_stack.pop(), operand_stack.pop()) {
                    (None, _) | (_, None) => todo!(),
                    (Some(Int), Some(Int)) => {
                        operand_stack.push(Int);
                        instructions.push(Instruction::Iadd);
                    }
                    _ => todo!(),
                }
            }
            PseudoInstruction::Cmpeq => {
                use JvmType::*;
                match (operand_stack.pop(), operand_stack.pop()) {
                    (None, _) | (_, None) => todo!(),
                    (Some(Int), Some(Int)) => {
                        operand_stack.push(Int);
                        let false_label = label_generator.get_new_label();
                        let out_label = label_generator.get_new_label();
                        instructions.push(Instruction::IfIcmpne(false_label));
                        instructions.push(Instruction::Bipush(1));
                        instructions.push(Instruction::Goto(out_label));
                        instructions.push(Instruction::Label(false_label));
                        instructions.push(Instruction::Bipush(0));
                        instructions.push(Instruction::Label(out_label));
                    }
                    _ => todo!(),
                }
            }
            PseudoInstruction::Cmpne => {
                use JvmType::*;
                match (operand_stack.pop(), operand_stack.pop()) {
                    (None, _) | (_, None) => todo!(),
                    (Some(Int), Some(Int)) => {
                        operand_stack.push(Int);
                        let false_label = label_generator.get_new_label();
                        let out_label = label_generator.get_new_label();
                        instructions.push(Instruction::IfIcmpeq(false_label));
                        instructions.push(Instruction::Bipush(1));
                        instructions.push(Instruction::Goto(out_label));
                        instructions.push(Instruction::Label(false_label));
                        instructions.push(Instruction::Bipush(0));
                        instructions.push(Instruction::Label(out_label));
                    }
                    _ => todo!(),
                }
            }

            PseudoInstruction::Ifeq(label) => {
                operand_stack.pop();
                instructions.push(Instruction::Ifeq(label));
            }
            PseudoInstruction::Ifne(label) => {
                operand_stack.pop();
                instructions.push(Instruction::Ifne(label));
            }
            PseudoInstruction::Getstatic { class, field, field_type } => {
                let index = constant_pool.add_field(class.clone(), field.clone(), field_type.clone());
                instructions.push(Instruction::Getstatic(index));
            }
            PseudoInstruction::Invokevirtual { class, method, descriptor } => {
                let index = constant_pool.add_method(class.clone(), method.clone(), descriptor.clone());
                instructions.push(Instruction::Invokevirtual(index));
            }
            PseudoInstruction::Return => instructions.push(Instruction::Return),
        };
    }

    instructions
}
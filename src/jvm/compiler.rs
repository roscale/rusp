use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::jvm::bytecode::{compile_instructions, Instruction, Label};
use crate::jvm::constant_pool::ConstantPool;
use crate::jvm::jvm_type::PushLiteral;
use crate::jvm::label_generator::LabelGenerator;
use crate::jvm::pseudo_instruction::{compile_to_jvm_instructions, PseudoInstruction};
use crate::jvm::structs::{Class, Method};
use crate::jvm::variable_stack::VariableStack;
use crate::lexer::Operator;
use crate::parser::{Expression, ExpressionWithMetadata, Value};

pub struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool: ConstantPool,
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    methods: Vec<InternalMethod>,
    attributes: Vec<GenericAttribute>,
}

pub enum ClassAccessFlags {
    Public = 0x0001,
    Final = 0x0010,
    Super = 0x0020,
    Interface = 0x0200,
    Abstract = 0x0400,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
}

pub(crate) enum MethodAccessFlags {
    Public = 1 << 0,
    Private = 1 << 1,
    Protected = 1 << 2,
    Static = 1 << 3,
    Final = 1 << 4,
    Synchronized = 1 << 5,
    Bridge = 1 << 6,
    Varargs = 1 << 7,
    Native = 1 << 8,
    Abstract = 1 << 9,
    Strict = 1 << 10,
    Synthetic = 1 << 11,
}

struct InternalMethod {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<GenericAttribute>,
}

struct GenericAttribute {
    name_index: u16,
    info: Vec<u8>,
}

struct CodeAttribute {
    name_index: u16,
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    attributes: Vec<GenericAttribute>,
}

impl From<CodeAttribute> for GenericAttribute {
    fn from(code_attribute: CodeAttribute) -> Self {
        Self {
            name_index: code_attribute.name_index,
            info: {
                let mut info = Vec::new();
                info.write_u16::<BigEndian>(code_attribute.max_stack).unwrap();
                info.write_u16::<BigEndian>(code_attribute.max_locals).unwrap();
                info.write_u32::<BigEndian>(code_attribute.code.len() as u32).unwrap();
                info.write_all(code_attribute.code.as_slice()).unwrap();
                info.write_u16::<BigEndian>(0).unwrap(); // exception table length
                info.write_u16::<BigEndian>(code_attribute.attributes.len() as u16).unwrap();
                for attribute in &code_attribute.attributes {
                    info.write_u16::<BigEndian>(attribute.name_index).unwrap();
                    info.write_u32::<BigEndian>(attribute.info.len() as u32).unwrap();
                    info.write_all(attribute.info.as_slice()).unwrap();
                }
                info
            },
        }
    }
}

struct LineNumberTableAttribute {
    name_index: u16,
    items: Vec<LineNumberItem>,
}

struct LineNumberItem {
    start_pc: u16,
    line_number: u16,
}

impl From<LineNumberTableAttribute> for GenericAttribute {
    fn from(line_number_table_attribute: LineNumberTableAttribute) -> Self {
        Self {
            name_index: line_number_table_attribute.name_index,
            info: {
                let mut info = Vec::new();
                info.write_u16::<BigEndian>(line_number_table_attribute.items.len() as u16).unwrap();
                for item in &line_number_table_attribute.items {
                    info.write_u16::<BigEndian>(item.start_pc as u16).unwrap();
                    info.write_u16::<BigEndian>(item.line_number as u16).unwrap();
                }
                info
            },
        }
    }
}

struct SourceFileAttribute {
    name_index: u16,
    sourcefile_index: u16,
}

impl From<SourceFileAttribute> for GenericAttribute {
    fn from(source_file_attribute: SourceFileAttribute) -> Self {
        Self {
            name_index: source_file_attribute.name_index,
            info: {
                let mut info = Vec::new();
                info.write_u16::<BigEndian>(source_file_attribute.sourcefile_index).unwrap();
                info
            },
        }
    }
}

impl ClassFile {
    pub fn new() -> Self {
        ClassFile {
            magic: 0xCAFEBABE,
            minor_version: 0,
            major_version: 52,
            constant_pool: ConstantPool::new(),
            access_flags: ClassAccessFlags::Public as u16 | ClassAccessFlags::Super as u16,
            this_class: 0,
            super_class: 0,
            methods: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn write_to_file(&mut self) -> io::Result<()> {
        let mut file = File::create("Main.class").unwrap();

        file.write_u32::<BigEndian>(self.magic)?;
        file.write_u16::<BigEndian>(self.minor_version)?;
        file.write_u16::<BigEndian>(self.major_version)?;
        file.write_u16::<BigEndian>(self.constant_pool.len() as u16 + 1)?;

        self.constant_pool.write_to_file(&mut file)?;

        file.write_u16::<BigEndian>(self.access_flags)?;
        file.write_u16::<BigEndian>(self.this_class)?;
        file.write_u16::<BigEndian>(self.super_class)?;
        file.write_u16::<BigEndian>(0)?; // interfaces count
        file.write_u16::<BigEndian>(0)?; // fields count

        file.write_u16::<BigEndian>(self.methods.len() as u16)?;
        for method in &self.methods {
            file.write_u16::<BigEndian>(method.access_flags)?;
            file.write_u16::<BigEndian>(method.name_index)?;
            file.write_u16::<BigEndian>(method.descriptor_index)?;
            file.write_u16::<BigEndian>(method.attributes.len() as u16)?;
            for attribute in &method.attributes {
                file.write_u16::<BigEndian>(attribute.name_index)?;
                file.write_u32::<BigEndian>(attribute.info.len() as u32)?;
                file.write_all(attribute.info.as_slice())?;
            }
        }

        file.write_u16::<BigEndian>(self.attributes.len() as u16)?;
        for attribute in &self.attributes {
            file.write_u16::<BigEndian>(attribute.name_index)?;
            file.write_u32::<BigEndian>(attribute.info.len() as u32)?;
            file.write_all(attribute.info.as_slice())?;
        }

        Ok(())
    }
}

pub struct CodeCompiler {
    code: Vec<PseudoInstruction>,
    variables: VariableStack,
    label_generator: LabelGenerator,
}

impl CodeCompiler {
    pub fn new() -> Self {
        CodeCompiler {
            code: vec![],
            variables: VariableStack::new(),
            label_generator: LabelGenerator::new(),
        }
    }

    pub fn compile_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Value(Value::Integer(int)) => {
                self.code.push(PseudoInstruction::Push(PushLiteral::Int(*int)));
            }
            Expression::Value(Value::String(string)) => {
                self.code.push(PseudoInstruction::Push(PushLiteral::String(string.clone())));
            }
            Expression::Id(name) => {
                self.code.push(PseudoInstruction::Load(name.clone()));
            }
            Expression::Scope(expressions) => {
                // TODO: implement shadowing and drop
                for e in expressions {
                    self.compile_expression(&e.expression);
                }
            }
            Expression::Declaration(label, rhs) => {
                self.compile_expression(&rhs.expression);
                self.code.push(PseudoInstruction::Store(label.label.clone(), true));
            }
            Expression::Assignment(label, rsh) => {
                self.compile_expression(&rsh.expression);
                self.code.push(PseudoInstruction::Store(label.label.clone(), false));
            }
            Expression::Operation(operator, terms) => {
                match terms.split_first() {
                    None => panic!(), // TODO
                    Some((first, tail)) => {
                        self.compile_expression(&first.expression);
                        for term in tail {
                            self.compile_expression(&term.expression);
                            match operator {
                                Operator::Plus => self.code.push(PseudoInstruction::Add),
                                Operator::Equality => self.code.push(PseudoInstruction::Cmpeq),
                                Operator::Inequality => self.code.push(PseudoInstruction::Cmpne),
                            }
                        }
                    }
                }
            }
            Expression::If { guard, base_case } => {
                let out_label = self.label_generator.get_new_label();
                self.compile_expression(&guard.expression);
                self.code.push(PseudoInstruction::Ifeq(out_label));
                self.compile_expression(&base_case.expression);
                self.code.push(PseudoInstruction::Label(out_label));
            }
            Expression::IfElse { guard, base_case, else_case } => {
                let else_label = self.label_generator.get_new_label();
                let out_label = self.label_generator.get_new_label();
                self.compile_expression(&guard.expression);
                self.code.push(PseudoInstruction::Ifeq(else_label));
                self.compile_expression(&base_case.expression);
                self.code.push(PseudoInstruction::Goto(out_label));
                self.code.push(PseudoInstruction::Label(else_label));
                self.compile_expression(&else_case.expression);
                self.code.push(PseudoInstruction::Label(out_label));
            }
            Expression::While { guard, body } => {
                let guard_label = self.label_generator.get_new_label();
                let out_label = self.label_generator.get_new_label();
                self.code.push(PseudoInstruction::Label(guard_label));
                self.compile_expression(&guard.expression);
                self.code.push(PseudoInstruction::Ifeq(out_label));
                self.compile_expression(&body.expression);
                self.code.push(PseudoInstruction::Goto(guard_label));
                self.code.push(PseudoInstruction::Label(out_label));
            }
            Expression::FunctionCall(name, arguments) => {
                let name = match &name.expression {
                    Expression::Id(name) => name,
                    _ => panic!(),
                };
                if name != "println" {
                    panic!();
                }
                self.code.push(PseudoInstruction::Getstatic {
                    class: "java/lang/System".to_string(),
                    field: "out".to_string(),
                    field_type: "Ljava/io/PrintStream;".to_string(),
                });
                for argument in arguments {
                    self.compile_expression(&argument.expression);
                }
                self.code.push(PseudoInstruction::Invokevirtual {
                    class: "java/io/PrintStream".to_string(),
                    method: "println".to_string(),
                    descriptor: "(Ljava/lang/String;)V".to_string(),
                });
            }
            _ => unimplemented!()
        }
    }
}

pub fn to_bytecode(expressions: Vec<ExpressionWithMetadata>) -> io::Result<()> {
    let expressions = (|| {
        for expr in expressions {
            if let Expression::NamedFunctionDefinition {
                name, parameters, body,
            } = expr.expression {
                if name.label == "main" {
                    if let Expression::Scope(scope) = body.expression {
                        return scope;
                    }
                }
            }
        }
        unreachable!();
    })();

    let mut code_compiler = CodeCompiler::new();
    for e in expressions {
        code_compiler.compile_expression(&e.expression);
    }

    let (mut code, mut variables, mut label_generator) =
        (code_compiler.code, code_compiler.variables, code_compiler.label_generator);
    code.push(PseudoInstruction::Return);

    let class_files = compile(vec![
        Class {
            name: "Main".to_string(),
            methods: vec![
                Method {
                    name: "main".to_string(),
                    signature: "([Ljava/lang/String;)V".to_string(),
                    label_generator,
                    code,
                    ..Default::default()
                }
            ],
            ..Default::default()
        },
    ]);

    for mut class_file in class_files {
        class_file.write_to_file()?
    }
    Ok(())
}

pub fn compile(classes: Vec<Class>) -> Vec<ClassFile> {
    classes.into_iter().map(|class| {
        let mut class_file = ClassFile::new();

        class_file.this_class = class_file.constant_pool.add_class(class.name);
        class_file.super_class = class_file.constant_pool.add_class("java/lang/Object".to_string());
        class_file.access_flags = class.access_flags;

        for mut method in class.methods {
            class_file.methods.push(InternalMethod {
                access_flags: method.access_flags,
                name_index: class_file.constant_pool.add_utf8(method.name),
                descriptor_index: class_file.constant_pool.add_utf8(method.signature),
                attributes: vec![
                    CodeAttribute {
                        name_index: class_file.constant_pool.add_utf8("Code".to_string()),
                        max_stack: 10,
                        max_locals: 10,
                        code: {
                            let instructions = compile_to_jvm_instructions(method.code, &mut method.label_generator, &mut class_file.constant_pool);
                            compile_instructions(&instructions)
                        },
                        attributes: vec![],
                    }.into()
                ],
            })
        }
        class_file
    }).collect()
}

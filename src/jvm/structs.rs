use crate::jvm::bytecode::Instruction;
use crate::jvm::compiler::ClassAccessFlags;
use crate::jvm::compiler::MethodAccessFlags;
use crate::jvm::label_generator::LabelGenerator;
use crate::jvm::pseudo_instruction::PseudoInstruction;

pub struct Class {
    pub name: String,
    pub access_flags: u16,
    pub methods: Vec<Method>,
}

impl Default for Class {
    fn default() -> Self {
        Self {
            name: "Class".to_string(),
            access_flags: ClassAccessFlags::Public as u16 | ClassAccessFlags::Super as u16,
            methods: Vec::new(),
        }
    }
}

pub struct Method {
    pub name: String,
    pub signature: String,
    pub access_flags: u16,
    pub label_generator: LabelGenerator,
    pub code: Vec<PseudoInstruction>,
}

impl Default for Method {
    fn default() -> Self {
        Self {
            name: "Method".to_string(),
            signature: "".to_string(),
            access_flags: MethodAccessFlags::Public as u16 | MethodAccessFlags::Static as u16,
            label_generator: LabelGenerator::new(),
            code: vec![],
        }
    }
}
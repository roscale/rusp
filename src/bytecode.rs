#[derive(Debug)]
pub enum Bytecode {
    Bipush(u8),
    Istore(u8),
    Return,
}

impl Bytecode {
    pub fn to_machine_code(&self) -> u8 {
        match self {
            Bytecode::Bipush(_) => 16,
            Bytecode::Istore(_) => 54,
            Bytecode::Return => 177,
        }
    }
}
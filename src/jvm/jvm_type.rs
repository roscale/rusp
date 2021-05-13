#[derive(Debug, Copy, Clone)]
pub enum JvmType {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Reference,
}

#[derive(Debug)]
pub enum PushLiteral {
    Boolean(bool),
    Byte(u8),
    Char(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
}

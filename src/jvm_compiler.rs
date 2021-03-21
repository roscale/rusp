use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::parser::ExpressionWithMetadata;

struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool_table: Vec<ConstantPoolItem>,
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    methods: Vec<Method>,
    attributes: Vec<GenericAttribute>,
}

enum ConstantPoolItem {
    String(String),
    ClassRef(u16),
    NameAndType { name: u16, descriptor: u16 },
    MethodRef { class_ref: u16, name_and_type: u16 },
}

enum ClassAccessFlags {
    Public = 0x0001,
    Final = 0x0010,
    Super = 0x0020,
    Interface = 0x0200,
    Abstract = 0x0400,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
}

enum MethodAccessFlags {
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

struct Method {
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

impl TryFrom<CodeAttribute> for GenericAttribute {
    type Error = io::Error;

    fn try_from(code_attribute: CodeAttribute) -> Result<Self, Self::Error> {
        Ok(Self {
            name_index: code_attribute.name_index,
            info: {
                let mut info = Vec::new();
                info.write_u16::<BigEndian>(code_attribute.max_stack)?;
                info.write_u16::<BigEndian>(code_attribute.max_locals)?;
                info.write_u32::<BigEndian>(code_attribute.code.len() as u32)?;
                info.write_all(code_attribute.code.as_slice())?;
                info.write_u16::<BigEndian>(0)?; // exception table length
                info.write_u16::<BigEndian>(code_attribute.attributes.len() as u16)?;
                for attribute in &code_attribute.attributes {
                    info.write_u16::<BigEndian>(attribute.name_index)?;
                    info.write_u32::<BigEndian>(attribute.info.len() as u32)?;
                    info.write_all(attribute.info.as_slice())?;
                }
                info
            },
        })
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

impl TryFrom<LineNumberTableAttribute> for GenericAttribute {
    type Error = io::Error;

    fn try_from(line_number_table_attribute: LineNumberTableAttribute) -> Result<Self, Self::Error> {
        Ok(Self {
            name_index: line_number_table_attribute.name_index,
            info: {
                let mut info = Vec::new();
                info.write_u16::<BigEndian>(line_number_table_attribute.items.len() as u16)?;
                for item in &line_number_table_attribute.items {
                    info.write_u16::<BigEndian>(item.start_pc as u16)?;
                    info.write_u16::<BigEndian>(item.line_number as u16)?;
                }
                info
            },
        })
    }
}

struct SourceFileAttribute {
    name_index: u16,
    sourcefile_index: u16,
}

impl TryFrom<SourceFileAttribute> for GenericAttribute {
    type Error = io::Error;
    
    fn try_from(source_file_attribute: SourceFileAttribute) -> Result<Self, Self::Error> {
        Ok(Self {
            name_index: source_file_attribute.name_index,
            info: {
                let mut info = Vec::new();
                info.write_u16::<BigEndian>(source_file_attribute.sourcefile_index)?;
                info
            },
        })
    }
}

impl ClassFile {
    pub fn new() -> Self {
        ClassFile {
            magic: 0xCAFEBABE,
            minor_version: 0,
            major_version: 52,
            constant_pool_table: Vec::new(),
            access_flags: ClassAccessFlags::Public as u16 | ClassAccessFlags::Super as u16,
            this_class: 0,
            super_class: 0,
            methods: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn add_class(&mut self, name: String) -> usize {
        self.constant_pool_table.push(ConstantPoolItem::String(name));
        self.constant_pool_table.push(ConstantPoolItem::ClassRef(self.constant_pool_table.len() as u16));
        self.constant_pool_table.len()
    }

    pub fn add_string(&mut self, name: String) -> usize {
        self.constant_pool_table.push(ConstantPoolItem::String(name));
        self.constant_pool_table.len()
    }

    pub fn write_to_file(&mut self) -> io::Result<()> {
        let mut file = File::create("Main.class").unwrap();

        file.write_u32::<BigEndian>(self.magic)?;
        file.write_u16::<BigEndian>(self.minor_version)?;
        file.write_u16::<BigEndian>(self.major_version)?;
        file.write_u16::<BigEndian>(self.constant_pool_table.len() as u16 + 1)?;

        for item in &self.constant_pool_table {
            match item {
                ConstantPoolItem::String(string) => {
                    file.write_u8(1)?;
                    file.write_u16::<BigEndian>(string.as_bytes().len() as u16)?;
                    file.write_all(string.as_bytes())?;
                }
                ConstantPoolItem::ClassRef(index) => {
                    file.write_u8(7)?;
                    file.write_u16::<BigEndian>(*index)?;
                }
                ConstantPoolItem::NameAndType { name, descriptor } => {
                    file.write_u8(12)?;
                    file.write_u16::<BigEndian>(*name)?;
                    file.write_u16::<BigEndian>(*descriptor)?;
                }
                ConstantPoolItem::MethodRef { class_ref, name_and_type } => {
                    file.write_u8(10)?;
                    file.write_u16::<BigEndian>(*class_ref)?;
                    file.write_u16::<BigEndian>(*name_and_type)?;
                }
            }
        }

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

pub fn to_bytecode(expressions: Vec<ExpressionWithMetadata>) -> io::Result<()> {
    let mut class_file = ClassFile::new();

    let n1 = {
        class_file.constant_pool_table.push(ConstantPoolItem::MethodRef { class_ref: 3, name_and_type: 12 });
        class_file.constant_pool_table.len()
    };
    let n2 = {
        class_file.constant_pool_table.push(ConstantPoolItem::ClassRef(13));
        class_file.constant_pool_table.len()
    };
    let n3 = {
        class_file.constant_pool_table.push(ConstantPoolItem::ClassRef(14));
        class_file.constant_pool_table.len()
    };
    let n4 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("<init>".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n5 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("()V".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n6 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("Code".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n7 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("LineNumberTable".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n8 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("main".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n9 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("([Ljava/lang/String;)V".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n10 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("SourceFile".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n11 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("Main.java".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n12 = {
        class_file.constant_pool_table.push(ConstantPoolItem::NameAndType { name: 4, descriptor: 5 });
        class_file.constant_pool_table.len()
    };
    let n13 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("Main".to_owned()));
        class_file.constant_pool_table.len()
    };
    let n14 = {
        class_file.constant_pool_table.push(ConstantPoolItem::String("java/lang/Object".to_owned()));
        class_file.constant_pool_table.len()
    };

    class_file.this_class = 2;
    class_file.super_class = 3;

    class_file.methods.push(Method {
        access_flags: MethodAccessFlags::Public as u16,
        name_index: 4,
        descriptor_index: 5,
        attributes: vec![
            CodeAttribute {
                name_index: 6,
                max_stack: 1,
                max_locals: 1,
                code: vec![0x2a, 0xb7, 0x00, 0x01, 0xb1],
                attributes: vec![
                    LineNumberTableAttribute {
                        name_index: 7,
                        items: vec![
                            LineNumberItem { start_pc: 0, line_number: 1 }.into(),
                        ],
                    }.try_into()?
                ],
            }.try_into()?
        ],
    });

    class_file.methods.push(Method {
        access_flags: MethodAccessFlags::Public as u16 | MethodAccessFlags::Static as u16,
        name_index: 8,
        descriptor_index: 9,
        attributes: vec![
            CodeAttribute {
                name_index: 6,
                max_stack: 1,
                max_locals: 2,
                code: vec![0x03, 0x3c, 0xb1],
                attributes: vec![
                    LineNumberTableAttribute {
                        name_index: 7,
                        items: vec![
                            LineNumberItem { start_pc: 0, line_number: 3 }.into(),
                            LineNumberItem { start_pc: 2, line_number: 4 }.into(),
                        ],
                    }.try_into()?
                ],
            }.try_into()?
        ],
    });

    class_file.attributes.push(SourceFileAttribute {
        name_index: 10,
        sourcefile_index: 11,
    }.try_into()?);

    // class_file.this_class = class_file.add_class("Main".to_owned()) as u16;
    // class_file.super_class = class_file.add_class("java/lang/Object".to_owned()) as u16;
    //
    // let constructor = Method {
    //     access_flags: MethodAccessFlags::Public as u16,
    //     name_index: {
    //         class_file.constant_pool_table.push(ConstantPoolItem::String("<init>".to_owned()));
    //         class_file.constant_pool_table.len() as u16
    //     },
    //     descriptor_index: {
    //         class_file.constant_pool_table.push(ConstantPoolItem::String("()V".to_owned()));
    //         class_file.constant_pool_table.len() as u16
    //     },
    //     attributes: vec![
    //         CodeAttribute {
    //             name_index: {
    //                 class_file.constant_pool_table.push(ConstantPoolItem::String("Code".to_owned()));
    //                 class_file.constant_pool_table.len() as u16
    //             },
    //             max_stack: 1,
    //             max_locals: 2,
    //             code: vec![0x2a, 0xb7, 0x00, 0x01, 0xb1],
    //             attributes: vec![],
    //         }.into()
    //     ],
    // };
    // class_file.methods.push(constructor);
    //
    // let main_method = Method {
    //     access_flags: MethodAccessFlags::Public as u16 | MethodAccessFlags::Static as u16,
    //     name_index: {
    //         class_file.constant_pool_table.push(ConstantPoolItem::String("main".to_owned()));
    //         class_file.constant_pool_table.len() as u16
    //     },
    //     descriptor_index: {
    //         class_file.constant_pool_table.push(ConstantPoolItem::String("([Ljava/lang/String;)V".to_owned()));
    //         class_file.constant_pool_table.len() as u16
    //     },
    //     attributes: vec![
    //         CodeAttribute {
    //             name_index: {
    //                 class_file.constant_pool_table.push(ConstantPoolItem::String("Code".to_owned()));
    //                 class_file.constant_pool_table.len() as u16
    //             },
    //             max_stack: 1,
    //             max_locals: 2,
    //             code: vec![0x03, 0x3c, 0xb1],
    //             attributes: vec![],
    //         }.into()
    //     ]
    // };
    // class_file.methods.push(main_method);

    class_file.write_to_file()
}
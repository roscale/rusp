use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

pub struct Utf8(String);

pub struct JString(Utf8);

pub struct Class(Utf8);

pub struct NameAndType(Utf8, Utf8);

pub struct MethodRef(Class, NameAndType);

pub enum PoolItem {
    Utf8(Utf8),
    String(JString),
    Class(Class),
    NameAndType(NameAndType),
    MethodRef(MethodRef),
}

#[derive(Eq, PartialEq, Hash)]
enum ConstantPoolItem {
    Utf8(String),
    String(u16),
    ClassRef(u16),
    NameAndType { name: u16, descriptor: u16 },
    MethodRef { class_ref: u16, name_and_type: u16 },
}

pub struct ConstantPool {
    pool: HashMap<ConstantPoolItem, u16>,
    next_index: u16,
}

impl ConstantPool {
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
            next_index: 1,
        }
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    fn get_or_insert(&mut self, item: ConstantPoolItem) -> u16 {
        match self.pool.get(&item) {
            Some(&index) => index,
            None => {
                let index = self.next_index;
                self.pool.insert(item, index);
                self.next_index += 1;
                index
            }
        }
    }

    pub fn add_item(&mut self, item: PoolItem) -> u16 {
        match item {
            PoolItem::Utf8(utf8) => self.get_or_insert(ConstantPoolItem::Utf8(utf8.0)),
            PoolItem::String(string) => {
                let index = self.add_item(PoolItem::Utf8(string.0));
                self.get_or_insert(ConstantPoolItem::String(index))
            }
            PoolItem::Class(class) => {
                let index = self.add_item(PoolItem::Utf8(class.0));
                self.get_or_insert(ConstantPoolItem::ClassRef(index))
            }
            PoolItem::NameAndType(name_and_type) => {
                let index1 = self.add_item(PoolItem::Utf8(name_and_type.0));
                let index2 = self.add_item(PoolItem::Utf8(name_and_type.1));
                self.get_or_insert(ConstantPoolItem::NameAndType {
                    name: index1,
                    descriptor: index2,
                })
            }
            PoolItem::MethodRef(method_ref) => {
                let index1 = self.add_item(PoolItem::Class(method_ref.0));
                let index2 = self.add_item(PoolItem::NameAndType(method_ref.1));
                self.get_or_insert(ConstantPoolItem::MethodRef {
                    class_ref: index1,
                    name_and_type: index2,
                })
            }
        }
    }

    pub fn add_utf8(&mut self, utf8: String) -> u16 {
        self.add_item(PoolItem::Utf8(Utf8(utf8)))
    }

    pub fn add_string(&mut self, string: String) -> u16 {
        self.add_item(PoolItem::String(JString(Utf8(string))))
    }

    pub fn add_class(&mut self, class: String) -> u16 {
        self.add_item(PoolItem::Class(Class(Utf8(class))))
    }

    pub fn add_method(&mut self, class: String, method: String, descriptor: String) -> u16 {
        self.add_item(PoolItem::MethodRef(MethodRef(
            Class(Utf8(class)),
            NameAndType(Utf8(method), Utf8(descriptor)),
        )))
    }

    pub fn write_to_file(&self, file: &mut File) -> io::Result<()> {
        let mut table = Vec::<&ConstantPoolItem>::new();
        table.resize_with(self.pool.len(), || &ConstantPoolItem::String(0)); // Placeholder value

        for (item, &index) in &self.pool {
            table[index as usize - 1] = item;
        }

        for item in table {
            match item {
                ConstantPoolItem::Utf8(string) => {
                    file.write_u8(1)?;
                    file.write_u16::<BigEndian>(string.as_bytes().len() as u16)?;
                    file.write_all(string.as_bytes())?;
                }
                &ConstantPoolItem::String(index) => {
                    file.write_u8(8)?;
                    file.write_u16::<BigEndian>(index)?;
                }
                &ConstantPoolItem::ClassRef(index) => {
                    file.write_u8(7)?;
                    file.write_u16::<BigEndian>(index)?;
                }
                &ConstantPoolItem::NameAndType { name, descriptor } => {
                    file.write_u8(12)?;
                    file.write_u16::<BigEndian>(name)?;
                    file.write_u16::<BigEndian>(descriptor)?;
                }
                &ConstantPoolItem::MethodRef { class_ref, name_and_type } => {
                    file.write_u8(10)?;
                    file.write_u16::<BigEndian>(class_ref)?;
                    file.write_u16::<BigEndian>(name_and_type)?;
                }
            }
        }
        Ok(())
    }
}
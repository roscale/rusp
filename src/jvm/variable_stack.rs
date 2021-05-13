use std::collections::HashMap;

use crate::jvm::jvm_type::JvmType;

pub struct VariableStack {
    indices: HashMap<String, (u8, JvmType)>,
    next_index: u8,
}

impl VariableStack {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
            next_index: 0,
        }
    }

    pub fn get(&mut self, name: &String) -> Option<(u8, JvmType)> {
        self.indices.get(name).cloned()
    }

    pub fn create(&mut self, name: String, jvm_type: JvmType) -> u8 {
        let index = self.next_index;
        self.indices.insert(name, (index, jvm_type));
        self.next_index += 1;
        index
    }

    pub fn drop(&mut self, name: &str) {
        self.indices.remove(name);
    }
}
use std::collections::HashMap;

pub struct VariableStack {
    indices: HashMap<String, u8>,
    next_index: u8,
}

impl VariableStack {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
            next_index: 0,
        }
    }

    pub fn get(&mut self, name: String) -> u8 {
        match self.indices.get(&name) {
            Some(&index) => index,
            None => {
                let index = self.next_index;
                self.indices.insert(name, index);
                self.next_index += 1;
                index
            }
        }
    }
}
use crate::jvm::bytecode::Label;

pub struct LabelGenerator {
    next_available_label: Label,
}

impl LabelGenerator {
    pub fn new() -> Self {
        LabelGenerator {
            next_available_label: 0,
        }
    }

    pub fn get_new_label(&mut self) -> Label {
        let label = self.next_available_label;
        self.next_available_label += 1;
        label
    }
}
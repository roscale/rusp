#![feature(or_patterns)]

use std::fs::File;
use std::io::Read;
use crate::tokenizer::tokenize;

mod tokenizer;

fn main() {
    let source = {
        let mut file = File::open("test.py").unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        source
    };

    let tokens = tokenize(&source);
    println!("{:?}", tokens)
}

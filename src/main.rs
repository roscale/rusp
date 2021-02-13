#![feature(or_patterns)]

use std::fs::File;
use std::io::Read;

use crate::lexer::Lexer;

mod lexer;

fn main() {
    let source = {
        let mut file = File::open("test.py").unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        source
    };

    let chars = source.chars().collect::<Vec<_>>();
    let mut lexer = Lexer::new(chars.as_slice());
    let result = lexer.tokenize();

    match result {
        Ok(tokens) => {
            println!("{:?}", &tokens);
        }
        Err(e) => {
            dbg!(e);
        }
    }
}

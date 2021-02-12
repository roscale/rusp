#![feature(or_patterns)]

use std::fs::File;
use std::io::Read;

use crate::lexer::Lexer;
use crate::lexer2::Lexer2;

mod lexer;
mod lexer2;
mod util;

fn main() {
    let source = {
        let mut file = File::open("test.py").unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        source
    };

    // let tokens = Lexer::new(&source).tokenize();

    let chars = source.chars().collect::<Vec<_>>();
    let mut lexer = Lexer2::new(chars.as_slice());
    let result = lexer.tokenize();

    match result {
        Ok(tokens) => {
            println!("{:?}", &tokens);
        }
        Err(e) => {
            dbg!(e);
        }
    }

    // println!("{:?}", tokens)
}

#![feature(or_patterns)]

use std::fs::File;
use std::io::Read;

use crate::lexer::Lexer;
use crate::parser::{Parser, Value};

mod lexer;
mod parser;

fn main() {
    let source = {
        let mut file = File::open("test.rkt").unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        source
    };

    let chars = source.chars().collect::<Vec<_>>();
    let mut lexer = Lexer::new(chars.as_slice());
    let result = lexer.tokenize();

    match result {
        Ok(tokens) => {
            // dbg!(&tokens);

            let parser = Parser::new(&tokens);
            let ast = parser.parse();
            match ast {
                Ok(ast) => {
                    // dbg!(&ast);
                    match ast.first().unwrap().call(&[&Value::Float(10.0)]) {
                        Ok(value) => println!("{}", value),
                        Err(err) => {
                            dbg!(&err);
                        },
                    };
                },
                Err(err) => {
                    dbg!(&err);
                },
            };
        }
        Err(e) => {
            dbg!(e);
        }
    }
}

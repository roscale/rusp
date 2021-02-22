#![feature(or_patterns)]

use std::fs::File;
use std::io::Read;

use crate::lexer::Lexer;
use crate::parser::Parser;

mod lexer;
mod parser;
mod interpreter;

fn main() {
    let source = {
        let mut file = File::open("test.rkt").unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        source
    };

    let chars = source.chars().collect::<Vec<_>>();
    let tokens = Lexer::new(chars.as_slice()).tokenize();

    match tokens {
        Ok(tokens) => {
            let global_context = Parser::new(&tokens).parse();
            match global_context {
                Ok(context) => {
                    match context.functions.get("main").unwrap().call(&context, vec![]) {
                        Ok(value) => println!("{}", value),
                        Err(err) => {
                            dbg!(&err);
                        },
                    };
                }
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

#![feature(or_patterns)]

use std::fs::File;
use std::io::Read;

use crate::lexer::Lexer;
use crate::parser::{Parser, Context, Value};

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
            let expressions = Parser::new(&tokens).parse();
            match expressions {
                Ok(expressions) => {
                    let mut global_context = Context::default();
                    let result = expressions.iter().fold(Ok(Value::Unit), |acc, expression| {
                        acc.and(expression.evaluate(&mut global_context))
                    });

                    match result {
                        Ok(result) => println!("{}", result),
                        Err(e) => {
                            dbg!(e);
                        },
                    }
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

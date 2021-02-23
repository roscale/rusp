#![feature(or_patterns)]
#![feature(box_patterns)]

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use crate::lexer::Lexer;
use crate::parser::{Context, Parser, Value};

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
                    let global_context = Rc::new(RefCell::new(Context::default()));
                    let result = expressions.iter().fold(Ok(Value::Unit), |acc, expression| {
                        acc.and(expression.evaluate(global_context.clone()))
                    });

                    match result {
                        Ok(result) => println!("{}", result),
                        Err(e) => {
                            dbg!(e);
                        }
                    }
                }
                Err(err) => {
                    dbg!(&err);
                }
            };
        }
        Err(e) => {
            dbg!(e);
        }
    }
}

#![feature(or_patterns)]
#![feature(box_patterns)]
#![feature(try_blocks)]
#![feature(exact_size_is_empty)]

use std::{env, process};
use std::fs::File;
use std::io::Read;

use crate::built_in_functions::create_global_context_with_built_in_functions;
use crate::interpreter::InterpreterError;
use crate::lexer::{Lexer, LexerError};
use crate::parser::{Parser, ParserError};

mod lexer;
mod parser;
mod interpreter;
mod built_in_functions;

fn main() -> Result<(), AllErrors> {
    let mut args = env::args();
    let path = args.next().unwrap();

    let script_path = match args.next() {
        Some(path) => path,
        None => {
            println!("TODO: REPL");
            println!("Usage: {} <file>", path);
            return Ok(());
        }
    };

    let source = {
        let mut file = match File::open(&script_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1);
            }
        };
        let mut source = String::new();
        if let Err(err) = file.read_to_string(&mut source) {
            eprintln!("{}", err);
            process::exit(2);
        }
        source
    };

    let tokens = {
        let chars = source.chars().collect::<Vec<_>>();
        Lexer::new(chars.as_slice()).tokenize()?
    };

    let expressions = Parser::new(&tokens).parse()?;
    let global_context = create_global_context_with_built_in_functions();

    for expression in &expressions {
        expression.evaluate(global_context.clone())?;
    }

    Ok(())
}

#[derive(Debug)]
enum AllErrors {
    LexerError(LexerError),
    ParserError(ParserError),
    InterpreterError(InterpreterError),
}

impl From<LexerError> for AllErrors {
    fn from(e: LexerError) -> Self { Self::LexerError(e) }
}

impl From<ParserError> for AllErrors {
    fn from(e: ParserError) -> Self { Self::ParserError(e) }
}

impl From<InterpreterError> for AllErrors {
    fn from(e: InterpreterError) -> Self { Self::InterpreterError(e) }
}
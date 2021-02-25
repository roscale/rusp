#![feature(or_patterns)]
#![feature(box_patterns)]
#![feature(try_blocks)]

use std::fs::File;
use std::io::Read;

use crate::interpreter::InterpreterError;
use crate::lexer::{Lexer, LexerError};
use crate::parser::{Parser, ParserError};
use crate::built_in_functions::create_global_context_with_built_in_functions;

mod lexer;
mod parser;
mod interpreter;
mod built_in_functions;

fn main() -> Result<(), AllErrors> {
    let source = {
        let mut file = File::open("examples.rsp").unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
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
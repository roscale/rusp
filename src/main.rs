#![feature(or_patterns)]
#![feature(box_patterns)]
#![feature(try_blocks)]
#![feature(exact_size_is_empty)]

use std::{env, process};
use std::fs::File;
use std::io::Read;

use codespan_reporting::files::SimpleFiles;

use crate::native_functions::create_global_context_with_native_functions;
use crate::interpreter::{InterpreterError, InterpreterErrorWithSpan};
use crate::lexer::{Lexer, LexerError};
use crate::parser::{Parser, ParserError};
use crate::errors::{show_lexer_error, show_parser_error, show_interpreter_error};

mod lexer;
mod parser;
mod interpreter;
mod native_functions;
mod errors;

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


    let mut files = SimpleFiles::new();
    let source_file = files.add(script_path, &source);

    let tokens_with_metadata = {
        let chars = source.chars().collect::<Vec<_>>();
        Lexer::new(chars.as_slice()).tokenize()
    };

    let tokens_with_metadata = match tokens_with_metadata {
        Ok(t) => t,
        Err(err) => {
            show_lexer_error(err, source_file, files);
            return Ok(());
        }
    };

    let expressions = Parser::new((tokens_with_metadata.0.as_slice(), tokens_with_metadata.1.as_slice())).parse();
    let expressions = match expressions {
        Ok(e) => e,
        Err(err) => {
            show_parser_error(err, source_file, files);
            return Ok(());
        }
    };

    let global_context = create_global_context_with_native_functions();

    let result: Result<(), InterpreterErrorWithSpan> = try {
        for expression in &expressions {
            expression.evaluate(global_context.clone())?;
        }
    };

    if let Err(err) = result {
        show_interpreter_error(err, source_file, files);
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
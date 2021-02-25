#![feature(or_patterns)]
#![feature(box_patterns)]
#![feature(try_blocks)]

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use crate::interpreter::InterpreterError;
use crate::lexer::{Lexer, LexerError};
use crate::parser::{Context, Function, Parser, ParserError, Value};

mod lexer;
mod parser;
mod interpreter;

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
    let global_context = create_global_context();

    for expression in &expressions {
        expression.evaluate(global_context.clone())?;
    }

    Ok(())
}

fn create_global_context() -> Rc<RefCell<Context>> {
    let global_context = Rc::new(RefCell::new(Context::default()));

    global_context.borrow_mut().variables.insert(String::from("print"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "print".to_string(),
        parameters: vec!["value".to_string()],
        fn_pointer: |_context, arguments| {
            print!("{}", arguments[0]);
            Ok(Value::Unit)
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("println"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "println".to_string(),
        parameters: vec!["value".to_string()],
        fn_pointer: |_context, arguments| {
            println!("{}", arguments[0]);
            Ok(Value::Unit)
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("dbg"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "dbg".to_string(),
        parameters: vec!["value".to_string()],
        fn_pointer: |_context, arguments| {
            println!("{:#?}", &arguments[0]);
            Ok(Value::Unit)
        },
    }));

    global_context
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
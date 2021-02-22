use std::collections::HashMap;

use ParserError::*;

use crate::lexer::{Keyword, Literal, Operator, Token};
use crate::parser::Expression::Scope;

#[derive(Default, Debug)]
pub struct Context<'p, 'k, 'v> {
    pub parent_context: Option<&'p Context<'p, 'k, 'v>>,
    pub functions: HashMap<&'k str, Function<'v, 'v>>,
    pub variables: HashMap<&'k str, Value>,
}

#[derive(Debug)]
pub enum Expression {
    Id(String),
    Value(Value),
    Operation(Operator, Vec<Expression>),
    Scope(Vec<Expression>),
    FunctionCall(String, Vec<Expression>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Integer(i32),
    Float(f32),
    String(String),
    Boolean(bool),
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token),
    UnexpectedEOF,
}

#[derive(Debug)]
pub struct Function<'n, 'p> {
    pub(crate) name: &'n str,
    pub(crate) parameters: Vec<&'p str>,
    pub(crate) body: Expression,
}

pub struct Parser<'a> {
    view: &'a [Token],
    global_context: Context<'a, 'a, 'a>,
}

impl<'a> Parser<'a> {
    pub fn new(view: &'a [Token]) -> Self {
        Self {
            view,
            global_context: Context::default(),
        }
    }

    pub fn parse(mut self) -> Result<Context<'a, 'a, 'a>, ParserError> {
        loop {
            match self.view {
                [Token::LeftParenthesis, Token::Keyword(Keyword::Fn), ..] => {
                    let function = self.parse_function()?;
                    self.global_context.functions.insert(function.name, function);
                }
                [] => break,
                [token, ..] => return Err(UnexpectedToken(token.clone())),
            }
        }
        Ok(self.global_context)
    }

    pub fn parse_function(&mut self) -> Result<Function<'a, 'a>, ParserError> {
        self.view = &self.view[2..]; // skip "(fn"

        let name = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => id,
            t => return Err(UnexpectedToken(t.to_owned()))
        };
        self.view = &self.view[1..];

        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::LeftParenthesis => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let mut parameters = Vec::new();
        loop {
            match self.view.first().ok_or(UnexpectedEOF)? {
                Token::Id(id) => {
                    parameters.push(id as &str);
                    self.view = &self.view[1..];
                }
                Token::RightParenthesis => {
                    self.view = &self.view[1..];
                    break;
                }
                t => return Err(UnexpectedToken(t.to_owned())),
            }
        }

        let body = self.parse_expression()?;

        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::RightParenthesis => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        Ok(Function {
            name,
            parameters,
            body,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let expression = match self.view {
            [Token::Id(id), ..] => {
                self.view = &self.view[1..];
                Expression::Id(id.to_owned())
            }
            [Token::Literal(l), ..] => {
                self.view = &self.view[1..];
                match l {
                    Literal::Integer(i) => Expression::Value(Value::Integer(*i)),
                    Literal::Float(f) => Expression::Value(Value::Float(*f)),
                    Literal::String(s) => Expression::Value(Value::String(s.to_owned())),
                }
            }
            [Token::LeftParenthesis, Token::Operator(_), ..] => self.parse_operation()?,
            [Token::LeftParenthesis, Token::Id(_), ..] => self.parse_function_call()?,
            [Token::LeftBrace, ..] => self.parse_scope()?,
            [t, ..] => return Err(UnexpectedToken(t.to_owned())),
            [] => return Err(UnexpectedEOF),
        };
        Ok(expression)
    }

    fn parse_operation(&mut self) -> Result<Expression, ParserError> {
        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::LeftParenthesis => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let operator = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Operator(op) => op.clone(),
            t => return Err(UnexpectedToken(t.to_owned()))
        };
        self.view = &self.view[1..];

        let mut arguments = Vec::new();
        loop {
            match self.view.first().ok_or(UnexpectedEOF)? {
                Token::RightParenthesis => {
                    self.view = &self.view[1..];
                    break;
                }
                _ => {
                    arguments.push(self.parse_expression()?)
                }
            }
        }

        Ok(Expression::Operation(operator, arguments))
    }

    fn parse_function_call(&mut self) -> Result<Expression, ParserError> {
        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::LeftParenthesis => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let name = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => id.to_owned(),
            t => return Err(UnexpectedToken(t.to_owned()))
        };
        self.view = &self.view[1..];

        let mut arguments = Vec::new();
        loop {
            match self.view.first().ok_or(UnexpectedEOF)? {
                Token::RightParenthesis => {
                    self.view = &self.view[1..];
                    break;
                }
                _ => {
                    arguments.push(self.parse_expression()?)
                }
            }
        }

        Ok(Expression::FunctionCall(name, arguments))
    }

    fn parse_scope(&mut self) -> Result<Expression, ParserError> {
        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::LeftBrace => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let mut expressions = vec![];
        loop {
            match self.view.first().ok_or(UnexpectedEOF)? {
                Token::RightBrace => {
                    self.view = &self.view[1..];
                    break;
                }
                _ => expressions.push(self.parse_expression()?)
            }
        }
        Ok(Scope(expressions))
    }
}
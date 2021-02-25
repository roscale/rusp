/// Same architecture as the lexer.
/// It outputs a vector of Expressions to be evaluated by the interpreter.
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use ParserError::*;

use crate::interpreter::InterpreterError;
use crate::lexer::{Keyword, Literal, Token};
use crate::parser::Expression::Scope;

#[derive(Default, Debug)]
pub struct Context {
    pub parent_context: Option<Rc<RefCell<Context>>>,
    pub variables: HashMap<String, Value>,
}

impl Context {
    pub fn with_parent(parent_context: Rc<RefCell<Context>>) -> Self {
        Self {
            parent_context: Some(parent_context),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Id(String),
    Value(Value),
    Declaration(String, Box<Expression>),
    Assignment(String, Box<Expression>),
    Scope(Vec<Expression>),
    NamedFunctionDefinition {
        name: String,
        parameters: Vec<String>,
        body: Box<Expression>,
    },
    AnonymousFunctionDefinition {
        parameters: Vec<String>,
        body: Box<Expression>,
    },
    FunctionCall(Box<Expression>, Vec<Expression>),
    If {
        guard: Box<Expression>,
        base_case: Box<Expression>,
    },
    IfElse {
        guard: Box<Expression>,
        base_case: Box<Expression>,
        else_case: Box<Expression>,
    },
    While {
        guard: Box<Expression>,
        body: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Integer(i32),
    Float(f32),
    String(String),
    Boolean(bool),
    Function(Function),
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token),
    UnexpectedEOF,
}

#[derive(Debug, Clone)]
pub enum Function {
    BuiltInFunction {
        closing_context: Rc<RefCell<Context>>,
        name: String,
        fn_pointer: fn(Rc<RefCell<Context>>, Vec<Value>) -> Result<Value, InterpreterError>,
    },
    LanguageFunction {
        closing_context: Rc<RefCell<Context>>,
        name: String,
        parameters: Vec<String>,
        body: Box<Expression>,
    },
}

pub struct Parser<'a> {
    view: &'a [Token],
}

impl<'a> Parser<'a> {
    pub fn new(view: &'a [Token]) -> Self {
        Self {
            view,
        }
    }

    pub fn parse(mut self) -> Result<Vec<Expression>, ParserError> {
        let mut expressions = vec![];
        while !self.view.is_empty() {
            expressions.push(self.parse_expression()?);
        }
        Ok(expressions)
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let expression = match self.view {
            [Token::Id(_), Token::Equal, ..] => self.parse_assignment()?,
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
            [Token::Keyword(Keyword::True), ..] => {
                self.view = &self.view[1..];
                Expression::Value(Value::Boolean(true))
            }
            [Token::Keyword(Keyword::False), ..] => {
                self.view = &self.view[1..];
                Expression::Value(Value::Boolean(false))
            }
            // [Token::LeftParenthesis, Token::Operator(_), ..] => self.parse_operation()?,
            [Token::LeftParenthesis, _, ..] => self.parse_function_call()?,
            [Token::LeftBrace, ..] => self.parse_scope()?,
            [Token::Keyword(Keyword::Fn), ..] => self.parse_function()?,
            [Token::Keyword(Keyword::Let), ..] => self.parse_declaration()?,
            [Token::Keyword(Keyword::If), ..] => self.parse_condition()?,
            [Token::Keyword(Keyword::While), ..] => self.parse_while_loop()?,
            [t, ..] => return Err(UnexpectedToken(t.to_owned())),
            [] => return Err(UnexpectedEOF),
        };
        Ok(expression)
    }

    pub fn parse_function(&mut self) -> Result<Expression, ParserError> {
        self.view = &self.view[1..]; // skip "fn"

        // If there's no name, then it's an anonymous function
        let name = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => {
                self.view = &self.view[1..];
                Some(id)
            }
            _ => None
        };

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
        let parameters = parameters.into_iter().map(|s| s.to_owned()).collect();

        let body = Box::new(self.parse_expression()?);

        Ok(match name {
            Some(name) => Expression::NamedFunctionDefinition {
                name: name.to_owned(),
                parameters,
                body,
            },
            None => Expression::AnonymousFunctionDefinition {
                parameters,
                body,
            },
        })
    }

    fn parse_declaration(&mut self) -> Result<Expression, ParserError> {
        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Keyword(Keyword::Let) => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let name = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => id,
            t => return Err(UnexpectedToken(t.to_owned())),
        };
        self.view = &self.view[1..];

        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Equal => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let rhs = self.parse_expression()?;

        Ok(Expression::Declaration(name.to_owned(), Box::new(rhs)))
    }

    fn parse_assignment(&mut self) -> Result<Expression, ParserError> {
        let name = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => id,
            t => return Err(UnexpectedToken(t.to_owned())),
        };
        self.view = &self.view[1..];

        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Equal => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let rhs = self.parse_expression()?;

        Ok(Expression::Assignment(name.to_owned(), Box::new(rhs)))
    }

    fn parse_function_call(&mut self) -> Result<Expression, ParserError> {
        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::LeftParenthesis => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let function_ptr = self.parse_expression()?;

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

        Ok(Expression::FunctionCall(Box::new(function_ptr), arguments))
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

    fn parse_condition(&mut self) -> Result<Expression, ParserError> {
        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Keyword(Keyword::If) => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let guard = self.parse_expression()?;
        let base_case = self.parse_expression()?;

        let else_guard_exists = match self.view.first() {
            Some(Token::Keyword(Keyword::Else)) => {
                self.view = &self.view[1..];
                true
            }
            _ => false,
        };

        match else_guard_exists {
            false => {
                Ok(Expression::If {
                    guard: Box::new(guard),
                    base_case: Box::new(base_case),
                })
            }
            true => {
                let else_case = self.parse_expression()?;

                Ok(Expression::IfElse {
                    guard: Box::new(guard),
                    base_case: Box::new(base_case),
                    else_case: Box::new(else_case),
                })
            }
        }
    }

    fn parse_while_loop(&mut self) -> Result<Expression, ParserError> {
        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Keyword(Keyword::While) => (),
            t => return Err(UnexpectedToken(t.to_owned())),
        }
        self.view = &self.view[1..];

        let guard = self.parse_expression()?;
        let body = self.parse_expression()?;

        Ok(Expression::While {
            guard: Box::new(guard),
            body: Box::new(body),
        })
    }
}
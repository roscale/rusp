/// Same architecture as the lexer.
/// It outputs a vector of Expressions to be evaluated by the interpreter.
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use ParserError::*;

use crate::interpreter::InterpreterErrorWithSpan;
use crate::lexer::{Keyword, Literal, Token};

#[derive(Default, Debug)]
pub struct Context {
    pub parent_context: Option<Rc<RefCell<Context>>>,
    pub variables: HashMap<String, Rc<RefCell<Value>>>,
}

impl Context {
    pub fn with_parent(parent_context: Rc<RefCell<Context>>) -> Self {
        Self {
            parent_context: Some(parent_context),
            ..Default::default()
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<Rc<RefCell<Value>>> {
        match self.variables.get(name) {
            None => self.parent_context.as_ref().and_then(|p| p.borrow().get_variable(name)),
            Some(value) => Some(value.clone())
        }
    }

    pub fn set_variable(&mut self, name: &str, new_value: Rc<RefCell<Value>>) -> Result<(), ()> {
        match self.variables.get_mut(name) {
            None => self.parent_context.as_ref().ok_or(()).and_then(|p| p.borrow_mut().set_variable(name, new_value)),
            Some(value) => {
                *value = new_value;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionWithMetadata {
    pub expression: Expression,
    pub span: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub label: String,
    pub span: Range<usize>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Id(String),
    Value(Value),
    Declaration(Label, Box<ExpressionWithMetadata>),
    Assignment(Label, Box<ExpressionWithMetadata>),
    Scope(Vec<ExpressionWithMetadata>),
    NamedFunctionDefinition {
        name: Label,
        parameters: Vec<Label>,
        body: Box<ExpressionWithMetadata>,
    },
    AnonymousFunctionDefinition {
        parameters: Vec<Label>,
        body: Box<ExpressionWithMetadata>,
    },
    FunctionCall(Box<ExpressionWithMetadata>, Vec<ExpressionWithMetadata>),
    If {
        guard: Box<ExpressionWithMetadata>,
        base_case: Box<ExpressionWithMetadata>,
    },
    IfElse {
        guard: Box<ExpressionWithMetadata>,
        base_case: Box<ExpressionWithMetadata>,
        else_case: Box<ExpressionWithMetadata>,
    },
    While {
        guard: Box<ExpressionWithMetadata>,
        body: Box<ExpressionWithMetadata>,
    },
    List(Vec<ExpressionWithMetadata>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Integer(i32),
    Float(f32),
    String(String),
    Boolean(bool),
    Function(Function),
    List(Vec<Rc<RefCell<Value>>>),
}


impl Value {
    pub fn unit() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Value::Unit))
    }
}

pub trait IntoSharedRef {
    fn into_shared_ref(self) -> Rc<RefCell<Self>> where Self: Sized {
        Rc::new(RefCell::new(self))
    }
}

impl IntoSharedRef for Value {}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Range<usize>),
    UnexpectedEOF,
}

#[derive(Debug, Clone)]
pub enum Function {
    NativeFunction {
        closing_context: Rc<RefCell<Context>>,
        name: String,
        fn_pointer: fn(Rc<RefCell<Context>>, Vec<Rc<RefCell<Value>>>) -> Result<Rc<RefCell<Value>>, InterpreterErrorWithSpan>,
    },
    RuspFunction {
        closing_context: Rc<RefCell<Context>>,
        name: String,
        parameters: Vec<String>,
        body: Box<ExpressionWithMetadata>,
    },
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    token_indices: &'a [Range<usize>],
    utf8_start_index: usize,
    utf8_end_index: usize,
}

impl<'a> Parser<'a> {
    pub fn new((tokens, indices): (&'a [Token], &'a [Range<usize>])) -> Self {
        Self {
            tokens,
            token_indices: indices,
            utf8_start_index: indices.first().map_or(0, |r| r.start),
            utf8_end_index: indices.first().map_or(0, |r| r.end),
        }
    }

    pub fn advance_by(&mut self, n: usize) {
        // We can't get the nth element at the end of the file.
        self.utf8_start_index = if let Some(span) = self.token_indices.get(n) {
            span.start
        } else {
            self.token_indices[n - 1].end
        };
        self.utf8_end_index = self.token_indices[n - 1].end;
        self.tokens = &self.tokens[n..];
        self.token_indices = &self.token_indices[n..];
    }

    pub fn parse(mut self) -> Result<Vec<ExpressionWithMetadata>, ParserError> {
        let mut expressions = vec![];

        while !self.tokens.is_empty() {
            expressions.push(self.parse_expression()?);
        }
        Ok(expressions)
    }

    fn parse_expression(&mut self) -> Result<ExpressionWithMetadata, ParserError> {
        let start_index = self.utf8_start_index;

        let expression = match self.tokens {
            [Token::Id(_), Token::Equal, ..] => self.parse_assignment()?,
            [Token::Id(id), ..] => {
                self.advance_by(1);
                Expression::Id(id.to_owned())
            }
            [Token::Literal(l), ..] => {
                self.advance_by(1);
                match l {
                    Literal::Integer(i) => Expression::Value(Value::Integer(*i)),
                    Literal::Float(f) => Expression::Value(Value::Float(*f)),
                    Literal::String(s) => Expression::Value(Value::String(s.to_owned())),
                }
            }
            [Token::Keyword(Keyword::True), ..] => {
                self.advance_by(1);
                Expression::Value(Value::Boolean(true))
            }
            [Token::Keyword(Keyword::False), ..] => {
                self.advance_by(1);
                Expression::Value(Value::Boolean(false))
            }
            [Token::LeftParenthesis, _, ..] => self.parse_function_call()?,
            [Token::LeftSquareBracket, ..] => self.parse_list()?,
            [Token::LeftBrace, ..] => self.parse_scope()?,
            [Token::Keyword(Keyword::Fn), ..] => self.parse_function()?,
            [Token::Keyword(Keyword::Let), ..] => self.parse_declaration()?,
            [Token::Keyword(Keyword::If), ..] => self.parse_condition()?,
            [Token::Keyword(Keyword::While), ..] => self.parse_while_loop()?,
            [_, ..] => return Err(UnexpectedToken(self.token_indices[0].clone())),
            [] => return Err(UnexpectedEOF),
        };
        Ok(ExpressionWithMetadata {
            expression,
            span: start_index..self.utf8_end_index,
        })
    }

    pub fn parse_function(&mut self) -> Result<Expression, ParserError> {
        self.advance_by(1); // skip "fn"

        // If there's no name, then it's an anonymous function
        let name_start_index = self.utf8_start_index;
        let name = match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => {
                self.advance_by(1);
                Some(id)
            }
            _ => None
        };
        let name_end_index = self.utf8_end_index;

        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::LeftParenthesis => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let mut parameters = Vec::new();
        loop {
            match self.tokens.first().ok_or(UnexpectedEOF)? {
                Token::Id(id) => {
                    let start_index = self.utf8_start_index;
                    self.advance_by(1);
                    let end_index = self.utf8_end_index;

                    parameters.push(Label {
                        label: id.to_owned(),
                        span: start_index..end_index,
                    });
                }
                Token::RightParenthesis => {
                    self.advance_by(1);
                    break;
                }
                _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
            }
        }
        let body = Box::new(self.parse_expression()?);

        Ok(match name {
            Some(name) => Expression::NamedFunctionDefinition {
                name: Label {
                    label: name.to_owned(),
                    span: name_start_index..name_end_index,
                },
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
        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::Keyword(Keyword::Let) => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let name_start_index = self.utf8_start_index;
        let name = match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => id,
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        };
        self.advance_by(1);
        let name_end_index = self.utf8_end_index;

        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::Equal => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let rhs = self.parse_expression()?;

        Ok(Expression::Declaration(Label {
            label: name.to_owned(),
            span: name_start_index..name_end_index,
        }, Box::new(rhs)))
    }

    fn parse_assignment(&mut self) -> Result<Expression, ParserError> {
        let name_start_index = self.utf8_start_index;
        let name = match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => id,
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        };
        self.advance_by(1);
        let name_end_index = self.utf8_end_index;

        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::Equal => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let rhs = self.parse_expression()?;

        Ok(Expression::Assignment(Label {
            label: name.to_owned(),
            span: name_start_index..name_end_index,
        }, Box::new(rhs)))
    }

    fn parse_function_call(&mut self) -> Result<Expression, ParserError> {
        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::LeftParenthesis => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let function_ptr = self.parse_expression()?;

        let mut arguments = Vec::new();
        loop {
            match self.tokens.first().ok_or(UnexpectedEOF)? {
                Token::RightParenthesis => {
                    self.advance_by(1);
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
        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::LeftBrace => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let mut expressions = vec![];
        loop {
            match self.tokens.first().ok_or(UnexpectedEOF)? {
                Token::RightBrace => {
                    self.advance_by(1);
                    break;
                }
                _ => expressions.push(self.parse_expression()?)
            }
        }
        Ok(Expression::Scope(expressions))
    }

    fn parse_condition(&mut self) -> Result<Expression, ParserError> {
        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::Keyword(Keyword::If) => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let guard = self.parse_expression()?;
        let base_case = self.parse_expression()?;

        let else_case_exists = match self.tokens.first() {
            Some(Token::Keyword(Keyword::Else)) => {
                self.advance_by(1);
                true
            }
            _ => false,
        };

        match else_case_exists {
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
        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::Keyword(Keyword::While) => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let guard = self.parse_expression()?;
        let body = self.parse_expression()?;

        Ok(Expression::While {
            guard: Box::new(guard),
            body: Box::new(body),
        })
    }

    fn parse_list(&mut self) -> Result<Expression, ParserError> {
        match self.tokens.first().ok_or(UnexpectedEOF)? {
            Token::LeftSquareBracket => (),
            _ => return Err(UnexpectedToken(self.token_indices[0].clone())),
        }
        self.advance_by(1);

        let mut elements = Vec::new();
        loop {
            match self.tokens.first().ok_or(UnexpectedEOF)? {
                Token::RightSquareBracket => {
                    self.advance_by(1);
                    break;
                }
                _ => {
                    elements.push(self.parse_expression()?)
                }
            }
        }
        Ok(Expression::List(elements))
    }
}
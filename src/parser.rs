use std::collections::HashMap;

use ParserError::*;

use crate::lexer::{Keyword, Literal, Operator, Token};
use crate::parser::ParserError::VariableNotFound;
use std::fmt::{Display, Formatter};

struct Context<'k, 'v> {
    variables: HashMap<&'k str, &'v Value>,
}

#[derive(Debug)]
enum Expression {
    Id(String),
    Value(Value),
    Operation(Operator, Vec<Expression>),
    FunctionCall(String, Vec<Expression>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i32),
    Float(f32),
    String(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(int) => write!(f, "{}", int),
            Value::Float(float) => write!(f, "{}", float),
            Value::String(string) => write!(f, "{}", string),
        }
    }
}

#[derive(Debug)]
pub enum ParserError<'a> {
    VariableNotFound(&'a str),
    WrongNumberOfArguments,
    UnexpectedToken(&'a Token),
    UnexpectedEOF,
    InvalidOperands,
}

impl Expression {
    fn evaluate<'a>(&'a self, context: &Context) -> Result<Value, ParserError<'a>> {
        match self {
            Expression::Id(id) => context.variables.get(id as &str).cloned().cloned().ok_or(VariableNotFound(id)),
            Expression::Value(value) => Ok(value.clone()),
            Expression::Operation(op, operands) => {
                let mut values = vec![];
                for op in operands {
                    values.push(op.evaluate(context)?);
                }

                use Value::*;
                match op {
                    Operator::Plus => Ok(values.into_iter().fold(Value::Integer(0), |acc, x| {
                        match (acc, x) {
                            (String(lhs), String(rhs)) => String(format!("{}{}", lhs, rhs)),
                            (String(lhs), Integer(rhs)) => String(format!("{}{}", lhs, rhs)),
                            (String(lhs), Float(rhs)) => String(format!("{}{}", lhs, rhs)),
                            (Integer(lhs), String(rhs)) => String(format!("{}{}", lhs, rhs)),
                            (Integer(lhs), Integer(rhs)) => Integer(lhs + rhs),
                            (Integer(lhs), Float(rhs)) => Float(lhs as f32 + rhs),
                            (Float(lhs), String(rhs)) => String(format!("{}{}", lhs, rhs)),
                            (Float(lhs), Integer(rhs)) => Float(lhs + rhs as f32),
                            (Float(lhs), Float(rhs)) => Float(lhs + rhs),
                        }
                    })),
                    _ => {
                        let mut iter = values.into_iter();
                        let first = iter.next().ok_or(WrongNumberOfArguments)?;
                        iter.fold(Ok(first), |acc, x| {
                            acc.and_then(|acc| {
                                fn compute_float_operation(lhs: f32, op: &Operator, rhs: f32) -> Value {
                                    match op {
                                        Operator::Plus => Float(lhs + rhs),
                                        Operator::Minus => Float(lhs - rhs),
                                        Operator::Asterisk => Float(lhs * rhs),
                                        Operator::Slash => Float(lhs / rhs),
                                        Operator::Pow => Float(lhs.powf(rhs)),
                                    }
                                }

                                match (acc, x) {
                                    (Integer(lhs), Integer(rhs)) => {
                                        match op {
                                            Operator::Minus => Ok(Integer(lhs - rhs)),
                                            Operator::Asterisk => Ok(Integer(lhs * rhs)),
                                            Operator::Slash => Ok(Integer(lhs / rhs)),
                                            Operator::Pow => Ok(Float((lhs as f32).powi(rhs))),
                                            Operator::Plus => Err(InvalidOperands),
                                        }
                                    },
                                    (Integer(lhs), Float(rhs)) => Ok(compute_float_operation(lhs as f32, op, rhs)),
                                    (Float(lhs), Integer(rhs)) => Ok(compute_float_operation(lhs, op, rhs as f32)),
                                    (Float(lhs), Float(rhs)) => Ok(compute_float_operation(lhs, op, rhs)),
                                    _ => Err(InvalidOperands)
                                }
                            })
                        })
                    }
                }
            }
            Expression::FunctionCall(name, arguments) => {
                let mut values = vec![];
                for arg in arguments {
                    values.push(arg.evaluate(context)?);
                }

                // TODO
                Ok((Value::Integer(42)))
            }
        }
    }
}

#[derive(Debug)]
pub struct Function<'n, 'p> {
    name: &'n str,
    parameters: Vec<&'p str>,
    body: Expression,
}

impl<'n, 'p> Function<'n, 'p> {
    pub fn call<'a>(&'a self, args: &[&Value]) -> Result<Value, ParserError<'a>> {
        if self.parameters.len() != args.len() {
            return Err(ParserError::WrongNumberOfArguments);
        }

        let context = Context {
            variables: {
                let mut hashmap = HashMap::<&str, &Value>::new();
                for (param, arg) in self.parameters.iter().zip(args) {
                    hashmap.insert(param, arg);
                }
                hashmap
            }
        };

        self.body.evaluate(&context)
    }
}

pub struct Parser<'a> {
    view: &'a [Token],
    functions: Vec<Function<'a, 'a>>,
}

type ParserResult<'a> = Result<Vec<Function<'a, 'a>>, ParserError<'a>>;

impl<'a> Parser<'a> {
    pub fn new(view: &'a [Token]) -> Self {
        Self {
            view,
            functions: vec![],
        }
    }

    pub fn parse(mut self) -> ParserResult<'a> {
        loop {
            match self.view {
                [Token::LeftParenthesis, Token::Keyword(Keyword::Fn), ..] => {
                    let function = self.parse_function()?;
                    self.functions.push(function);
                }
                [] => break,
                [token, ..] => return Err(UnexpectedToken(token)),
            }
        }
        Ok(self.functions)
    }

    pub fn parse_function(&mut self) -> Result<Function<'a, 'a>, ParserError<'a>> {
        self.view = &self.view[2..]; // skip "(fn"

        let name = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => id,
            t => return Err(UnexpectedToken(t))
        };
        self.view = &self.view[1..];

        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::LeftParenthesis => (),
            t => return Err(UnexpectedToken(t)),
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
                t => return Err(UnexpectedToken(t)),
            }
        }

        let body = self.parse_expression()?;

        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::RightParenthesis => (),
            t => return Err(UnexpectedToken(t)),
        }
        self.view = &self.view[1..];

        Ok(Function {
            name,
            parameters,
            body,
        })
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParserError<'a>> {
        let expression = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Id(id) => {
                self.view = &self.view[1..];
                Expression::Id(id.to_owned())
            },
            Token::Literal(l) => {
                self.view = &self.view[1..];
                match l {
                    Literal::Integer(i) => Expression::Value(Value::Integer(*i)),
                    Literal::Float(f) => Expression::Value(Value::Float(*f)),
                    Literal::String(s) => Expression::Value(Value::String(s.to_owned())),
                }
            }
            Token::LeftParenthesis => self.parse_operation()?,
            t => return Err(UnexpectedToken(t)),
        };
        Ok(expression)
    }

    pub fn parse_operation(&mut self) -> Result<Expression, ParserError<'a>> {
        match self.view.first().ok_or(UnexpectedEOF)? {
            Token::LeftParenthesis => (),
            t => return Err(UnexpectedToken(t)),
        }
        self.view = &self.view[1..];

        let operator = match self.view.first().ok_or(UnexpectedEOF)? {
            Token::Operator(op) => op.clone(),
            t => return Err(UnexpectedToken(t))
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
                },
            }
        }

        Ok(Expression::Operation(operator, arguments))
    }
}
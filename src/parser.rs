use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use ParserError::*;

use crate::lexer::{Keyword, Literal, Operator, Token};
use crate::parser::Expression::Scope;
use crate::parser::ParserError::VariableNotFound;

#[derive(Default, Debug)]
pub struct Context<'p, 'k, 'v> {
    pub parent_context: Option<&'p Context<'p, 'k, 'v>>,
    pub functions: HashMap<&'k str, Function<'v, 'v>>,
    pub variables: HashMap<&'k str, Value>,
}

impl<'p, 'k, 'v> Context<'p, 'k, 'v> {
    pub fn get_function(&self, name: &str) -> Option<&Function<'v, 'v>> {
        self.functions.get(name).or_else(|| self.parent_context.and_then(|c| c.get_function(name)))
    }
}

#[derive(Debug)]
enum Expression {
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

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Integer(int) => write!(f, "{}", int),
            Value::Float(float) => write!(f, "{}", float),
            Value::String(string) => write!(f, "{}", string),
            Value::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    VariableNotFound(String),
    FunctionNotFound(String),
    WrongNumberOfArguments,
    UnexpectedToken(Token),
    UnexpectedEOF,
    InvalidOperands,
}

impl Expression {
    fn evaluate(&self, context: &Context) -> Result<Value, ParserError> {
        match self {
            Expression::Id(id) => context.variables.get(id as &str).cloned().ok_or(VariableNotFound(id.to_owned())),
            Expression::Value(value) => Ok(value.clone()),
            Expression::Scope(expressions) => {
                expressions.iter().fold(Ok(Value::Unit), |_, expression| {
                    expression.evaluate(context)
                })
            }
            Expression::Operation(op, operands) => {
                let mut values = vec![];
                for op in operands {
                    values.push(op.evaluate(context)?);
                }

                use Value::*;
                match op {
                    Operator::Plus => values.into_iter().fold(Ok(Value::Integer(0)), |acc, x| {
                        acc.and_then(|acc| {
                            match (acc, x) {
                                (String(lhs), String(rhs)) => Ok(String(format!("{}{}", lhs, rhs))),
                                (String(lhs), Integer(rhs)) => Ok(String(format!("{}{}", lhs, rhs))),
                                (String(lhs), Float(rhs)) => Ok(String(format!("{}{}", lhs, rhs))),
                                (Integer(lhs), String(rhs)) => Ok(String(format!("{}{}", lhs, rhs))),
                                (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs + rhs)),
                                (Integer(lhs), Float(rhs)) => Ok(Float(lhs as f32 + rhs)),
                                (Float(lhs), String(rhs)) => Ok(String(format!("{}{}", lhs, rhs))),
                                (Float(lhs), Integer(rhs)) => Ok(Float(lhs + rhs as f32)),
                                (Float(lhs), Float(rhs)) => Ok(Float(lhs + rhs)),
                                _ => Err(InvalidOperands),
                            }
                        })
                    }),
                    Operator::Equal => {
                        let result = values.windows(2).all(|slice| {
                            match (&slice[0], &slice[1]) {
                                (Boolean(x), Boolean(y)) => x == y,
                                (Integer(x), Integer(y)) => x == y,
                                (Float(x), Float(y)) => x == y,
                                (String(x), String(y)) => x == y,
                                _ => false,
                            }
                        });
                        Ok(Boolean(result))
                    },
                    Operator::GreaterThan => {
                        let result = values.windows(2).all(|slice| {
                            match (&slice[0], &slice[1]) {
                                (Integer(x), Integer(y)) => x > y,
                                (Float(x), Float(y)) => x > y,
                                (String(x), String(y)) => x > y,
                                _ => false,
                            }
                        });
                        Ok(Boolean(result))
                    },
                    Operator::LessThan => {
                        let result = values.windows(2).all(|slice| {
                            match (&slice[0], &slice[1]) {
                                (Integer(x), Integer(y)) => x < y,
                                (Float(x), Float(y)) => x < y,
                                (String(x), String(y)) => x < y,
                                _ => false,
                            }
                        });
                        Ok(Boolean(result))
                    },
                    Operator::GreaterThanOrEqual => {
                        let result = values.windows(2).all(|slice| {
                            match (&slice[0], &slice[1]) {
                                (Integer(x), Integer(y)) => x >= y,
                                (Float(x), Float(y)) => x >= y,
                                (String(x), String(y)) => x >= y,
                                _ => false,
                            }
                        });
                        Ok(Boolean(result))
                    },
                    Operator::LessThanOrEqual => {
                        let result = values.windows(2).all(|slice| {
                            match (&slice[0], &slice[1]) {
                                (Integer(x), Integer(y)) => x <= y,
                                (Float(x), Float(y)) => x <= y,
                                (String(x), String(y)) => x <= y,
                                _ => false,
                            }
                        });
                        Ok(Boolean(result))
                    },
                    _ => {
                        let mut iter = values.into_iter();
                        let first = iter.next().ok_or(WrongNumberOfArguments)?;
                        iter.fold(Ok(first), |acc, x| {
                            acc.and_then(|acc| {
                                fn compute_float_operation(lhs: f32, op: &Operator, rhs: f32) -> Result<Value, ParserError> {
                                    match op {
                                        Operator::Plus => Ok(Float(lhs + rhs)),
                                        Operator::Minus => Ok(Float(lhs - rhs)),
                                        Operator::Asterisk => Ok(Float(lhs * rhs)),
                                        Operator::Slash => Ok(Float(lhs / rhs)),
                                        Operator::Pow => Ok(Float(lhs.powf(rhs))),
                                        _ => Err(InvalidOperands),
                                    }
                                }

                                match (acc, x) {
                                    (Integer(lhs), Integer(rhs)) => {
                                        match op {
                                            Operator::Minus => Ok(Integer(lhs - rhs)),
                                            Operator::Asterisk => Ok(Integer(lhs * rhs)),
                                            Operator::Slash => Ok(Integer(lhs / rhs)),
                                            Operator::Pow => Ok(Float((lhs as f32).powi(rhs))),
                                            _ => Err(InvalidOperands),
                                        }
                                    }
                                    (Integer(lhs), Float(rhs)) => compute_float_operation(lhs as f32, op, rhs),
                                    (Float(lhs), Integer(rhs)) => compute_float_operation(lhs, op, rhs as f32),
                                    (Float(lhs), Float(rhs)) => compute_float_operation(lhs, op, rhs),
                                    _ => Err(InvalidOperands)
                                }
                            })
                        })
                    }
                }
            }
            Expression::FunctionCall(name, arguments) => {
                let function = context.get_function(name as &str).ok_or(FunctionNotFound(name.to_owned()))?;
                let mut values = vec![];
                for arg in arguments {
                    values.push(arg.evaluate(context)?);
                }
                function.call(&context, values)
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
    pub fn call(&self, context: &Context, args: Vec<Value>) -> Result<Value, ParserError> {
        if self.parameters.len() != args.len() {
            return Err(ParserError::WrongNumberOfArguments);
        }

        let context = Context {
            parent_context: Some(context),
            functions: HashMap::new(),
            variables: {
                let mut hashmap = HashMap::new();
                for (&param, arg) in self.parameters.iter().zip(args) {
                    hashmap.insert(param, arg);
                }
                hashmap
            },
        };

        self.body.evaluate(&context)
    }
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
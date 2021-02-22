use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::lexer::Operator;
use crate::parser::{Context, Expression, Function, Value};
use crate::interpreter::InterpreterError::{VariableNotFound, InvalidOperands, WrongNumberOfArguments, FunctionNotFound};

#[derive(Debug)]
pub enum InterpreterError {
    VariableNotFound(String),
    FunctionNotFound(String),
    WrongNumberOfArguments,
    InvalidOperands,
}

impl<'p, 'k, 'v> Context<'p, 'k, 'v> {
    pub fn get_function(&self, name: &str) -> Option<&Function<'v, 'v>> {
        self.functions.get(name).or_else(|| self.parent_context.and_then(|c| c.get_function(name)))
    }
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

impl Expression {
    fn evaluate(&self, context: &Context) -> Result<Value, InterpreterError> {
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
                    }
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
                    }
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
                    }
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
                    }
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
                    }
                    _ => {
                        let mut iter = values.into_iter();
                        let first = iter.next().ok_or(WrongNumberOfArguments)?;
                        iter.fold(Ok(first), |acc, x| {
                            acc.and_then(|acc| {
                                fn compute_float_operation(lhs: f32, op: &Operator, rhs: f32) -> Result<Value, InterpreterError> {
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

impl<'n, 'p> Function<'n, 'p> {
    pub fn call(&self, context: &Context, args: Vec<Value>) -> Result<Value, InterpreterError> {
        if self.parameters.len() != args.len() {
            return Err(InterpreterError::WrongNumberOfArguments);
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

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::interpreter::InterpreterError::{FunctionNotFound, InvalidOperands, VariableNotFound, WrongNumberOfArguments};
use crate::lexer::Operator;
use crate::parser::{Context, Expression, Function, Value};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub enum InterpreterError {
    VariableNotFound(String),
    FunctionNotFound(String),
    WrongNumberOfArguments,
    InvalidOperands,
}

pub trait ContextTrait {
    fn get_variable(&self, name: &str) -> Option<Value>;
    fn get_function(&self, name: &str) -> Option<(Self, Rc<Function>)> where Self: Sized;
}

impl ContextTrait for Rc<RefCell<Context>> {
    fn get_variable(&self, name: &str) -> Option<Value> {
        let b = RefCell::borrow(self);
        match b.variables.get(name) {
            None => b.parent_context.as_ref().and_then(|p| p.get_variable(name)),
            Some(value) => Some(value.clone())
        }
    }

    fn get_function(&self, name: &str) -> Option<(Self, Rc<Function>)> {
        let b = RefCell::borrow(self);
        match b.functions.get(name).cloned() {
            Some(function) => Some((self.clone(), function)),
            None => b.parent_context.as_ref().and_then(|c| c.get_function(name))
        }
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
    pub(crate) fn evaluate(&self, context: Rc<RefCell<Context>>) -> Result<Value, InterpreterError> {
        match self {
            Expression::Id(id) => context.get_variable(id as &str).ok_or(VariableNotFound(id.to_owned())),
            Expression::Value(value) => Ok(value.clone()),
            Expression::Declaration(name, rhs) => {
                let rhs = rhs.evaluate(context.clone())?;
                context.borrow_mut().variables.insert(name.to_owned(), rhs);
                Ok(Value::Unit)
            }
            Expression::Scope(expressions) => {
                let context = Rc::new(RefCell::new(Context::with_parent(context.clone())));

                expressions.iter().fold(Ok(Value::Unit), |acc, expression| {
                    acc.and(expression.evaluate(context.clone()))
                })
            }
            Expression::Function(function) => {
                context.borrow_mut().functions.insert(function.name.to_owned(), function.clone());
                Ok(Value::Unit)
            }
            Expression::FunctionCall(name, arguments) => {
                let mut values = vec![];
                for arg in arguments {
                    values.push(arg.evaluate(context.clone())?);
                }
                let (fn_context, function) = context.get_function(name as &str).ok_or(FunctionNotFound(name.to_owned()))?;
                function.call(fn_context, values)
            }
            Expression::Operation(op, operands) => {
                let mut values = vec![];
                for op in operands {
                    values.push(op.evaluate(context.clone())?);
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
        }
    }
}

impl Function {
    pub fn call(&self, context: Rc<RefCell<Context>>, args: Vec<Value>) -> Result<Value, InterpreterError> {
        if self.parameters.len() != args.len() {
            return Err(InterpreterError::WrongNumberOfArguments);
        }

        let context = Rc::new(RefCell::new(Context {
            parent_context: Some(context),
            functions: HashMap::new(),
            variables: {
                let mut hashmap = HashMap::new();
                for (param, arg) in self.parameters.iter().zip(args) {
                    hashmap.insert(param.to_owned(), arg);
                }
                hashmap
            },
        }));
        self.body.evaluate(context)
    }
}

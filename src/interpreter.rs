use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use crate::interpreter::InterpreterError::*;
use crate::lexer::Operator;
use crate::parser::{Context, Expression, Function, Value};

#[derive(Debug)]
pub enum InterpreterError {
    VariableNotFound(String),
    FunctionNotFound(String),
    WrongNumberOfArguments,
    InvalidOperands,
}

pub trait ContextTrait {
    fn get_variable(&self, name: &str) -> Option<Value>;
    fn set_variable(&self, name: &str, value: Value) -> Result<(), ()>;
}

impl ContextTrait for Rc<RefCell<Context>> {
    fn get_variable(&self, name: &str) -> Option<Value> {
        let b = RefCell::borrow(self);
        match b.variables.get(name) {
            None => b.parent_context.as_ref().and_then(|p| p.get_variable(name)),
            Some(value) => Some(value.clone())
        }
    }

    fn set_variable(&self, name: &str, new_value: Value) -> Result<(), ()> {
        let mut b = RefCell::borrow_mut(self);
        match b.variables.get_mut(name) {
            None => b.parent_context.as_ref().ok_or(()).and_then(|p| p.set_variable(name, new_value)),
            Some(value) => {
                *value = new_value;
                Ok(())
            }
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
            Value::Function(Function::BuiltInFunction { name, .. }) => write!(f, "fn {}", name),
            Value::Function(Function::LanguageFunction { name, .. }) => write!(f, "fn {}", name),
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
            Expression::Assignment(name, rhs) => {
                let rhs = rhs.evaluate(context.clone())?;
                context.set_variable(name, rhs).map_err(|_| VariableNotFound(name.to_owned()))?;
                Ok(Value::Unit)
            }
            Expression::Scope(expressions) => {
                let context = Rc::new(RefCell::new(Context::with_parent(context.clone())));

                expressions.iter().fold(Ok(Value::Unit), |acc, expression| {
                    acc.and(expression.evaluate(context.clone()))
                })
            }
            Expression::NamedFunctionDefinition { name, parameters, body } => {
                context.borrow_mut().variables.insert(name.to_owned(), Value::Function(Function::LanguageFunction {
                    closing_context: context.clone(),
                    name: name.to_owned(),
                    parameters: parameters.to_owned(),
                    body: body.clone(),
                }));
                Ok(Value::Unit)
            }
            Expression::AnonymousFunctionDefinition { parameters, body } => {
                Ok(Value::Function(Function::LanguageFunction {
                    closing_context: context.clone(),
                    name: "anonymous".to_owned(),
                    parameters: parameters.to_owned(),
                    body: body.clone(),
                }))
            }
            Expression::FunctionCall(function_ptr, arguments) => {
                let mut values = vec![];
                for arg in arguments {
                    values.push(arg.evaluate(context.clone())?);
                }
                match function_ptr.evaluate(context)? {
                    Value::Function(f) => f.call(values),
                    v => Err(FunctionNotFound(v.to_string()))
                }
            }
            Expression::If { guard, base_case } => {
                let context = Rc::new(RefCell::new(Context::with_parent(context)));

                let is_guard_true = match guard.evaluate(context.clone())? {
                    Value::Boolean(b) => b,
                    _ => false, // We don't do implicit casting to boolean
                };
                if is_guard_true {
                    base_case.evaluate(context)?;
                }
                Ok(Value::Unit)
            }
            Expression::IfElse { guard, base_case, else_case } => {
                let context = Rc::new(RefCell::new(Context::with_parent(context)));

                let is_guard_true = match guard.evaluate(context.clone())? {
                    Value::Boolean(b) => b,
                    _ => false, // We don't do implicit casting to boolean
                };
                match is_guard_true {
                    true => base_case.evaluate(context),
                    false => else_case.evaluate(context),
                }
            }
            Expression::While { guard, body } => {
                let context = Rc::new(RefCell::new(Context::with_parent(context)));

                while {
                    match guard.evaluate(context.clone())? {
                        Value::Boolean(b) => b,
                        _ => false, // We don't do implicit casting to boolean
                    }
                } {
                    body.evaluate(context.clone())?;
                }
                Ok(Value::Unit)
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
    pub fn call(&self, args: Vec<Value>) -> Result<Value, InterpreterError> {
        match self {
            Function::BuiltInFunction { closing_context, name: _, parameters, fn_pointer } => {
                if parameters.len() != args.len() {
                    return Err(InterpreterError::WrongNumberOfArguments);
                }
                fn_pointer(closing_context.clone(), args)
            }
            Function::LanguageFunction { closing_context, name: _, parameters, body } => {
                if parameters.len() != args.len() {
                    return Err(InterpreterError::WrongNumberOfArguments);
                }

                let context = Rc::new(RefCell::new(Context {
                    parent_context: Some(closing_context.clone()),
                    variables: {
                        let mut hashmap = HashMap::new();
                        for (param, arg) in parameters.iter().zip(args) {
                            hashmap.insert(param.to_owned(), arg);
                        }
                        hashmap
                    },
                }));
                body.evaluate(context)
            }
        }
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Range, Deref};

use crate::interpreter::InterpreterError::*;
use crate::parser::{Context, Expression, ExpressionWithMetadata, Function, Value, IntoSharedRef};
use std::rc::Rc;

#[derive(Debug)]
pub struct InterpreterErrorWithSpan {
    pub error: InterpreterError,
    pub span: Option<Range<usize>>,
}

#[derive(Debug)]
pub enum InterpreterError {
    VariableNotFound(String),
    NotAFunction,
    WrongNumberOfArguments,
    InvalidOperands,
    StdInError,
    IndexOutOfBounds,
    InvalidIndex,
}

impl InterpreterError {
    pub fn with_span(self, span: Range<usize>) -> InterpreterErrorWithSpan {
        InterpreterErrorWithSpan {
            error: self,
            span: Some(span),
        }
    }
}

impl From<InterpreterError> for InterpreterErrorWithSpan {
    fn from(error: InterpreterError) -> Self {
        InterpreterErrorWithSpan {
            error,
            span: None,
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
            Value::Function(Function::NativeFunction { name, .. }) => write!(f, "fn {}", name),
            Value::Function(Function::RuspFunction { name, .. }) => write!(f, "fn {}", name),
            Value::List(values) => {
                write!(f, "[")?;
                match values.as_slice() {
                    [single] => write!(f, "{}", single.borrow())?,
                    [init @ .., last] => {
                        for value in init {
                            write!(f, "{} ", value.borrow())?;
                        }
                        write!(f, "{}", last.borrow())?;
                    }
                    [] => {}
                }
                write!(f, "]")
            }
        }
    }
}

impl ExpressionWithMetadata {
    pub(crate) fn evaluate(&self, context: Rc<RefCell<Context>>) -> Result<Rc<RefCell<Value>>, InterpreterErrorWithSpan> {
        match &self.expression {
            Expression::Id(id) => context.borrow().get_variable(id as &str)
                .ok_or(VariableNotFound(id.to_owned()).with_span(self.span.clone())),
            Expression::Value(value) => Ok(Rc::new(RefCell::new(value.clone()))),
            Expression::Declaration(name, rhs) => {
                let rhs = rhs.evaluate(context.clone())?;
                context.borrow_mut().variables.insert(name.label.clone(), rhs);
                Ok(Value::unit())
            }
            Expression::Assignment(name, rhs) => {
                let rhs = rhs.evaluate(context.clone())?;
                match context.borrow_mut().set_variable(&name.label, rhs) {
                    Ok(()) => Ok(Value::unit()),
                    Err(()) => Err(VariableNotFound(name.label.to_owned())
                        .with_span(name.span.clone()))
                }
            }
            Expression::Scope(expressions) => {
                let context = Rc::new(RefCell::new(Context::with_parent(context.clone())));

                expressions.iter().fold(Ok(Value::unit()), |acc, expression| {
                    acc.and(expression.evaluate(context.clone()))
                })
            }
            Expression::NamedFunctionDefinition { name, parameters, body } => {
                context.borrow_mut().variables.insert(name.label.clone(), Value::Function(Function::RuspFunction {
                    closing_context: context.clone(),
                    name: name.label.clone(),
                    parameters: parameters.iter().map(|p| p.label.clone()).collect(),
                    body: body.clone(),
                }).into_shared_ref());
                Ok(Value::unit())
            }
            Expression::AnonymousFunctionDefinition { parameters, body } => {
                Ok(Value::Function(Function::RuspFunction {
                    closing_context: context.clone(),
                    name: "*anonymous*".to_owned(),
                    parameters: parameters.iter().map(|p| p.label.clone()).collect(),
                    body: body.clone(),
                }).into_shared_ref())
            }
            Expression::FunctionCall(function_ptr, arguments) => {
                let mut values = vec![];
                for arg in arguments {
                    values.push(arg.evaluate(context.clone())?);
                }
                match function_ptr.evaluate(context)?.borrow().deref() {
                    Value::Function(f) => {
                        f.call(values).map_err(|mut err| {
                            if err.span.is_none() {
                                err.span = Some(self.span.clone());
                            }
                            err
                        })
                    }
                    _ => Err(NotAFunction.with_span(function_ptr.span.clone()))
                }
            }
            Expression::If { guard, base_case } => {
                let context = Rc::new(RefCell::new(Context::with_parent(context)));

                let is_guard_true = match guard.evaluate(context.clone())?.borrow().deref() {
                    Value::Boolean(b) => *b,
                    _ => false, // We don't do implicit casting to boolean
                };
                if is_guard_true {
                    base_case.evaluate(context)?;
                }
                Ok(Value::unit())
            }
            Expression::IfElse { guard, base_case, else_case } => {
                let context = Rc::new(RefCell::new(Context::with_parent(context)));

                let is_guard_true = match guard.evaluate(context.clone())?.borrow().deref() {
                    Value::Boolean(b) => *b,
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
                    match guard.evaluate(context.clone())?.borrow().deref() {
                        Value::Boolean(b) => *b,
                        _ => false, // We don't do implicit casting to boolean
                    }
                } {
                    body.evaluate(context.clone())?;
                }
                Ok(Value::unit())
            }
            Expression::List(expressions) => {
                let mut values = Vec::new();
                for expression in expressions {
                    values.push(expression.evaluate(context.clone())?);
                }
                Ok(Value::List(values).into_shared_ref())
            }
        }
    }
}

impl Function {
    pub fn call(&self, args: Vec<Rc<RefCell<Value>>>) -> Result<Rc<RefCell<Value>>, InterpreterErrorWithSpan> {
        match self {
            Function::NativeFunction { closing_context, name: _, fn_pointer } => {
                fn_pointer(closing_context.clone(), args)
            }
            Function::RuspFunction { closing_context, name: _, parameters, body } => {
                if parameters.len() != args.len() {
                    return Err(InterpreterError::WrongNumberOfArguments.into());
                }

                // Put the arguments in the context
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

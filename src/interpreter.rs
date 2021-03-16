use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::rc::Rc;

use crate::interpreter::InterpreterError::*;
use crate::parser::{Context, Expression, ExpressionWithMetadata, Function, Value};

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
            Value::Function(Function::NativeFunction { name, .. }) => write!(f, "fn {}", name),
            Value::Function(Function::RuspFunction { name, .. }) => write!(f, "fn {}", name),
            Value::List(values) => {
                write!(f, "[")?;
                match values.as_slice() {
                    [single] => write!(f, "{}", single)?,
                    [init @ .., last] => {
                        for value in init {
                            write!(f, "{} ", value)?;
                        }
                        write!(f, "{}", last)?;
                    }
                    [] => {}
                }
                write!(f, "]")
            }
        }
    }
}

impl ExpressionWithMetadata {
    pub(crate) fn evaluate(&self, context: Rc<RefCell<Context>>) -> Result<Value, InterpreterErrorWithSpan> {
        match &self.expression {
            Expression::Id(id) => context.get_variable(id as &str)
                .ok_or(VariableNotFound(id.to_owned()).with_span(self.span.clone())),
            Expression::Value(value) => Ok(value.clone()),
            Expression::Declaration(name, rhs) => {
                let rhs = rhs.evaluate(context.clone())?;
                context.borrow_mut().variables.insert(name.label.clone(), rhs);
                Ok(Value::Unit)
            }
            Expression::Assignment(name, rhs) => {
                let rhs = rhs.evaluate(context.clone())?;
                match context.set_variable(&name.label, rhs) {
                    Ok(()) => Ok(Value::Unit),
                    Err(()) => Err(VariableNotFound(name.label.to_owned())
                        .with_span(name.span.clone()))
                }
            }
            Expression::Scope(expressions) => {
                let context = Rc::new(RefCell::new(Context::with_parent(context.clone())));

                expressions.iter().fold(Ok(Value::Unit), |acc, expression| {
                    acc.and(expression.evaluate(context.clone()))
                })
            }
            Expression::NamedFunctionDefinition { name, parameters, body } => {
                context.borrow_mut().variables.insert(name.label.clone(), Value::Function(Function::RuspFunction {
                    closing_context: context.clone(),
                    name: name.label.clone(),
                    parameters: parameters.iter().map(|p| p.label.clone()).collect(),
                    body: body.clone(),
                }));
                Ok(Value::Unit)
            }
            Expression::AnonymousFunctionDefinition { parameters, body } => {
                Ok(Value::Function(Function::RuspFunction {
                    closing_context: context.clone(),
                    name: "*anonymous*".to_owned(),
                    parameters: parameters.iter().map(|p| p.label.clone()).collect(),
                    body: body.clone(),
                }))
            }
            Expression::FunctionCall(function_ptr, arguments) => {
                let mut values = vec![];
                for arg in arguments {
                    values.push(arg.evaluate(context.clone())?);
                }
                match function_ptr.evaluate(context)? {
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
            Expression::List(expressions) => {
                let mut values = Vec::new();
                for expression in expressions {
                    values.push(expression.evaluate(context.clone())?);
                }
                Ok(Value::List(values))
            }
        }
    }
}

impl Function {
    pub fn call(&self, args: Vec<Value>) -> Result<Value, InterpreterErrorWithSpan> {
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

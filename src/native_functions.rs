use std::cell::RefCell;
use std::convert::TryFrom;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::interpreter::{InterpreterError, InterpreterErrorWithSpan};
use crate::parser::{Context, Function, IntoSharedRef, Value};

pub fn add_native_function(
    context: &mut Rc<RefCell<Context>>,
    name: &str,
    fn_pointer: fn(Rc<RefCell<Context>>, Vec<Rc<RefCell<Value>>>) -> Result<Rc<RefCell<Value>>, InterpreterErrorWithSpan>,
) {
    context.borrow_mut().variables.insert(name.to_owned(), Value::Function(Function::NativeFunction {
        closing_context: context.clone(),
        name: name.to_owned(),
        fn_pointer,
    }).into_shared_ref());
}

pub fn create_global_context_with_native_functions() -> Rc<RefCell<Context>> {
    let mut global_context = Rc::new(RefCell::new(Context::default()));

    add_native_function(&mut global_context, "==", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0].borrow().deref(), &slice[1].borrow().deref()) {
                (Boolean(x), Boolean(y)) => x == y,
                (Integer(x), Integer(y)) => x == y,
                (Float(x), Float(y)) => x == y,
                (String(x), String(y)) => x == y,
                _ => false,
            }
        });
        Ok(Boolean(result).into_shared_ref())
    });

    add_native_function(&mut global_context, "!=", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0].borrow().deref(), &slice[1].borrow().deref()) {
                (Boolean(x), Boolean(y)) => x != y,
                (Integer(x), Integer(y)) => x != y,
                (Float(x), Float(y)) => x != y,
                (String(x), String(y)) => x != y,
                _ => false,
            }
        });
        Ok(Boolean(result).into_shared_ref())
    });

    add_native_function(&mut global_context, "<", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0].borrow().deref(), &slice[1].borrow().deref()) {
                (Integer(x), Integer(y)) => x < y,
                (Float(x), Float(y)) => x < y,
                (String(x), String(y)) => x < y,
                _ => false,
            }
        });
        Ok(Boolean(result).into_shared_ref())
    });

    add_native_function(&mut global_context, ">", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0].borrow().deref(), &slice[1].borrow().deref()) {
                (Integer(x), Integer(y)) => x > y,
                (Float(x), Float(y)) => x > y,
                (String(x), String(y)) => x > y,
                _ => false,
            }
        });
        Ok(Boolean(result).into_shared_ref())
    });

    add_native_function(&mut global_context, "<=", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0].borrow().deref(), &slice[1].borrow().deref()) {
                (Integer(x), Integer(y)) => x <= y,
                (Float(x), Float(y)) => x <= y,
                (String(x), String(y)) => x <= y,
                _ => false,
            }
        });
        Ok(Boolean(result).into_shared_ref())
    });

    add_native_function(&mut global_context, ">=", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0].borrow().deref(), &slice[1].borrow().deref()) {
                (Integer(x), Integer(y)) => x >= y,
                (Float(x), Float(y)) => x >= y,
                (String(x), String(y)) => x >= y,
                _ => false,
            }
        });
        Ok(Boolean(result).into_shared_ref())
    });

    add_native_function(&mut global_context, "+", |_context, arguments| {
        let mut iter = arguments.into_iter();
        let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments.into());

        iter.into_iter().fold(first, |acc, x| {
            use Value::*;
            acc.and_then(|acc| {
                match (acc.borrow().deref(), x.borrow().deref()) {
                    (String(lhs), String(rhs)) => Ok(String(format!("{}{}", lhs, rhs)).into_shared_ref()),
                    (String(lhs), Integer(rhs)) => Ok(String(format!("{}{}", lhs, rhs)).into_shared_ref()),
                    (String(lhs), Float(rhs)) => Ok(String(format!("{}{}", lhs, rhs)).into_shared_ref()),
                    (Integer(lhs), String(rhs)) => Ok(String(format!("{}{}", lhs, rhs)).into_shared_ref()),
                    (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs + rhs).into_shared_ref()),
                    (Integer(lhs), Float(rhs)) => Ok(Float(*lhs as f32 + rhs).into_shared_ref()),
                    (Float(lhs), String(rhs)) => Ok(String(format!("{}{}", lhs, rhs)).into_shared_ref()),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs + *rhs as f32).into_shared_ref()),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs + rhs).into_shared_ref()),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "-", |_context, arguments| {
        let mut iter = arguments.into_iter();
        let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments.into());

        iter.into_iter().fold(first, |acc, x| {
            use Value::*;
            acc.and_then(|acc| {
                match (acc.borrow().deref(), x.borrow().deref()) {
                    (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs - rhs).into_shared_ref()),
                    (Integer(lhs), Float(rhs)) => Ok(Float(*lhs as f32 - rhs).into_shared_ref()),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs - *rhs as f32).into_shared_ref()),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs - rhs).into_shared_ref()),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "*", |_context, arguments| {
        let mut iter = arguments.into_iter();
        let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments.into());

        iter.into_iter().fold(first, |acc, x| {
            use Value::*;
            acc.and_then(|acc| {
                match (acc.borrow().deref(), x.borrow().deref()) {
                    (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs * rhs).into_shared_ref()),
                    (Integer(lhs), Float(rhs)) => Ok(Float(*lhs as f32 * rhs).into_shared_ref()),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs * *rhs as f32).into_shared_ref()),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs * rhs).into_shared_ref()),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "/", |_context, arguments| {
        let mut iter = arguments.into_iter();
        let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments.into());

        iter.into_iter().fold(first, |acc, x| {
            use Value::*;
            acc.and_then(|acc| {
                match (acc.borrow().deref(), x.borrow().deref()) {
                    (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs / rhs).into_shared_ref()),
                    (Integer(lhs), Float(rhs)) => Ok(Float(*lhs as f32 / rhs).into_shared_ref()),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs / *rhs as f32).into_shared_ref()),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs / rhs).into_shared_ref()),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "**", |_context, arguments| {
        let mut iter = arguments.into_iter();
        let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments.into());

        iter.into_iter().fold(first, |acc, x| {
            use Value::*;
            acc.and_then(|acc| {
                match (acc.borrow().deref(), x.borrow().deref()) {
                    (Integer(lhs), Integer(rhs)) => Ok(Float((*lhs as f32).powi(*rhs)).into_shared_ref()),
                    (Integer(lhs), Float(rhs)) => Ok(Float((*lhs as f32).powf(*rhs)).into_shared_ref()),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs.powf(*rhs as f32)).into_shared_ref()),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs.powf(*rhs)).into_shared_ref()),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "!", |_context, arguments| {
        match arguments.as_slice() {
            [boolean] => {
                if let Value::Boolean(b) = boolean.borrow().deref() {
                    Ok(Value::Boolean(!*b).into_shared_ref())
                } else {
                    return Err(InterpreterError::WrongNumberOfArguments.into());
                }
            }
            _ => Err(InterpreterError::WrongNumberOfArguments.into())
        }
    });

    add_native_function(&mut global_context, "&&", |_context, arguments| {
        arguments.into_iter().fold(Ok(Value::Boolean(true).into_shared_ref()), |acc, x| {
            use Value::*;
            acc.and_then(|acc| {
                match (acc.borrow().deref(), x.borrow().deref()) {
                    (Boolean(lhs), Boolean(rhs)) => Ok(Value::Boolean(*lhs && *rhs).into_shared_ref()),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "||", |_context, arguments| {
        arguments
            .into_iter()
            .fold(Ok(Value::Boolean(false)), |acc, x| {
                use Value::*;
                acc.and_then(|acc| {
                    match (acc, x.borrow().deref().clone()) {
                        (Boolean(lhs), Boolean(rhs)) => Ok(Value::Boolean(lhs || rhs)),
                        _ => Err(InterpreterError::InvalidOperands.into()),
                    }
                })
            })
            .map(|a| a.into_shared_ref())
    });

    add_native_function(&mut global_context, "print", |_context, arguments| {
        match arguments.as_slice() {
            [value] => print!("{}", value.borrow()),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into()),
        }
        Ok(Value::unit())
    });

    add_native_function(&mut global_context, "println", |_context, arguments| {
        match arguments.as_slice() {
            [] => println!(),
            [value] => println!("{}", value.borrow()),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into()),
        }
        Ok(Value::unit())
    });

    add_native_function(&mut global_context, "eprint", |_context, arguments| {
        match arguments.as_slice() {
            [value] => eprint!("{}", value.borrow()),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into()),
        }
        Ok(Value::unit())
    });

    add_native_function(&mut global_context, "eprintln", |_context, arguments| {
        match arguments.as_slice() {
            [] => eprintln!(),
            [value] => eprintln!("{}", value.borrow()),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into()),
        }
        Ok(Value::unit())
    });

    add_native_function(&mut global_context, "dbg", |_context, arguments| {
        println!("{:#?}", &arguments[0].borrow());
        Ok(Value::unit())
    });

    add_native_function(&mut global_context, "input", |_context, arguments| {
        match arguments.as_slice() {
            [] => (),
            [to_print] => print!("{}", to_print.borrow()),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into())
        }
        std::io::stdout().flush().map_err(|_| InterpreterError::StdInError)?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).map_err(|_| InterpreterError::StdInError)?;
        trim_newline(&mut line);
        Ok(Value::String(line).into_shared_ref())
    });

    add_native_function(&mut global_context, "get", |_context, arguments| {
        match arguments.as_slice() {
            [list, index] => {
                match (list.borrow().deref(), index.borrow().deref()) {
                    (Value::List(elements), Value::Integer(index)) => {
                        let index = usize::try_from(*index).map_err(|_| InterpreterError::InvalidIndex)?;
                        elements.get(index).cloned().ok_or(InterpreterError::IndexOutOfBounds.into())
                    }
                    _ => return Err(InterpreterError::WrongNumberOfArguments.into())
                }
            }
            _ => Err(InterpreterError::WrongNumberOfArguments.into())
        }
    });

    add_native_function(&mut global_context, "push", |_context, arguments| {
        match arguments.as_slice() {
            [list, value] => {
                if let Value::List(elements) = list.borrow_mut().deref_mut() {
                    elements.push(value.clone());
                    Ok(Value::unit())
                } else {
                    // TODO: Wrong type error message
                    return Err(InterpreterError::WrongNumberOfArguments.into());
                }
            }
            _ => Err(InterpreterError::WrongNumberOfArguments.into())
        }
    });

    global_context
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
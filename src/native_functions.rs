use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use crate::interpreter::{InterpreterError, InterpreterErrorWithSpan};
use crate::parser::{Context, Function, Value};

pub fn add_native_function(
    context: &mut Rc<RefCell<Context>>,
    name: &str,
    fn_pointer: fn(Rc<RefCell<Context>>, Vec<Value>) -> Result<Value, InterpreterErrorWithSpan>) {
    context.borrow_mut().variables.insert(name.to_owned(), Value::Function(Function::NativeFunction {
        closing_context: context.clone(),
        name: name.to_owned(),
        fn_pointer,
    }));
}

pub fn create_global_context_with_native_functions() -> Rc<RefCell<Context>> {
    let mut global_context = Rc::new(RefCell::new(Context::default()));

    add_native_function(&mut global_context, "==", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0], &slice[1]) {
                (Boolean(x), Boolean(y)) => x == y,
                (Integer(x), Integer(y)) => x == y,
                (Float(x), Float(y)) => x == y,
                (String(x), String(y)) => x == y,
                _ => false,
            }
        });
        Ok(Boolean(result))
    });

    add_native_function(&mut global_context, "!=", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0], &slice[1]) {
                (Boolean(x), Boolean(y)) => x != y,
                (Integer(x), Integer(y)) => x != y,
                (Float(x), Float(y)) => x != y,
                (String(x), String(y)) => x != y,
                _ => false,
            }
        });
        Ok(Boolean(result))
    });

    add_native_function(&mut global_context, "<", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0], &slice[1]) {
                (Integer(x), Integer(y)) => x < y,
                (Float(x), Float(y)) => x < y,
                (String(x), String(y)) => x < y,
                _ => false,
            }
        });
        Ok(Boolean(result))
    });

    add_native_function(&mut global_context, ">", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0], &slice[1]) {
                (Integer(x), Integer(y)) => x > y,
                (Float(x), Float(y)) => x > y,
                (String(x), String(y)) => x > y,
                _ => false,
            }
        });
        Ok(Boolean(result))
    });

    add_native_function(&mut global_context, "<=", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0], &slice[1]) {
                (Integer(x), Integer(y)) => x <= y,
                (Float(x), Float(y)) => x <= y,
                (String(x), String(y)) => x <= y,
                _ => false,
            }
        });
        Ok(Boolean(result))
    });

    add_native_function(&mut global_context, ">=", |_context, arguments| {
        use Value::*;
        let result = arguments.windows(2).all(|slice| {
            match (&slice[0], &slice[1]) {
                (Integer(x), Integer(y)) => x >= y,
                (Float(x), Float(y)) => x >= y,
                (String(x), String(y)) => x >= y,
                _ => false,
            }
        });
        Ok(Boolean(result))
    });

    add_native_function(&mut global_context, "+", |_context, arguments| {
        let mut iter = arguments.into_iter();
        let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments.into());

        iter.into_iter().fold(first, |acc, x| {
            use Value::*;
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
                match (acc, x) {
                    (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs - rhs)),
                    (Integer(lhs), Float(rhs)) => Ok(Float(lhs as f32 - rhs)),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs - rhs as f32)),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs - rhs)),
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
                match (acc, x) {
                    (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs * rhs)),
                    (Integer(lhs), Float(rhs)) => Ok(Float(lhs as f32 * rhs)),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs * rhs as f32)),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs * rhs)),
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
                match (acc, x) {
                    (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs / rhs)),
                    (Integer(lhs), Float(rhs)) => Ok(Float(lhs as f32 / rhs)),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs / rhs as f32)),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs / rhs)),
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
                match (acc, x) {
                    (Integer(lhs), Integer(rhs)) => Ok(Float((lhs as f32).powi(rhs))),
                    (Integer(lhs), Float(rhs)) => Ok(Float((lhs as f32).powf(rhs))),
                    (Float(lhs), Integer(rhs)) => Ok(Float(lhs.powf(rhs as f32))),
                    (Float(lhs), Float(rhs)) => Ok(Float(lhs.powf(rhs))),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "!", |_context, arguments| {
        match arguments.as_slice() {
            [Value::Boolean(b)] => Ok(Value::Boolean(!*b)),
            _ => Err(InterpreterError::WrongNumberOfArguments.into())
        }
    });

    add_native_function(&mut global_context, "&&", |_context, arguments| {
        arguments.into_iter().fold(Ok(Value::Boolean(true)), |acc, x| {
            use Value::*;
            acc.and_then(|acc| {
                match (acc, x) {
                    (Boolean(lhs), Boolean(rhs)) => Ok(Value::Boolean(lhs && rhs)),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "||", |_context, arguments| {
        arguments.into_iter().fold(Ok(Value::Boolean(false)), |acc, x| {
            use Value::*;
            acc.and_then(|acc| {
                match (acc, x) {
                    (Boolean(lhs), Boolean(rhs)) => Ok(Value::Boolean(lhs || rhs)),
                    _ => Err(InterpreterError::InvalidOperands.into()),
                }
            })
        })
    });

    add_native_function(&mut global_context, "print", |_context, arguments| {
        match arguments.as_slice() {
            [value] => print!("{}", value),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into()),
        }
        Ok(Value::Unit)
    });

    add_native_function(&mut global_context, "println", |_context, arguments| {
        match arguments.as_slice() {
            [] => println!(),
            [value] => println!("{}", value),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into()),
        }
        Ok(Value::Unit)
    });

    add_native_function(&mut global_context, "eprint", |_context, arguments| {
        match arguments.as_slice() {
            [value] => eprint!("{}", value),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into()),
        }
        Ok(Value::Unit)
    });

    add_native_function(&mut global_context, "eprintln", |_context, arguments| {
        match arguments.as_slice() {
            [] => eprintln!(),
            [value] => eprintln!("{}", value),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into()),
        }
        Ok(Value::Unit)
    });

    add_native_function(&mut global_context, "dbg", |_context, arguments| {
        println!("{:#?}", &arguments[0]);
        Ok(Value::Unit)
    });

    add_native_function(&mut global_context, "input", |_context, arguments| {
        match arguments.as_slice() {
            [] => (),
            [to_print] => print!("{}", to_print),
            _ => return Err(InterpreterError::WrongNumberOfArguments.into())
        }
        std::io::stdout().flush().map_err(|_| InterpreterError::StdInError)?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).map_err(|_| InterpreterError::StdInError)?;
        trim_newline(&mut line);
        Ok(Value::String(line))
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
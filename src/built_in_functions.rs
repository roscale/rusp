use std::rc::Rc;
use std::cell::RefCell;
use crate::parser::{Context, Value, Function};
use crate::interpreter::InterpreterError;

pub fn create_global_context_with_built_in_functions() -> Rc<RefCell<Context>> {
    let global_context = Rc::new(RefCell::new(Context::default()));

    global_context.borrow_mut().variables.insert(String::from("=="), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "==".to_string(),
        fn_pointer: |_context, arguments| {
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
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("!="), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "!=".to_string(),
        fn_pointer: |_context, arguments| {
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
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("<"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "<".to_string(),
        fn_pointer: |_context, arguments| {
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
        },
    }));

    global_context.borrow_mut().variables.insert(String::from(">"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: ">".to_string(),
        fn_pointer: |_context, arguments| {
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
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("<="), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "<=".to_string(),
        fn_pointer: |_context, arguments| {
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
        },
    }));

    global_context.borrow_mut().variables.insert(String::from(">="), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: ">=".to_string(),
        fn_pointer: |_context, arguments| {
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
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("+"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "+".to_string(),
        fn_pointer: |_context, arguments| {
            let mut iter = arguments.into_iter();
            let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments);

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
                        _ => Err(InterpreterError::InvalidOperands),
                    }
                })
            })
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("-"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "-".to_string(),
        fn_pointer: |_context, arguments| {
            let mut iter = arguments.into_iter();
            let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments);

            iter.into_iter().fold(first, |acc, x| {
                use Value::*;
                acc.and_then(|acc| {
                    match (acc, x) {
                        (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs - rhs)),
                        (Integer(lhs), Float(rhs)) => Ok(Float(lhs as f32 - rhs)),
                        (Float(lhs), Integer(rhs)) => Ok(Float(lhs - rhs as f32)),
                        (Float(lhs), Float(rhs)) => Ok(Float(lhs - rhs)),
                        _ => Err(InterpreterError::InvalidOperands),
                    }
                })
            })
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("*"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "*".to_string(),
        fn_pointer: |_context, arguments| {
            let mut iter = arguments.into_iter();
            let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments);

            iter.into_iter().fold(first, |acc, x| {
                use Value::*;
                acc.and_then(|acc| {
                    match (acc, x) {
                        (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs * rhs)),
                        (Integer(lhs), Float(rhs)) => Ok(Float(lhs as f32 * rhs)),
                        (Float(lhs), Integer(rhs)) => Ok(Float(lhs * rhs as f32)),
                        (Float(lhs), Float(rhs)) => Ok(Float(lhs * rhs)),
                        _ => Err(InterpreterError::InvalidOperands),
                    }
                })
            })
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("/"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "/".to_string(),
        fn_pointer: |_context, arguments| {
            let mut iter = arguments.into_iter();
            let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments);

            iter.into_iter().fold(first, |acc, x| {
                use Value::*;
                acc.and_then(|acc| {
                    match (acc, x) {
                        (Integer(lhs), Integer(rhs)) => Ok(Integer(lhs / rhs)),
                        (Integer(lhs), Float(rhs)) => Ok(Float(lhs as f32 / rhs)),
                        (Float(lhs), Integer(rhs)) => Ok(Float(lhs / rhs as f32)),
                        (Float(lhs), Float(rhs)) => Ok(Float(lhs / rhs)),
                        _ => Err(InterpreterError::InvalidOperands),
                    }
                })
            })
        },
    }));

    global_context.borrow_mut().variables.insert(String::from("**"), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "**".to_string(),
        fn_pointer: |_context, arguments| {
            let mut iter = arguments.into_iter();
            let first = iter.next().ok_or(InterpreterError::WrongNumberOfArguments);

            iter.into_iter().fold(first, |acc, x| {
                use Value::*;
                acc.and_then(|acc| {
                    match (acc, x) {
                        (Integer(lhs), Integer(rhs)) => Ok(Float((lhs as f32).powi(rhs))),
                        (Integer(lhs), Float(rhs)) => Ok(Float((lhs as f32).powf(rhs))),
                        (Float(lhs), Integer(rhs)) => Ok(Float(lhs.powf(rhs as f32))),
                        (Float(lhs), Float(rhs)) => Ok(Float(lhs.powf(rhs))),
                        _ => Err(InterpreterError::InvalidOperands),
                    }
                })
            })
        },
    }));

    global_context.borrow_mut().variables.insert("print".to_owned(), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "print".to_owned(),
        fn_pointer: |_context, arguments| {
            print!("{}", arguments[0]);
            Ok(Value::Unit)
        },
    }));

    global_context.borrow_mut().variables.insert("println".to_owned(), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "println".to_owned(),
        fn_pointer: |_context, arguments| {
            println!("{}", arguments[0]);
            Ok(Value::Unit)
        },
    }));

    global_context.borrow_mut().variables.insert("dbg".to_owned(), Value::Function(Function::BuiltInFunction {
        closing_context: global_context.clone(),
        name: "dbg".to_owned(),
        fn_pointer: |_context, arguments| {
            println!("{:#?}", &arguments[0]);
            Ok(Value::Unit)
        },
    }));

    global_context
}

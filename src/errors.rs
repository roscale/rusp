use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use crate::lexer::LexerError;
use crate::parser::ParserError;

pub fn show_lexer_error<Name, Source>(error: LexerError, file_id: usize, files: SimpleFiles<Name, Source>)
    where Name: std::fmt::Display + Clone,
          Source: AsRef<str> {
    match error {
        LexerError::UnexpectedCharacter(span) => {
            let diagnostic = Diagnostic::error()
                .with_code("E0001")
                .with_message("Developer has suboptimal IQ")
                .with_labels(vec![
                    Label::primary(file_id, span).with_message("Learn the language syntax, you dumbass!")
                ]);

            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();

            let _ = term::emit(&mut writer.lock(), &config, &files, &diagnostic);
        }
    }
}

pub fn show_parser_error<Name, Source>(error: ParserError, file_id: usize, files: SimpleFiles<Name, Source>)
    where Name: std::fmt::Display + Clone,
          Source: AsRef<str> {
    let diagnostic = match error {
        ParserError::UnexpectedToken(span) => {
            Diagnostic::error()
                .with_code("E0001")
                .with_message("unexpected token")
                .with_labels(vec![
                    Label::primary(file_id, span).with_message("unexpected token")
                ])
        }
        ParserError::UnexpectedEOF => {
            Diagnostic::error()
                .with_code("E0002")
                .with_message("unexpected end of file")
        }
    };

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    let _ = term::emit(&mut writer.lock(), &config, &files, &diagnostic);
}

// pub fn show_interpreter_error<Name, Source>(error: InterpreterErrorWithSpan, file_id: usize, files: SimpleFiles<Name, Source>)
//     where Name: std::fmt::Display + Clone,
//           Source: AsRef<str> {
//     let diagnostic = match error.error {
//         InterpreterError::VariableNotFound(name) => {
//             Diagnostic::error()
//                 .with_code("E0003")
//                 .with_message(format!("variable '{}' not found", name))
//                 .with_labels(vec![
//                     Label::primary(file_id, error.span.unwrap()).with_message("variable not found")
//                 ])
//         }
//         InterpreterError::WrongNumberOfArguments => {
//             Diagnostic::error()
//                 .with_code("E0004")
//                 .with_message("wrong number of arguments")
//                 .with_labels(vec![
//                     Label::primary(file_id, error.span.unwrap()).with_message("wrong number of arguments")
//                 ])
//         }
//         InterpreterError::StdInError => {
//             Diagnostic::error()
//                 .with_code("E0005")
//                 .with_message("stdin error")
//                 .with_labels(vec![
//                     Label::primary(file_id, error.span.unwrap()).with_message("stdin error")
//                 ])
//         }
//         InterpreterError::InvalidOperands => {
//             Diagnostic::error()
//                 .with_code("E0006")
//                 .with_message("invalid operands")
//                 .with_labels(vec![
//                     Label::primary(file_id, error.span.unwrap()).with_message("invalid operands")
//                 ])
//         }
//         InterpreterError::NotAFunction => {
//             Diagnostic::error()
//                 .with_code("E0007")
//                 .with_message("expression does not evaluate to a function")
//                 .with_labels(vec![
//                     Label::primary(file_id, error.span.unwrap()).with_message("not a function")
//                 ])
//         }
//     };
//
//     let writer = StandardStream::stderr(ColorChoice::Always);
//     let config = codespan_reporting::term::Config::default();
//
//     let _ = term::emit(&mut writer.lock(), &config, &files, &diagnostic);
// }
use crate::util::{Boxed, next, skip};

#[derive(Debug, Clone)]
pub enum Token {
    Id(String),
    If,
    While,
    For,
    True,
    False,
    Float(f32),
    Integer(i32),
    StringLiteral(String),
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBracket,
    RightSquareBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Colon,
    Equal,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Pow,
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char),
}

pub trait LexerMode {
    fn tokenize(&mut self, view: &mut &[char], tokens: &mut Vec<Token>) -> Result<Box<dyn LexerMode>, LexerError>;
}

struct DecisionMakingMode;

impl LexerMode for DecisionMakingMode {
    fn tokenize(&mut self, view: &mut &[char], _tokens: &mut Vec<Token>) -> Result<Box<dyn LexerMode>, LexerError> {
        loop {
            match view {
                [w, ..] if w.is_whitespace() => *view = &view[1..],
                ['/', '/', ..] => break Ok(CommentMode.boxed()),
                ['"', ..] => break Ok(StringLiteralMode.boxed()),
                [digit, ..] if digit.is_ascii_digit() => break Ok(NumericLiteralMode.boxed()),
                ['+' | '-', digit, ..] if digit.is_ascii_digit() => break Ok(NumericLiteralMode.boxed()),
                [p, ..] if PunctuationMode::is_punctuation(*p) => break Ok(PunctuationMode.boxed()),
                [c, ..] if IdentifierMode::is_valid_character(*c) => break Ok(IdentifierMode.boxed()),
                [] => break Ok(DecisionMakingMode.boxed()),
                [e, ..] => break Err(LexerError::UnexpectedCharacter(*e))
            }
        }
    }
}

struct IdentifierMode;

impl IdentifierMode {
    fn is_valid_character(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}

impl LexerMode for IdentifierMode {
    fn tokenize(&mut self, view: &mut &[char], tokens: &mut Vec<Token>) -> Result<Box<dyn LexerMode>, LexerError> {
        let start = *view;
        let mut i = 0;

        loop {
            match view {
                [c, ..] if !Self::is_valid_character(*c) => {
                    let mut end_token = || {
                        if start.is_empty() {
                            return;
                        }
                        let token = start[..i].iter().collect::<String>();
                        let token = match token.as_str() {
                            "if" => Token::If,
                            "while" => Token::While,
                            "for" => Token::For,
                            "true" => Token::True,
                            "false" => Token::False,
                            _ => Token::Id(start[..i].iter().collect::<String>())
                        };
                        tokens.push(token);
                    };
                    end_token();
                    break Ok(DecisionMakingMode.boxed());
                }
                [_, ..] => {
                    next(view);
                    i += 1;
                }
                [] => break Ok(DecisionMakingMode.boxed()),
            }
        }
    }
}

struct PunctuationMode;

impl PunctuationMode {
    fn is_punctuation(c: char) -> bool {
        match c {
            ',' | ';' | ':' | '=' | '+' | '-' | '*' | '/' |
            '(' | ')' | '[' | ']' | '{' | '}' => true,
            _ => false,
        }
    }
}

impl LexerMode for PunctuationMode {
    fn tokenize(&mut self, view: &mut &[char], tokens: &mut Vec<Token>) -> Result<Box<dyn LexerMode>, LexerError> {
        let token = match view {
            ['(', ..] => Some((1, Token::LeftParenthesis)),
            [')', ..] => Some((1, Token::RightParenthesis)),
            ['[', ..] => Some((1, Token::LeftSquareBracket)),
            [']', ..] => Some((1, Token::RightSquareBracket)),
            ['{', ..] => Some((1, Token::LeftBrace)),
            ['}', ..] => Some((1, Token::RightBrace)),
            [',', ..] => Some((1, Token::Comma)),
            [';', ..] => Some((1, Token::Semicolon)),
            [':', ..] => Some((1, Token::Colon)),
            ['=', ..] => Some((1, Token::Equal)),
            ['+', ..] => Some((1, Token::Plus)),
            ['-', ..] => Some((1, Token::Minus)),
            ['*', '*', ..] => Some((2, Token::Pow)),
            ['*', ..] => Some((1, Token::Asterisk)),
            ['/', ..] => Some((1, Token::Slash)),
            _ => None,
        };
        if let Some((n, token)) = token {
            tokens.push(token);
            skip(view, n);
        }
        Ok(DecisionMakingMode.boxed())
    }
}

struct StringLiteralMode;

impl LexerMode for StringLiteralMode {
    fn tokenize(&mut self, view: &mut &[char], tokens: &mut Vec<Token>) -> Result<Box<dyn LexerMode>, LexerError> {
        next(view); // Skip first quotes
        let start = *view;
        let mut i = 0;
        loop {
            match view {
                ['\\', '"', ..] => {
                    skip(view, 2);
                    i += 2;
                }
                ['"', ..] => {
                    let str = &start[..i];
                    tokens.push(Token::StringLiteral(str.iter().collect::<String>()));
                    next(view);
                    break Ok(DecisionMakingMode.boxed());
                }
                [_, ..] => {
                    next(view);
                    i += 1;
                }
                [] => break Ok(DecisionMakingMode.boxed()),
            }
        }
    }
}

struct NumericLiteralMode;

impl LexerMode for NumericLiteralMode {
    fn tokenize(&mut self, view: &mut &[char], tokens: &mut Vec<Token>) -> Result<Box<dyn LexerMode>, LexerError> {
        let start = *view;
        let mut i = 0;
        let mut is_float = false;

        let mut is_sign_allowed = true;
        let mut is_point_allowed = true;

        loop {
            match view {
                ['+' | '-', ..] if is_sign_allowed => {
                    is_sign_allowed = false;
                    next(view);
                    i += 1;
                }
                ['.', ..] if is_point_allowed => {
                    is_point_allowed = false;
                    is_sign_allowed = false;
                    is_float = true;
                    next(view);
                    i += 1;
                }
                [d, ..] if d.is_ascii_digit() => {
                    is_sign_allowed = false;
                    next(view);
                    i += 1;
                }
                _ => {
                    let number = &start[..i].iter().collect::<String>();
                    if is_float {
                        let float = number.parse::<f32>().unwrap();
                        tokens.push(Token::Float(float));
                    } else {
                        let integer = number.parse::<i32>().unwrap();
                        tokens.push(Token::Integer(integer))
                    };
                    break Ok(DecisionMakingMode.boxed());
                }
            }
        }
    }
}

struct CommentMode;

impl LexerMode for CommentMode {
    fn tokenize(&mut self, view: &mut &[char], _tokens: &mut Vec<Token>) -> Result<Box<dyn LexerMode>, LexerError> {
        loop {
            match view {
                ['\n', ..] | [] => break Ok(DecisionMakingMode.boxed()),
                _ => next(view)
            }
        }
    }
}

pub struct Lexer2<'a> {
    view: &'a [char],
    tokens: Vec<Token>,
    mode: Box<dyn LexerMode>,
}

impl<'a> Lexer2<'a> {
    pub fn new(view: &'a [char]) -> Self {
        Lexer2 {
            view,
            tokens: vec![],
            mode: DecisionMakingMode.boxed(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        while !self.view.is_empty() {
            self.mode = self.mode.tokenize(&mut self.view, &mut self.tokens)?;
        }
        Ok(self.tokens.clone())
    }
}
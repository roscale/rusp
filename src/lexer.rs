// Architecture similar to this image
// https://miro.medium.com/max/875/1%2aluy_LfooQ8dLjhOiaZ1mrg.png

#[derive(Debug, Clone)]
pub enum Token {
    Id(String),
    Literal(Literal),
    Keyword(Keyword),
    Operator(Operator),
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBracket,
    RightSquareBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Colon,
    Dot,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Float(f32),
    Integer(i32),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Keyword {
    If,
    While,
    For,
    True,
    False,
    Fn,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Pow,
    Equal,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char),
}

pub struct Lexer<'a> {
    view: &'a [char],
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(view: &'a [char]) -> Self {
        Self {
            view,
            tokens: vec![],
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexerError> {
        loop {
            let is_the_last_token_an_operator = {
                match self.tokens.last() {
                    None => true,
                    Some(Token::Operator(_)) => true,
                    Some(_) => false,
                }
            };

            match self.view {
                [w, ..] if w.is_whitespace() => self.view = &self.view[1..],
                ['/', '/', ..] => self.process_comments()?,
                ['"', ..] => self.process_string_literals()?,
                [digit, ..] if digit.is_ascii_digit() => self.process_numeric_literals()?,
                ['+' | '-', digit, ..] if digit.is_ascii_digit() && is_the_last_token_an_operator => self.process_numeric_literals()?,
                [p, ..] if is_punctuation(*p) => self.process_operators_and_punctuation()?,
                [c, ..] if is_valid_identifier_character(*c) => self.process_keywords_and_identifiers()?,
                [e, ..] => return Err(LexerError::UnexpectedCharacter(*e)),
                [] => break,
            }
        }
        Ok(self.tokens)
    }

    fn process_keywords_and_identifiers(&mut self) -> Result<(), LexerError> {
        let start = self.view;
        let mut i = 0;

        fn end_token(start: &[char], i: usize) -> Option<Token> {
            use Keyword::*;
            match start.is_empty() {
                true => None,
                false => {
                    let token = start[..i].iter().collect::<String>();
                    let token = match token.as_str() {
                        "if" => Token::Keyword(If),
                        "while" => Token::Keyword(While),
                        "for" => Token::Keyword(For),
                        "true" => Token::Keyword(True),
                        "false" => Token::Keyword(False),
                        "fn" => Token::Keyword(Fn),
                        _ => Token::Id(start[..i].iter().collect::<String>())
                    };
                    Some(token)
                }
            }
        }

        loop {
            match self.view {
                [c, ..] if !is_valid_identifier_character(*c) => {
                    if let Some(token) = end_token(start, i) {
                        self.tokens.push(token);
                    }
                    break Ok(());
                }
                [] => {
                    if let Some(token) = end_token(start, i) {
                        self.tokens.push(token);
                    }
                    break Ok(());
                }
                [_, ..] => {
                    self.view = &self.view[1..];
                    i += 1;
                }
            }
        }
    }

    fn process_operators_and_punctuation(&mut self) -> Result<(), LexerError> {
        use Operator::*;
        let token = match self.view {
            ['(', ..] => Some((1, Token::LeftParenthesis)),
            [')', ..] => Some((1, Token::RightParenthesis)),
            ['[', ..] => Some((1, Token::LeftSquareBracket)),
            [']', ..] => Some((1, Token::RightSquareBracket)),
            ['{', ..] => Some((1, Token::LeftBrace)),
            ['}', ..] => Some((1, Token::RightBrace)),
            [',', ..] => Some((1, Token::Comma)),
            [';', ..] => Some((1, Token::Semicolon)),
            [':', ..] => Some((1, Token::Colon)),
            ['.', ..] => Some((1, Token::Dot)),
            ['=', ..] => Some((1, Token::Operator(Equal))),
            ['>', '=', ..] => Some((2, Token::Operator(GreaterThanOrEqual))),
            ['>', ..] => Some((1, Token::Operator(GreaterThan))),
            ['<', '=', ..] => Some((2, Token::Operator(LessThanOrEqual))),
            ['<', ..] => Some((1, Token::Operator(LessThan))),
            ['+', ..] => Some((1, Token::Operator(Plus))),
            ['-', ..] => Some((1, Token::Operator(Minus))),
            ['*', '*', ..] => Some((2, Token::Operator(Pow))),
            ['*', ..] => Some((1, Token::Operator(Asterisk))),
            ['/', ..] => Some((1, Token::Operator(Slash))),
            _ => None,
        };
        if let Some((n, token)) = token {
            self.tokens.push(token);
            self.view = &self.view[n..];
        }
        Ok(())
    }

    fn process_string_literals(&mut self) -> Result<(), LexerError> {
        self.view = &self.view[1..]; // Eat first quote
        let start = self.view;
        let mut i = 0;
        loop {
            match self.view {
                ['\\', '"', ..] => {
                    self.view = &self.view[2..];
                    i += 2;
                }
                ['"', ..] => {
                    let string = start[..i].iter().collect::<String>();
                    self.tokens.push(Token::Literal(Literal::String(string)));

                    self.view = &self.view[1..]; // Eat last quote
                    break Ok(());
                }
                [_, ..] => {
                    self.view = &self.view[1..];
                    i += 1;
                }
                [] => break Ok(()),
            }
        }
    }

    fn process_numeric_literals(&mut self) -> Result<(), LexerError> {
        let start = self.view;
        let mut i = 0;
        let mut is_float = false;

        let mut is_sign_allowed = true;
        let mut is_point_allowed = true;

        loop {
            match self.view {
                ['+' | '-', ..] if is_sign_allowed => {
                    is_sign_allowed = false;

                    self.view = &self.view[1..];
                    i += 1;
                }
                ['.', ..] if is_point_allowed => {
                    is_point_allowed = false;
                    is_sign_allowed = false;
                    is_float = true;

                    self.view = &self.view[1..];
                    i += 1;
                }
                [d, ..] if d.is_ascii_digit() => {
                    is_sign_allowed = false;

                    self.view = &self.view[1..];
                    i += 1;
                }
                _ => {
                    let number = &start[..i].iter().collect::<String>();
                    self.tokens.push(if is_float {
                        let float = number.parse::<f32>().unwrap();
                        Token::Literal(Literal::Float(float))
                    } else {
                        let integer = number.parse::<i32>().unwrap();
                        Token::Literal(Literal::Integer(integer))
                    });
                    break Ok(());
                }
            }
        }
    }

    fn process_comments(&mut self) -> Result<(), LexerError> {
        loop {
            match self.view {
                ['\n', ..] | [] => break Ok(()),
                _ => self.view = &self.view[1..]
            }
        }
    }
}

fn is_valid_identifier_character(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_punctuation(c: char) -> bool {
    match c {
        ',' | ';' | ':' | '=' | '+' | '-' | '*' | '/' | '.' |
        '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' => true,
        _ => false,
    }
}
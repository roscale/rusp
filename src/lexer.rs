// Architecture similar to this image
// https://miro.medium.com/max/875/1%2aluy_LfooQ8dLjhOiaZ1mrg.png

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
    Dot,
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

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        loop {
            match self.view {
                [w, ..] if w.is_whitespace() => self.view = &self.view[1..],
                ['/', '/', ..] => self.process_comments()?,
                ['"', ..] => self.process_string_literals()?,
                [digit, ..] if digit.is_ascii_digit() => self.process_numeric_literals()?,
                ['+' | '-', digit, ..] if digit.is_ascii_digit() => self.process_numeric_literals()?,
                [p, ..] if is_punctuation(*p) => self.process_operators_and_punctuation()?,
                [c, ..] if is_valid_identifier_character(*c) => self.process_keywords_and_identifiers()?,
                [e, ..] => return Err(LexerError::UnexpectedCharacter(*e)),
                [] => break,
            }
        }
        Ok(self.tokens.clone())
    }

    fn process_keywords_and_identifiers(&mut self) -> Result<(), LexerError> {
        let start = self.view;
        let mut i = 0;

        loop {
            match self.view {
                [c, ..] if !is_valid_identifier_character(*c) => {
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
                        self.tokens.push(token);
                    };
                    end_token();
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

    fn process_operators_and_punctuation(&mut self) -> Result<(), LexerError> {
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
            ['=', ..] => Some((1, Token::Equal)),
            ['+', ..] => Some((1, Token::Plus)),
            ['-', ..] => Some((1, Token::Minus)),
            ['*', '*', ..] => Some((2, Token::Pow)),
            ['*', ..] => Some((1, Token::Asterisk)),
            ['/', ..] => Some((1, Token::Slash)),
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
                    self.tokens.push(Token::StringLiteral(string));

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
                        Token::Float(float)
                    } else {
                        let integer = number.parse::<i32>().unwrap();
                        Token::Integer(integer)
                    });
                    break Ok(())
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
        '(' | ')' | '[' | ']' | '{' | '}' => true,
        _ => false,
    }
}
use std::ops::Range;

/// Architecture similar to this image:
/// https://miro.medium.com/max/875/1%2aluy_LfooQ8dLjhOiaZ1mrg.png
///
/// Initially, Lexer::view is the array of chars of the source code.
/// I opted to use a char array instead of using iterators because I can fully exploit pattern
/// matching to look ahead, instead of manually calling .peak() on an iterator.
/// As the lexer reads the characters, it re-slices the view. The next character to be read will
/// always be at index 0. It outputs a vector of Tokens to be used by the parser.
#[derive(Debug, Clone)]
pub enum Token {
    Id(String),
    Literal(Literal),
    Keyword(Keyword),
    Equal,
    Operator(Operator),
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBracket,
    RightSquareBracket,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
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
    Else,
    While,
    For,
    True,
    False,
    Fn,
    Let,
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(Range<usize>),
}

pub struct Lexer<'a> {
    chars: &'a [char],
    utf8_index: usize,
    tokens: Vec<Token>,
    indices: Vec<Range<usize>>,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: &'a [char]) -> Self {
        Self {
            chars,
            utf8_index: 0,
            tokens: vec![],
            indices: vec![],
        }
    }

    pub fn advance_by(&mut self, n: usize) {
        for char in &self.chars[..n] {
            self.utf8_index += char.len_utf8();
        }
        self.chars = &self.chars[n..];
    }

    pub fn add_token(&mut self, token: Token, range: Range<usize>) {
        self.tokens.push(token);
        self.indices.push(range);
    }

    pub fn tokenize(mut self) -> Result<(Vec<Token>, Vec<Range<usize>>), LexerError> {
        loop {
            match self.chars {
                [w, ..] if w.is_whitespace() => self.advance_by(1),
                ['/', '/', ..] => self.process_comments()?,
                ['"', ..] => self.process_string_literals()?,
                [digit, ..] if digit.is_ascii_digit() => self.process_numeric_literals()?,
                ['+' | '-', digit, ..] if digit.is_ascii_digit() => self.process_numeric_literals()?,
                // Special rules for the equal sign
                // "=" alone is reserved but it can be used in identifiers
                ['=', c, ..] if !is_valid_identifier_character(*c) => self.process_operators_and_punctuation()?,
                ['=', c, ..] if is_valid_identifier_character(*c) => self.process_keywords_and_identifiers()?,
                [p, ..] if is_punctuation(*p) => self.process_operators_and_punctuation()?,
                [c, ..] if is_valid_identifier_character(*c) => self.process_keywords_and_identifiers()?,
                [e, ..] => return Err(LexerError::UnexpectedCharacter(self.utf8_index..self.utf8_index + e.len_utf8())),
                [] => break,
            }
        }
        Ok((self.tokens, self.indices))
    }

    fn process_keywords_and_identifiers(&mut self) -> Result<(), LexerError> {
        let start_index = self.utf8_index;
        let start = self.chars;
        let mut i = 0;

        let end_token = |i: usize| -> Option<Token> {
            use Keyword::*;
            match start.is_empty() {
                true => None,
                false => {
                    let token = start[..i].iter().collect::<String>();
                    let token = match token.as_str() {
                        "if" => Token::Keyword(If),
                        "else" => Token::Keyword(Else),
                        "while" => Token::Keyword(While),
                        "for" => Token::Keyword(For),
                        "true" => Token::Keyword(True),
                        "false" => Token::Keyword(False),
                        "fn" => Token::Keyword(Fn),
                        "let" => Token::Keyword(Let),
                        _ => Token::Id(start[..i].iter().collect::<String>())
                    };
                    Some(token)
                }
            }
        };

        loop {
            match self.chars {
                [c, ..] if !is_valid_identifier_character(*c) => {
                    if let Some(token) = end_token(i) {
                        self.add_token(token, start_index..self.utf8_index)
                    }
                    break Ok(());
                }
                [] => {
                    if let Some(token) = end_token(i) {
                        self.add_token(token, start_index..self.utf8_index)
                    }
                    break Ok(());
                }
                [_, ..] => {
                    self.advance_by(1);
                    i += 1;
                }
            }
        }
    }

    fn process_operators_and_punctuation(&mut self) -> Result<(), LexerError> {
        let token = match self.chars {
            ['=', ..] => Some((1, Token::Equal)),
            ['+', ..] => Some((1, Token::Operator(Operator::Plus))),
            ['(', ..] => Some((1, Token::LeftParenthesis)),
            [')', ..] => Some((1, Token::RightParenthesis)),
            ['[', ..] => Some((1, Token::LeftSquareBracket)),
            [']', ..] => Some((1, Token::RightSquareBracket)),
            ['{', ..] => Some((1, Token::LeftBrace)),
            ['}', ..] => Some((1, Token::RightBrace)),
            _ => None,
        };
        if let Some((n, token)) = token {
            let start_index = self.utf8_index;
            self.advance_by(n);
            self.add_token(token, start_index..self.utf8_index)
        }
        Ok(())
    }

    fn process_string_literals(&mut self) -> Result<(), LexerError> {
        let start_index = self.utf8_index;

        self.advance_by(1); // Eat first quote
        let string_start = self.chars;
        let mut i = 0;
        loop {
            match self.chars {
                ['\\', '"', ..] => {
                    self.advance_by(2);
                    i += 2;
                }
                ['"', ..] => {
                    let string = string_start[..i].iter().collect::<String>();
                    self.advance_by(1); // Eat last quote

                    let token = Token::Literal(Literal::String(string));
                    self.add_token(token, start_index..self.utf8_index);

                    break Ok(());
                }
                [_, ..] => {
                    self.advance_by(1);
                    i += 1;
                }
                [] => break Ok(()),
            }
        }
    }

    fn process_numeric_literals(&mut self) -> Result<(), LexerError> {
        let start_index = self.utf8_index;
        let start = self.chars;
        let mut i = 0;
        let mut is_float = false;

        let mut is_sign_allowed = true;
        let mut is_point_allowed = true;

        loop {
            match self.chars {
                ['+' | '-', ..] if is_sign_allowed => {
                    is_sign_allowed = false;

                    self.advance_by(1);
                    i += 1;
                }
                ['.', ..] if is_point_allowed => {
                    is_point_allowed = false;
                    is_sign_allowed = false;
                    is_float = true;

                    self.advance_by(1);
                    i += 1;
                }
                [d, ..] if d.is_ascii_digit() => {
                    is_sign_allowed = false;

                    self.advance_by(1);
                    i += 1;
                }
                _ => {
                    let number = &start[..i].iter().collect::<String>();
                    let token = if is_float {
                        let float = number.parse::<f32>().unwrap();
                        Token::Literal(Literal::Float(float))
                    } else {
                        let integer = number.parse::<i32>().unwrap();
                        Token::Literal(Literal::Integer(integer))
                    };
                    self.add_token(token, start_index..self.utf8_index);
                    break Ok(());
                }
            }
        }
    }

    fn process_comments(&mut self) -> Result<(), LexerError> {
        loop {
            match self.chars {
                ['\n', ..] | [] => break Ok(()),
                _ => self.advance_by(1)
            }
        }
    }
}

fn is_valid_identifier_character(c: char) -> bool {
    match c {
        '+' | '(' | ')' | '[' | ']' | '{' | '}' => false,
        c if c.is_whitespace() => false,
        _ => true,
    }
}

fn is_punctuation(c: char) -> bool {
    match c {
        '=' | '+' | '(' | ')' | '[' | ']' | '{' | '}' => true,
        _ => false,
    }
}
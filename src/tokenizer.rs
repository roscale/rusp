use std::iter::Enumerate;
use std::str::Chars;

#[derive(Debug, Copy, Clone)]
pub enum Token<'a> {
    Id(&'a str),
    If,
    While,
    For,
    Number(&'a str),
    StringLiteral(&'a str),
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
}

impl<'a> Token<'a> {
    pub fn from_single_char(c: char) -> Self {
        match c {
            ',' => Self::Comma,
            ';' => Self::Semicolon,
            ':' => Self::Colon,
            '=' => Self::Equal,
            '+' => Self::Plus,
            '-' => Self::Minus,
            '*' => Self::Asterisk,
            '/' => Self::Slash,
            '(' => Self::LeftParenthesis,
            ')' => Self::RightParenthesis,
            '[' => Self::LeftSquareBracket,
            ']' => Self::RightSquareBracket,
            '{' => Self::LeftBrace,
            '}' => Self::RightBrace,
            _ => unimplemented!()
        }
    }

    pub fn from_symbol(symbol: &'a str) -> Self {
        match symbol {
            "if" => Token::If,
            "while" => Token::While,
            "for" => Token::For,
            _ => {
                if symbol.starts_with(|c: char| c.is_numeric()) {
                    Token::Number(symbol)
                } else {
                    Token::Id(symbol)
                }
            }
        }
    }
}

pub enum TokenType {
    Symbol,
    StringLiteral(char), // stores the opening/closing character, either ' or "
}

pub struct Tokenizer<'a> {
    source: &'a str,
    current_token_start_index: usize,
    starting_new_token: bool,
    current_token_type: TokenType,

    tokens: Vec<Token<'a>>,

    it: Enumerate<Chars<'a>>,
    current_character: Option<(usize, char)>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut this = Self {
            source,
            current_token_start_index: 0,
            starting_new_token: true,
            current_token_type: TokenType::Symbol,

            tokens: vec![],

            it: source.chars().enumerate(),
            current_character: None,
        };
        this.current_character = this.it.next();
        this
    }

    pub fn tokenize(mut self) -> Vec<Token<'a>> {
        while let Some((i, c)) = self.current_character {

            // String literals
            match self.current_token_type {
                // End
                TokenType::StringLiteral(closing_quote) if c == closing_quote => {
                    self.end_current_token();
                    self.current_token_type = TokenType::Symbol;
                    self.current_character = self.it.next();
                    continue;
                }
                // Middle
                TokenType::StringLiteral(_) => {
                    if c == '\\' {
                        self.it.next(); // Skip the next character
                    }
                    self.current_character = self.it.next();
                    continue;
                }
                // Start
                _ => if matches!(c, '\"' | '\'') {
                    self.end_current_token();
                    self.current_token_type = TokenType::StringLiteral(c);
                    self.current_character = self.it.next();
                    continue;
                }
            }

            match c {
                // Special characters and operators that terminate a token
                ',' | ';' | ':' | '=' | '+' | '-' | '*' | '/' |
                '(' | ')' | '[' | ']' | '{' | '}' => {
                    self.end_current_token();
                    self.tokens.push(Token::from_single_char(c));
                }
                whitespace if whitespace.is_whitespace() => {
                    self.end_current_token();
                }
                // Mark the beginning of the new token
                _ => {
                    if self.starting_new_token {
                        self.current_token_start_index = i;
                        self.starting_new_token = false;
                    }
                }
            }
            self.current_character = self.it.next();
        }
        self.end_current_token();
        self.tokens.clone()
    }

    pub fn end_current_token(&mut self) {
        let end_index = if let Some((i, _)) = self.current_character {
            i
        } else {
            self.source.len()
        };

        let token_str = &self.source[self.current_token_start_index..end_index];

        match self.current_token_type {
            TokenType::Symbol => {
                if !token_str.is_empty() {
                    self.tokens.push(Token::from_symbol(token_str));
                }
            }
            TokenType::StringLiteral(_) => {
                self.tokens.push(Token::StringLiteral(token_str));
            }
        }
        self.starting_new_token = true;
        self.current_token_start_index = end_index + 1;
    }
}


#[derive(Debug)]
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
    Newline,
    Tab,
}

impl Token<'_> {
    pub fn from_single_char(c: char) -> Self {
        match c {
            '\n' => Self::Newline,
            '\t' => Self::Tab,
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
}

pub enum TokenType {
    Symbol,
    StringLiteral(char), // stores the opening/closing character, either ' or "
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut current_token_start_index = 0;
    let mut new_token = false;
    let mut current_token_type = TokenType::Symbol;

    let mut tokens = vec![];

    let mut it = source.chars().enumerate();
    let mut current_character = it.next();

    while let Some((i, c)) = current_character {
        let mut end_current_token = || {
            let token_str = &source[current_token_start_index..i];

            match current_token_type {
                TokenType::Symbol => {
                    if !token_str.is_empty() {
                        if token_str.starts_with(|c: char| c.is_numeric()) {
                            tokens.push(Token::Number(token_str));
                        } else {
                            // Reserved keywords
                            match token_str {
                                "if" => tokens.push(Token::If),
                                "while" => tokens.push(Token::While),
                                "for" => tokens.push(Token::For),
                                _ => tokens.push(Token::Id(token_str))
                            }
                        }
                    }
                }
                TokenType::StringLiteral(_) => {
                    tokens.push(Token::StringLiteral(token_str));
                }
            }
            new_token = true;
            current_token_start_index = i + 1;
        };

        // String literals
        match current_token_type {
            // End
            TokenType::StringLiteral(closing_quote) if c == closing_quote => {
                end_current_token();
                current_token_type = TokenType::Symbol;
                current_character = it.next();
                continue;
            }
            // Middle
            TokenType::StringLiteral(_) => {
                if c == '\\' {
                    it.next(); // Skip the next character
                }
                current_character = it.next();
                continue;
            }
            // Start
            _ => if matches!(c, '\"' | '\'') {
                end_current_token();
                current_token_type = TokenType::StringLiteral(c);
                current_character = it.next();
                continue;
            }
        }

        match c {
            '\n' | '\t' | ',' | ';' | ':' | '=' | '+' | '-' | '*' | '/' | '(' |
            ')' | '[' | ']' | '{' | '}' => {
                end_current_token();
                tokens.push(Token::from_single_char(c));
            }
            whitespace if whitespace.is_whitespace() => {
                end_current_token();
            }
            _ => {
                if new_token {
                    current_token_start_index = i;
                    new_token = false;
                }
            }
        }

        current_character = it.next();
        if current_character.is_none() {
            end_current_token();
        }
    }
    tokens
}
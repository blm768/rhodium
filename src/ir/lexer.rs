use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use base::source::{SourceLocation, SourceText};
use ir::charclass;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Whitespace,
    Open,
    Close,
    Symbol,
    Integer,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub location: SourceLocation,
}

#[derive(Clone, Debug)]
pub struct LexicalError {
    pub location: SourceLocation,
}

impl Display for LexicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "unexpected token at {}", self.location)
    }
}

impl Error for LexicalError {
    fn description(&self) -> &str {
        "unexpected token"
    }
}

pub struct Lexer {
    source: Rc<SourceText>,
    offset: usize,
}

impl Lexer {
    pub fn new(source: Rc<SourceText>) -> Lexer {
        Lexer { source, offset: 0 }
    }

    pub fn source(&self) -> &Rc<SourceText> {
        &self.source
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    fn pop_token(&mut self, token_type: TokenType, length: usize) -> Token {
        let start = self.offset;
        self.offset += length;
        Token {
            token_type,
            location: SourceLocation::new(Rc::clone(&self.source), start, length),
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<Token, LexicalError>;

    fn next(&mut self) -> Option<Result<Token, LexicalError>> {
        let source = Rc::clone(&self.source);
        let remaining = &source.text()[self.offset..];

        if remaining.is_empty() {
            return None;
        }

        let first_char = remaining.chars().next().unwrap();

        if first_char == '(' {
            return Some(Ok(self.pop_token(TokenType::Open, '('.len_utf8())));
        }
        if first_char == ')' {
            return Some(Ok(self.pop_token(TokenType::Close, ')'.len_utf8())));
        }

        let white_len = charclass::match_length(remaining, charclass::is_whitespace);
        if white_len > 0 {
            return Some(Ok(self.pop_token(TokenType::Whitespace, white_len)));
        }

        let int_length = charclass::match_length(remaining, charclass::is_decimal_digit);
        if int_length > 0 {
            return Some(Ok(self.pop_token(TokenType::Integer, int_length)));
        }

        // TODO: just make symbols the "default" token type (if nothing else matches)?
        // (Allows for "non-identifier" symbols)
        if charclass::is_identifier_start(&first_char) {
            let first_len = first_char.len_utf8();
            let rest = &remaining[first_len..];
            let rest_len = charclass::match_length(rest, charclass::is_identifier);
            return Some(Ok(self.pop_token(TokenType::Symbol, first_len + rest_len)));
        }

        // If we get here, the token is invalid.
        // Stop lexing.
        let err_offset = self.offset;
        self.offset = source.len();
        Some(Err(LexicalError {
            location: SourceLocation::new(Rc::clone(&source), err_offset, 0),
        }))
    }
}

use std::rc::Rc;

use base;
use ir::charclass;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Invalid,
    Whitespace,
    Open,
    Close,
    Symbol,
    Integer,
}

pub struct Token {
    pub token_type: TokenType,
    pub location: base::SourceLocation,
}

pub struct Lexer {
    source: Rc<base::SourceText>,
    offset: usize,
}

impl Lexer {
    pub fn new(source: Rc<base::SourceText>) -> Lexer {
        Lexer { source: source, offset: 0 }
    }

    // TODO: move the Some to the caller?
    fn pop_token(&mut self, token_type: TokenType, length: usize) -> Option<Token> {
        let start = self.offset;
        self.offset += length;
        Some(Token {
            token_type: token_type,
            location: base::SourceLocation::new(self.source.clone(), start, length)
        })
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let source = self.source.clone();
        let remaining = &source.text()[self.offset ..];

        if remaining.len() == 0 {
            return None;
        }

        let first_char = remaining.chars().next().unwrap();

        if first_char == '(' {
            return self.pop_token(TokenType::Open, '('.len_utf8());
        }
        if first_char == ')' {
            return self.pop_token(TokenType::Close, ')'.len_utf8());
        }

        let white_len = charclass::match_length(remaining, charclass::is_whitespace);
        if white_len > 0 {
            return self.pop_token(TokenType::Whitespace, white_len);
        }

        let int_length = charclass::match_length(remaining, charclass::is_decimal_digit);
        if int_length > 0 {
            return self.pop_token(TokenType::Integer, int_length);
        }

        // TODO: just make this the "default" token type (if nothing else matches)?
        // (Allows for "non-identifier" symbols)
        if charclass::is_identifier_start(&first_char) {
            let first_len = first_char.len_utf8();
            let rest = &remaining[first_len ..];
            let rest_len = charclass::match_length(rest, charclass::is_identifier);
            return self.pop_token(TokenType::Symbol, first_len + rest_len);
        }

        // If we get here, the token is invalid.
        // Stop lexing.
        self.offset = source.len();
        self.pop_token(TokenType::Invalid, 0)
    }
}

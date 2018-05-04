use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::iter;

use base::source::SourceLocation;
use ir;
use ir::{Lexer, LexicalError};

pub enum ParseEventType {
    Error,
    Open { op_text: SourceLocation },
    Close,
    Integer,
}

pub struct ParseEvent {
    pub location: SourceLocation,
    pub event_type: ParseEventType,
}

impl ParseEvent {
    pub fn new(location: SourceLocation, event_type: ParseEventType) -> ParseEvent {
        ParseEvent {
            location: location,
            event_type: event_type,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ParseErrorCause {
    Lexical(LexicalError),
}

#[derive(Clone, Debug)]
pub struct ParseError {
    location: SourceLocation,
    cause: ParseErrorCause,
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.cause {
            ParseErrorCause::Lexical(ref err) => err.fmt(formatter),
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "parsing error"
    }
}

// Used internally by Parser
type FilteredLexer<'a> = iter::Filter<&'a mut Lexer, fn(&Result<ir::Token, LexicalError>) -> bool>;

pub struct Parser<'a> {
    lexer: FilteredLexer<'a>,
    level: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Parser<'a> {
        fn is_non_white(t: &Result<ir::Token, LexicalError>) -> bool {
            match t {
                Ok(ref tok) => tok.token_type != ir::TokenType::Whitespace,
                _ => true,
            }
        }

        let filtered = lexer.filter(is_non_white as fn(&Result<ir::Token, LexicalError>) -> bool);
        Parser {
            lexer: filtered,
            level: 0,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = ParseEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let next_token = self.lexer.next();

        match next_token {
            Some(Ok(token)) => {
                match token.token_type {
                    ir::TokenType::Whitespace => panic!("Whitespace should be filtered out"),
                    ir::TokenType::Open => {
                        let op_token = self.lexer.next();
                        match op_token {
                            Some(Ok(op_t)) => {
                                if op_t.token_type == ir::TokenType::Symbol {
                                    self.level += 1;

                                    Some(ParseEvent::new(
                                        SourceLocation::span(&token.location, &op_t.location),
                                        ParseEventType::Open {
                                            op_text: op_t.location.clone(),
                                        },
                                    ))
                                } else {
                                    Some(ParseEvent::new(op_t.location, ParseEventType::Error))
                                }
                            }
                            Some(Err(_)) => {
                                // TODO: propagate error info.
                                Some(ParseEvent::new(token.location, ParseEventType::Error))
                            }
                            None => Some(ParseEvent::new(token.location, ParseEventType::Error)),
                        }
                    }
                    ir::TokenType::Close => {
                        if self.level == 0 {
                            Some(ParseEvent::new(token.location, ParseEventType::Error))
                        } else {
                            self.level -= 1;
                            Some(ParseEvent::new(token.location, ParseEventType::Close))
                        }
                    }
                    ir::TokenType::Symbol => {
                        Some(ParseEvent::new(token.location, ParseEventType::Error))
                    }
                    ir::TokenType::Integer => {
                        Some(ParseEvent::new(token.location, ParseEventType::Integer))
                    }
                }
            }
            Some(Err(_)) => None, // TODO: handle errors better.
            None => None,
        }
    }
}

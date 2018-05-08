use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};
use std::iter::Filter;

use base::source::SourceLocation;
use ir;
use ir::lexer::{Lexer, LexicalError, Token};

/**
 * The type of a parsing event
 *
 * For open events, the location of the operator text is embedded in
 * this enum (for now…)
 */
pub enum ParseEventType {
    Open { op_text: SourceLocation },
    Close,
    Integer,
}

/**
 * A parsing event
 */
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
    Lexical,
    UnclosedParen,
    ExtraCloseParen,
    MisplacedSymbol,
    MissingOperation,
}

#[derive(Clone, Debug)]
pub struct ParseError {
    location: SourceLocation,
    cause: ParseErrorCause,
}

impl ParseError {
    pub fn new(location: SourceLocation, cause: ParseErrorCause) -> ParseError {
        ParseError {
            location: location,
            cause: cause,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.cause.fmt(formatter)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "parsing error"
    }
}

pub struct Parser {
    lexer: Lexer,
    level: usize,
}

impl<'a> Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser {
            lexer: lexer,
            level: 0,
        }
    }

    fn lexer_without_whitespace(
        &mut self,
    ) -> Filter<&mut Lexer, fn(&Result<Token, LexicalError>) -> bool> {
        fn is_non_white(t: &Result<ir::Token, LexicalError>) -> bool {
            match t {
                Ok(ref tok) => tok.token_type != ir::TokenType::Whitespace,
                _ => true,
            }
        }

        self.lexer.by_ref().filter(is_non_white)
    }
}

/*
 * TODO: remove the Iterator interface?
 *
 * (Or maybe return "nested" iterators for stuff inside operations instead of explicit open/close events?)
 */
impl<'a> Iterator for Parser {
    type Item = Result<ParseEvent, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_token = self.lexer_without_whitespace().next();

        match next_token {
            Some(Ok(token)) => match token.token_type {
                ir::TokenType::Whitespace => panic!("Whitespace should be filtered out"),
                ir::TokenType::Open => {
                    self.level += 1;

                    let op_token = self.lexer_without_whitespace().next();
                    match op_token {
                        Some(Ok(op_t)) => {
                            if op_t.token_type == ir::TokenType::Symbol {
                                Some(Ok(ParseEvent::new(
                                    SourceLocation::span(&token.location, &op_t.location),
                                    ParseEventType::Open {
                                        op_text: op_t.location.clone(),
                                    },
                                )))
                            } else {
                                Some(Err(ParseError::new(
                                    op_t.location,
                                    ParseErrorCause::MissingOperation,
                                )))
                            }
                        }
                        Some(Err(error)) => Some(Err(ParseError::new(
                            error.location,
                            ParseErrorCause::Lexical,
                        ))),
                        None => Some(Err(ParseError::new(
                            token.location,
                            ParseErrorCause::UnclosedParen,
                        ))),
                    }
                }
                ir::TokenType::Close => {
                    if self.level == 0 {
                        // TODO: make sure that higher-level code triggers this.
                        Some(Err(ParseError::new(
                            token.location,
                            ParseErrorCause::ExtraCloseParen,
                        )))
                    } else {
                        self.level -= 1;
                        Some(Ok(ParseEvent::new(token.location, ParseEventType::Close)))
                    }
                }
                ir::TokenType::Symbol => Some(Err(ParseError::new(
                    token.location,
                    ParseErrorCause::MisplacedSymbol,
                ))),
                ir::TokenType::Integer => {
                    Some(Ok(ParseEvent::new(token.location, ParseEventType::Integer)))
                }
            },
            Some(Err(error)) => Some(Err(ParseError::new(
                error.location,
                ParseErrorCause::Lexical,
            ))),
            None => if self.level == 0 {
                None
            } else {
                Some(Err(ParseError::new(
                    SourceLocation::new(self.lexer.source(), self.lexer.offset(), 0),
                    ParseErrorCause::UnclosedParen,
                )))
            },
        }
    }
}

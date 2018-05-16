use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

use base::source::SourceLocation;
use ir;
use ir::lexer::{Lexer, LexicalError};

pub struct Element<'a> {
    pub location: SourceLocation,
    pub data: ElementData<'a>,
}

impl<'a> Element<'a> {
    pub fn new(location: SourceLocation, data: ElementData) -> Element {
        Element { location, data }
    }
}

pub enum ElementData<'a> {
    Operation(OperationIterator<'a>),
    Integer,
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
        ParseError { location, cause }
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

/*
 * Helper functions for Parser
 */

fn next_non_white(lexer: &mut Lexer) -> Option<<Lexer as Iterator>::Item> {
    fn is_non_white(t: &Result<ir::Token, LexicalError>) -> bool {
        match t {
            Ok(ref tok) => tok.token_type != ir::TokenType::Whitespace,
            _ => true,
        }
    }

    lexer.find(is_non_white)
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser { lexer }
    }

    /**
     * Returns the next element (operation or atom)
     *
     * (We can't use the regular Iterator interface due to self-borrowing in the return value.)
     */
    pub fn next_element(&mut self) -> Option<Result<Element, ParseError>> {
        let next_token = next_non_white(&mut self.lexer);

        match next_token {
            Some(Ok(token)) => Some(match token.token_type {
                ir::TokenType::Whitespace => panic!("Whitespace should be filtered out"),
                ir::TokenType::Open => match OperationIterator::new(&mut self.lexer) {
                    Ok(iter) => Ok(Element::new(token.location, ElementData::Operation(iter))),
                    Err(error) => Err(error),
                },
                ir::TokenType::Close => Err(ParseError::new(
                    token.location,
                    ParseErrorCause::ExtraCloseParen,
                )),
                ir::TokenType::Symbol => Err(ParseError::new(
                    token.location,
                    ParseErrorCause::MisplacedSymbol,
                )),
                ir::TokenType::Integer => Ok(Element::new(token.location, ElementData::Integer)),
            }),
            Some(Err(error)) => Some(Err(ParseError::new(
                error.location,
                ParseErrorCause::Lexical,
            ))),
            None => None,
        }
    }
}

pub struct OperationIterator<'a> {
    pub op_text: SourceLocation,
    lexer: &'a mut Lexer,
}

impl<'a> OperationIterator<'a> {
    /**
     * Given a lexer that has just "seen" the opening parenthesis of an
     * operation, returns either an OperationIterator or a syntax error
     */
    fn new(lexer: &'a mut Lexer) -> Result<OperationIterator<'a>, ParseError> {
        let op_token = next_non_white(lexer);
        match op_token {
            Some(Ok(op_t)) => {
                if op_t.token_type == ir::TokenType::Symbol {
                    Ok(OperationIterator {
                        op_text: op_t.location,
                        lexer,
                    })
                } else {
                    Err(ParseError::new(
                        op_t.location,
                        ParseErrorCause::MissingOperation,
                    ))
                }
            }
            Some(Err(error)) => Err(ParseError::new(error.location, ParseErrorCause::Lexical)),
            None => Err(ParseError::new(
                SourceLocation::new(lexer.source(), lexer.offset(), 0),
                ParseErrorCause::UnclosedParen,
            )),
        }
    }

    pub fn next_element(&mut self) -> Option<Result<Element, ParseError>> {
        let next_token = next_non_white(self.lexer);

        match next_token {
            Some(Ok(token)) => match token.token_type {
                ir::TokenType::Whitespace => panic!("Whitespace should be filtered out"),
                ir::TokenType::Open => match OperationIterator::new(self.lexer) {
                    Ok(iter) => Some(Ok(Element::new(
                        token.location,
                        ElementData::Operation(iter),
                    ))),
                    Err(error) => Some(Err(error)),
                },
                ir::TokenType::Close => None,
                ir::TokenType::Symbol => Some(Err(ParseError::new(
                    token.location,
                    ParseErrorCause::MisplacedSymbol,
                ))),
                ir::TokenType::Integer => {
                    Some(Ok(Element::new(token.location, ElementData::Integer)))
                }
            },
            Some(Err(error)) => Some(Err(ParseError::new(
                error.location,
                ParseErrorCause::Lexical,
            ))),
            None => Some(Err(ParseError::new(
                SourceLocation::new(self.lexer.source(), self.lexer.offset(), 0),
                ParseErrorCause::UnclosedParen,
            ))),
        }
    }
}

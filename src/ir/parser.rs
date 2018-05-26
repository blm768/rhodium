use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use base::source::{SourceLocation, SourceText};
use ir::lexer::{Lexer, LexicalError, Token, TokenType};

pub struct Element<'a> {
    pub location: SourceLocation,
    pub data: ElementData<'a>,
}

impl<'a> Element<'a> {
    pub fn new(location: SourceLocation, data: ElementData) -> Element {
        Element { location, data }
    }
}

// TODO: parse integers and strings. (Handle in lexer?)
pub enum ElementData<'a> {
    Operation(OperationIterator<'a>),
    Integer,
    String,
}

#[derive(Clone, Debug)]
pub enum ParseErrorCause {
    Lexical,
    UnclosedParen,
    ExtraCloseParen,
    MisplacedSymbol,
    MissingOperation,
    UndefinedOperation,
    TrailingText,
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
    fn is_non_white(t: &Result<Token, LexicalError>) -> bool {
        match t {
            Ok(ref tok) => tok.token_type != TokenType::Whitespace,
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

    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(Rc::clone(self.lexer.source()), self.lexer.offset(), 0)
    }

    pub fn offset(&self) -> usize {
        self.lexer.offset()
    }

    pub fn source(&self) -> &Rc<SourceText> {
        &self.lexer.source()
    }

    pub fn expect_end_of_source(&mut self) -> Result<(), ParseError> {
        while let Some(tok) = self.lexer.next() {
            match tok {
                Ok(token) => {
                    if token.token_type != TokenType::Whitespace {
                        return Err(ParseError::new(
                            token.location,
                            ParseErrorCause::TrailingText,
                        ));
                    }
                }
                Err(error) => {
                    return Err(ParseError::new(error.location, ParseErrorCause::Lexical));
                }
            }
        }
        Ok(())
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
                TokenType::Whitespace => panic!("Whitespace should be filtered out"),
                TokenType::Open => match OperationIterator::new(&mut self.lexer) {
                    Ok(iter) => Ok(Element::new(token.location, ElementData::Operation(iter))),
                    Err(error) => Err(error),
                },
                TokenType::Close => Err(ParseError::new(
                    token.location,
                    ParseErrorCause::ExtraCloseParen,
                )),
                TokenType::Symbol => Err(ParseError::new(
                    token.location,
                    ParseErrorCause::MisplacedSymbol,
                )),
                TokenType::Integer => Ok(Element::new(token.location, ElementData::Integer)),
                TokenType::String => Ok(Element::new(token.location, ElementData::String)),
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
                if op_t.token_type == TokenType::Symbol {
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
                SourceLocation::new(Rc::clone(lexer.source()), lexer.offset(), 0),
                ParseErrorCause::UnclosedParen,
            )),
        }
    }

    pub fn next_element(&mut self) -> Option<Result<Element, ParseError>> {
        let next_token = next_non_white(self.lexer);

        match next_token {
            Some(Ok(token)) => match token.token_type {
                TokenType::Whitespace => panic!("Whitespace should be filtered out"),
                TokenType::Open => match OperationIterator::new(self.lexer) {
                    Ok(iter) => Some(Ok(Element::new(
                        token.location,
                        ElementData::Operation(iter),
                    ))),
                    Err(error) => Some(Err(error)),
                },
                TokenType::Close => None,
                TokenType::Symbol => Some(Err(ParseError::new(
                    token.location,
                    ParseErrorCause::MisplacedSymbol,
                ))),
                TokenType::Integer => Some(Ok(Element::new(token.location, ElementData::Integer))),
                TokenType::String => Some(Ok(Element::new(token.location, ElementData::String))),
            },
            Some(Err(error)) => Some(Err(ParseError::new(
                error.location,
                ParseErrorCause::Lexical,
            ))),
            None => Some(Err(ParseError::new(
                SourceLocation::new(Rc::clone(self.lexer.source()), self.lexer.offset(), 0),
                ParseErrorCause::UnclosedParen,
            ))),
        }
    }
}

use std::iter;

use base::SourceLocation;
use ir;

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
        ParseEvent { location: location, event_type: event_type }
    }
}

// Used internally by Parser
type FilteredLexer<'a> = iter::Filter<&'a mut ir::Lexer, fn(&ir::Token) -> bool>;

pub struct Parser<'a> where {
    lexer: FilteredLexer<'a>,
}

impl<'a> Parser <'a> {
    pub fn new(lexer: &'a mut ir::Lexer) -> Parser<'a> {
        fn is_non_white(t: &ir::Token) -> bool { t.token_type != ir::TokenType::Whitespace }
        // Force the function to the correct type.
        // TODO: figure out why this is necessary (see
        // http://stackoverflow.com/questions/34459976/; there should be an implicit cast.)
        let filter: fn(&ir::Token) -> bool = is_non_white;
        let filtered = lexer.filter(filter);
        Parser { lexer: filtered }
    }
}

// TODO: figure out how we'll handle unbalanced parentheses.
impl<'a> Iterator for Parser<'a> {
    type Item = ParseEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let next_token = self.lexer.next();

        match next_token {
            Some(token) => {
                let loc = token.location.clone();

                match token.token_type {
                    ir::TokenType::Invalid => None,
                    ir::TokenType::Whitespace => panic!("Whitespace should be filtered out"),
                    ir::TokenType::Open => {
                        let op_token = self.lexer.next();
                        match op_token {
                            Some(op_t) => {
                                if op_t.token_type == ir::TokenType::Symbol {
                                    let op_t_loc = op_t.location.clone();
                                    Some(ParseEvent::new(
                                        SourceLocation::span(&loc, &op_t_loc),
                                        ParseEventType::Open { op_text: op_t_loc }
                                    ))
                                } else {
                                    Some(ParseEvent::new(op_t.location, ParseEventType::Error))
                                }
                            },
                            None => Some(ParseEvent::new(token.location, ParseEventType::Error)),
                        }
                    },
                    ir::TokenType::Close => Some(ParseEvent::new(token.location, ParseEventType::Close)),
                    ir::TokenType::Symbol => Some(ParseEvent::new(token.location, ParseEventType::Error)),
                    ir::TokenType::Integer => Some(ParseEvent::new(token.location, ParseEventType::Integer)),
                }
            },
            None => None
        }
    }
}

use std::error;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug)]
pub struct SourceText {
    text: String,
}

impl SourceText {
    pub fn new(text: String) -> SourceText {
        SourceText { text }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

// TODO: split into "location" and "range"?
// TODO: allow retrieval of line numbers.
// Line nums must be calculated before we get our hands on a SourceLocation pointing into the line.
#[derive(Clone, Debug)]
pub struct SourceLocation {
    source: Rc<SourceText>,
    offset: usize, // TODO: store a Range instead?
    length: usize,
}

impl SourceLocation {
    pub fn new(source: Rc<SourceText>, offset: usize, length: usize) -> SourceLocation {
        SourceLocation {
            source,
            offset,
            length,
        }
    }

    pub fn end(&self) -> usize {
        self.offset + self.length
    }

    // TODO: make this order-independent?
    pub fn span(loc1: &SourceLocation, loc2: &SourceLocation) -> SourceLocation {
        assert!(Rc::ptr_eq(&loc1.source, &loc2.source));
        assert!(loc1.offset <= loc2.end());

        SourceLocation {
            source: Rc::clone(&loc1.source),
            offset: loc1.offset,
            length: loc2.end() - loc1.offset,
        }
    }

    pub fn text(&self) -> &str {
        &self.source.text[self.offset..self.offset + self.length]
    }
}

impl Display for SourceLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // TODO: improve the display format.
        write!(formatter, "{}:{}", self.offset, self.offset + self.length)
    }
}

/**
 * An error that is associated with a specific location in a `SourceText`
 */
pub trait Error: error::Error {
    fn location(&self) -> SourceLocation;
}

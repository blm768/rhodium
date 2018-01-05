pub mod context;
pub mod operation;
pub mod symbol;
pub mod value;

use std::rc::Rc;

pub struct SourceText {
    text: String,
}

impl SourceText {
    pub fn new(text: String) -> SourceText {
        SourceText { text: text }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }
}

// TODO: split into "location" and "range"?
// TODO: allow retrieval of line numbers.
// Line nums must be calculated before we get our hands on a SourceLocation pointing into the line.
#[derive(Clone)]
pub struct SourceLocation {
    source: Rc<SourceText>,
    offset: usize, // TODO: store a Range instead?
    length: usize,
}

impl SourceLocation {
    pub fn new(source: Rc<SourceText>, offset: usize, length: usize) -> SourceLocation {
        SourceLocation {
            source: source,
            offset: offset,
            length: length,
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
            source: loc1.source.clone(),
            offset: loc1.offset,
            length: loc2.end() - loc1.offset,
        }
    }


    pub fn text(&self) -> &str {
        &self.source.text[self.offset..self.offset + self.length]
    }
}

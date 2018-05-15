use std::cell::RefCell;
use std::rc::{Rc, Weak};

use base::operation;
use base::symbol::SymbolTable;
use base::value::{Value, ValueResult};

#[derive(Debug, Default)]
pub struct Scope {
    symbols: SymbolTable,
    parent: Weak<RefCell<Scope>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            symbols: SymbolTable::new(),
            parent: Weak::default(),
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        match self.symbols.get(name) {
            Some(value) => Some(value.clone()),
            None => {
                let parent = self.parent.upgrade()?;
                let borrowed = parent.borrow();
                borrowed.lookup(name)
            }
        }
    }

    /**
     * Defines a named symbol
     *
     * If the name already exists, an error is returned with the value parameter contained inside.
     *
     * TODO: wrap the value in something?
     */
    pub fn define_symbol(&mut self, name: &str, value: Value) -> Result<(), Value> {
        self.symbols.insert(name, value)
    }
}

#[derive(Clone, Debug)]
pub struct EvaluationContext {
    scope: Rc<RefCell<Scope>>,
}

impl EvaluationContext {
    pub fn new(scope: Rc<RefCell<Scope>>) -> EvaluationContext {
        EvaluationContext { scope }
    }

    pub fn scope(&self) -> Rc<RefCell<Scope>> {
        Rc::clone(&self.scope)
    }
}

impl operation::EvaluationContext for EvaluationContext {
    type Value = ValueResult;
}

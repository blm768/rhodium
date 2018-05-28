use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use base::expression;
use base::expression::EvaluationListener;
use base::symbol::SymbolTable;
use base::value::{PartialExpression, Value, ValueResult};

#[derive(Debug, Default)]
pub struct Scope {
    symbols: SymbolTable,
    parent: Weak<RefCell<Scope>>,
    listeners: HashMap<Box<str>, Vec<Weak<PartialExpression>>>,
}

pub enum LookupResult {
    Total(Value),
    Pending,
    NotFound,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            symbols: SymbolTable::new(),
            parent: Weak::default(),
            listeners: HashMap::new(),
        }
    }

    pub fn get_symbol(&self, name: &str) -> LookupResult {
        match self.symbols.get(name) {
            Some(value) => LookupResult::Total(value.clone()),
            None => {
                if self.symbols.finalized() {
                    match self.parent.upgrade() {
                        Some(ref parent) => parent.borrow().get_symbol(name),
                        None => LookupResult::NotFound,
                    }
                } else {
                    LookupResult::Pending
                }
            }
        }
    }

    /**
     * Defines a named symbol
     *
     * If the name already exists, an error is returned with the value parameter contained inside.
     *
     * TODO: figure out how we'll represent non-total values.
     */
    pub fn define_symbol(&mut self, name: &str, value: Value) -> Result<(), Value> {
        self.symbols.insert(name, value.clone())?;
        if let Some(listeners) = self.listeners.remove(name) {
            for listener in listeners.iter().filter_map({ |l| l.upgrade() }) {
                listener.on_evaluated(&*listener, Ok(value.clone()));
            }
        }
        Ok(())
    }

    pub fn register_listener(&mut self, name: &str, listener: Weak<PartialExpression>) {
        self.listeners
            .entry(name.into())
            .or_insert_with(Vec::new)
            .push(listener);
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

impl expression::EvaluationContext for EvaluationContext {
    type Value = ValueResult;
}

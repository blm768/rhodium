use std::cell::RefCell;
use std::rc::Weak;

use base::symbol::SymbolTable;
use base::value::Value;

struct EvaluationContext {
    symbols: SymbolTable,
    parent: Weak<RefCell<EvaluationContext>>
}

impl EvaluationContext {
    fn lookup(name: &str) -> Option<Value> {
        // TODO: implement.
        None
    }
}

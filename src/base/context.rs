use std::cell::RefCell;
use std::rc::Weak;

use base::symbol::Symbol;

struct SymbolTable {}

struct EvaluationContext {
    symbols: SymbolTable,
    parent: Weak<RefCell<EvaluationContext>>
}

impl EvaluationContext {
    fn lookup(symbol: Symbol) -> Option<Symbol> {
        // TODO: implement.
        None
    }
}

use std::collections::HashMap;

use base::value::Value;

pub struct SymbolTable {
    entries: HashMap<Box<str>, Value>,
}

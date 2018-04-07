use std::collections::HashMap;

use base::value::Value;

#[derive(Debug, Default)]
pub struct SymbolTable {
    entries: HashMap<Box<str>, Value>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.entries.get(name)
    }

    pub fn insert(&mut self, name: &str, value: Value) -> Result<(), Value> {
        if let Some(old_val) = self.entries.get(name) {
            return Err(old_val.clone());
        }
        self.entries.insert(Box::from(name), value);
        Ok(())
    }
}

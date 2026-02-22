use std::collections::HashMap;

/// Symbol metadata placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub index: usize,
}

/// Symbol table placeholder.
#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    next_index: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, name: impl Into<String>) -> Symbol {
        // TODO(step-6): support scopes and symbol kinds.
        let name = name.into();
        let symbol = Symbol {
            name: name.clone(),
            index: self.next_index,
        };
        self.next_index += 1;
        self.symbols.insert(name, symbol.clone());
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

/// Stable builtin symbol ordering used by compiler symbol registration.
pub const BUILTIN_NAMES: &[&str] = &["len", "first", "last", "rest", "push", "puts"];

/// Symbol scope classification for compiler name resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolScope {
    Global,
    Local,
    Builtin,
    Free,
    Function,
}

impl Display for SymbolScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = match self {
            SymbolScope::Global => "Global",
            SymbolScope::Local => "Local",
            SymbolScope::Builtin => "Builtin",
            SymbolScope::Free => "Free",
            SymbolScope::Function => "Function",
        };
        write!(f, "{name}")
    }
}

/// A resolved symbol descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: usize,
}

impl Symbol {
    pub fn new(name: impl Into<String>, scope: SymbolScope, index: usize) -> Self {
        Self {
            name: name.into(),
            scope,
            index,
        }
    }
}

pub type SymbolTableRef = Rc<RefCell<SymbolTable>>;

/// Lexical symbol table used by compiler frontend.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub store: HashMap<String, Symbol>,
    pub outer: Option<SymbolTableRef>,
    pub num_definitions: usize,
    pub free_symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_enclosed(outer: SymbolTableRef) -> Self {
        Self {
            outer: Some(outer),
            ..Self::default()
        }
    }

    pub fn into_ref(self) -> SymbolTableRef {
        Rc::new(RefCell::new(self))
    }

    pub fn define(&mut self, name: impl Into<String>) -> Symbol {
        let name = name.into();
        let scope = if self.outer.is_none() {
            SymbolScope::Global
        } else {
            SymbolScope::Local
        };

        if let Some(existing) = self.store.get(&name) {
            if existing.scope == scope {
                return existing.clone();
            }
        }

        let symbol = Symbol::new(name.clone(), scope, self.num_definitions);
        self.store.insert(name, symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    pub fn define_builtin(&mut self, index: usize, name: impl Into<String>) -> Symbol {
        let name = name.into();
        let symbol = Symbol::new(name.clone(), SymbolScope::Builtin, index);
        self.store.insert(name, symbol.clone());
        symbol
    }

    pub fn define_function_name(&mut self, name: impl Into<String>) -> Symbol {
        let name = name.into();
        let symbol = Symbol::new(name.clone(), SymbolScope::Function, 0);
        self.store.insert(name, symbol.clone());
        symbol
    }

    pub fn resolve(&mut self, name: &str) -> Option<Symbol> {
        if let Some(symbol) = self.store.get(name) {
            return Some(symbol.clone());
        }

        let outer = self.outer.clone()?;
        let resolved = outer.borrow_mut().resolve(name)?;

        match resolved.scope {
            SymbolScope::Global | SymbolScope::Builtin => Some(resolved),
            SymbolScope::Local | SymbolScope::Free | SymbolScope::Function => {
                Some(self.define_free(resolved))
            }
        }
    }

    fn define_free(&mut self, original: Symbol) -> Symbol {
        if let Some(existing) = self.store.get(&original.name) {
            if existing.scope == SymbolScope::Free {
                return existing.clone();
            }
        }

        let symbol = Symbol::new(
            original.name.clone(),
            SymbolScope::Free,
            self.free_symbols.len(),
        );
        self.free_symbols.push(original);
        self.store.insert(symbol.name.clone(), symbol.clone());
        symbol
    }
}

pub fn define_builtins(table: &mut SymbolTable) {
    // TODO(step-10): compiler will consume builtin symbol indices for GetBuiltin emission.
    for (index, &name) in BUILTIN_NAMES.iter().enumerate() {
        table.define_builtin(index, name);
    }
}

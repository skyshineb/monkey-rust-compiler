use monkey_rust_compiler::builtins::builtin_names;
use monkey_rust_compiler::symbol_table::{
    define_builtins, Symbol, SymbolScope, SymbolTable, BUILTIN_NAMES,
};

#[test]
fn defines_global_symbols_with_stable_indices() {
    let mut table = SymbolTable::new();

    let a = table.define("a");
    let b = table.define("b");
    let c = table.define("c");

    assert_eq!(a, Symbol::new("a", SymbolScope::Global, 0));
    assert_eq!(b, Symbol::new("b", SymbolScope::Global, 1));
    assert_eq!(c, Symbol::new("c", SymbolScope::Global, 2));
}

#[test]
fn redefine_global_symbol_reuses_slot() {
    let mut table = SymbolTable::new();

    let first = table.define("x");
    let second = table.define("x");

    assert_eq!(first, Symbol::new("x", SymbolScope::Global, 0));
    assert_eq!(second, Symbol::new("x", SymbolScope::Global, 0));
    assert_eq!(table.num_definitions, 1);
}

#[test]
fn defines_local_symbols_in_enclosed_scope() {
    let mut global = SymbolTable::new();
    let g = global.define("a");

    let global_ref = global.into_ref();
    let mut local = SymbolTable::new_enclosed(global_ref.clone());

    let b = local.define("b");
    let c = local.define("c");

    assert_eq!(g, Symbol::new("a", SymbolScope::Global, 0));
    assert_eq!(b, Symbol::new("b", SymbolScope::Local, 0));
    assert_eq!(c, Symbol::new("c", SymbolScope::Local, 1));

    let resolved_global = local.resolve("a").expect("resolve global from local");
    assert_eq!(resolved_global, Symbol::new("a", SymbolScope::Global, 0));
}

#[test]
fn redefine_local_symbol_reuses_slot() {
    let global_ref = SymbolTable::new().into_ref();
    let mut local = SymbolTable::new_enclosed(global_ref);

    let first = local.define("x");
    let second = local.define("x");

    assert_eq!(first, Symbol::new("x", SymbolScope::Local, 0));
    assert_eq!(second, Symbol::new("x", SymbolScope::Local, 0));
    assert_eq!(local.num_definitions, 1);
}

#[test]
fn builtins_are_defined_with_stable_indices_and_resolve_from_nested_scopes() {
    let mut root = SymbolTable::new();
    define_builtins(&mut root);

    for (i, name) in BUILTIN_NAMES.iter().enumerate() {
        let symbol = root.resolve(name).expect("builtin should resolve in root");
        assert_eq!(symbol, Symbol::new(*name, SymbolScope::Builtin, i));
    }

    let root_ref = root.into_ref();
    let mut nested = SymbolTable::new_enclosed(root_ref);

    for (i, name) in BUILTIN_NAMES.iter().enumerate() {
        let symbol = nested
            .resolve(name)
            .expect("builtin should resolve from nested scope");
        assert_eq!(symbol, Symbol::new(*name, SymbolScope::Builtin, i));
    }
}

#[test]
fn builtin_can_be_shadowed_by_global_definition() {
    let mut table = SymbolTable::new();
    define_builtins(&mut table);

    let symbol = table.define("len");
    assert_eq!(symbol, Symbol::new("len", SymbolScope::Global, 0));
    assert_eq!(
        table.resolve("len"),
        Some(Symbol::new("len", SymbolScope::Global, 0))
    );
    assert_eq!(table.num_definitions, 1);
}

#[test]
fn resolve_local_global_and_unknown_symbols() {
    let mut root = SymbolTable::new();
    root.define("a");

    let root_ref = root.into_ref();
    let mut nested = SymbolTable::new_enclosed(root_ref);
    nested.define("a");
    nested.define("b");

    assert_eq!(
        nested.resolve("a"),
        Some(Symbol::new("a", SymbolScope::Local, 0))
    );
    assert_eq!(
        nested.resolve("b"),
        Some(Symbol::new("b", SymbolScope::Local, 1))
    );
    assert_eq!(nested.resolve("missing"), None);
}

#[test]
fn resolves_nested_free_symbols_deterministically() {
    let mut global = SymbolTable::new();
    global.define("a");

    let global_ref = global.into_ref();

    let mut local1 = SymbolTable::new_enclosed(global_ref.clone());
    local1.define("b");
    local1.define("c");
    let local1_ref = local1.into_ref();

    let mut local2 = SymbolTable::new_enclosed(local1_ref);
    local2.define("d");

    assert_eq!(
        local2.resolve("a"),
        Some(Symbol::new("a", SymbolScope::Global, 0))
    );
    assert_eq!(
        local2.resolve("b"),
        Some(Symbol::new("b", SymbolScope::Free, 0))
    );
    assert_eq!(
        local2.resolve("c"),
        Some(Symbol::new("c", SymbolScope::Free, 1))
    );
    assert_eq!(
        local2.resolve("d"),
        Some(Symbol::new("d", SymbolScope::Local, 0))
    );

    assert_eq!(
        local2.free_symbols,
        vec![
            Symbol::new("b", SymbolScope::Local, 0),
            Symbol::new("c", SymbolScope::Local, 1),
        ]
    );
}

#[test]
fn free_symbol_resolution_is_deduplicated() {
    let mut global = SymbolTable::new();
    global.define("a");
    let global_ref = global.into_ref();

    let mut local1 = SymbolTable::new_enclosed(global_ref);
    local1.define("b");
    let local1_ref = local1.into_ref();

    let mut local2 = SymbolTable::new_enclosed(local1_ref);

    let first = local2.resolve("b").expect("first free resolve");
    let free_count_after_first = local2.free_symbols.len();
    let second = local2.resolve("b").expect("second free resolve");

    assert_eq!(first, second);
    assert_eq!(first, Symbol::new("b", SymbolScope::Free, 0));
    assert_eq!(free_count_after_first, 1);
    assert_eq!(local2.free_symbols.len(), 1);
}

#[test]
fn unknown_symbol_returns_none() {
    let mut table = SymbolTable::new();
    assert_eq!(table.resolve("nope"), None);
}

#[test]
fn function_scope_symbol_can_be_captured_as_free() {
    let mut root = SymbolTable::new();
    let f = root.define_function_name("f");
    assert_eq!(f, Symbol::new("f", SymbolScope::Function, 0));

    let root_ref = root.into_ref();
    let mut nested = SymbolTable::new_enclosed(root_ref);

    let resolved = nested.resolve("f").expect("function symbol should resolve");
    assert_eq!(resolved, Symbol::new("f", SymbolScope::Free, 0));

    let resolved_again = nested
        .resolve("f")
        .expect("function symbol should resolve again");
    assert_eq!(resolved_again, Symbol::new("f", SymbolScope::Free, 0));
    assert_eq!(nested.free_symbols.len(), 1);
    assert_eq!(
        nested.free_symbols[0],
        Symbol::new("f", SymbolScope::Function, 0)
    );
}

#[test]
fn builtin_constant_order_matches_builtin_registry() {
    assert_eq!(BUILTIN_NAMES, builtin_names());
}

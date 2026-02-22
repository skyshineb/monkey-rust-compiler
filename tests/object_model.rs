use std::rc::Rc;

use monkey_rust_compiler::object::{
    BuiltinObject, ClosureObject, CompiledFunctionObject, HashKey, Object,
};
use monkey_rust_compiler::position::Position;

fn int(v: i64) -> Rc<Object> {
    Object::Integer(v).rc()
}

fn str_obj(v: &str) -> Rc<Object> {
    Object::String(v.to_string()).rc()
}

#[test]
fn type_name_is_stable_for_all_supported_variants() {
    let compiled = Rc::new(CompiledFunctionObject {
        name: Some("adder".to_string()),
        num_params: 2,
        num_locals: 1,
        instructions: vec![1, 2, 3],
        positions: vec![(0, Position::new(1, 1))],
    });
    let closure = Rc::new(ClosureObject {
        function: Rc::clone(&compiled),
        free: vec![int(1)],
    });

    let cases = vec![
        (Object::Integer(1), "INTEGER"),
        (Object::Boolean(true), "BOOLEAN"),
        (Object::String("x".to_string()), "STRING"),
        (Object::Null, "NULL"),
        (Object::Array(vec![int(1)]), "ARRAY"),
        (Object::Hash(vec![(str_obj("a"), int(1))]), "HASH"),
        (Object::CompiledFunction(compiled), "FUNCTION"),
        (Object::Closure(closure), "CLOSURE"),
        (
            Object::Builtin(BuiltinObject {
                name: "len".to_string(),
            }),
            "BUILTIN",
        ),
    ];

    for (obj, expected) in cases {
        assert_eq!(obj.type_name(), expected);
    }
}

#[test]
fn truthiness_matches_monkey_rules() {
    assert!(!Object::Boolean(false).is_truthy());
    assert!(!Object::Null.is_truthy());

    assert!(Object::Boolean(true).is_truthy());
    assert!(Object::Integer(0).is_truthy());
    assert!(Object::String("".to_string()).is_truthy());
    assert!(Object::Array(vec![]).is_truthy());
    assert!(Object::Hash(vec![]).is_truthy());
}

#[test]
fn hash_key_is_only_defined_for_hashable_types() {
    assert_eq!(Object::Integer(7).hash_key(), Some(HashKey::Integer(7)));
    assert_eq!(
        Object::Boolean(true).hash_key(),
        Some(HashKey::Boolean(true))
    );
    assert_eq!(
        Object::String("abc".to_string()).hash_key(),
        Some(HashKey::String("abc".to_string()))
    );

    let compiled = Rc::new(CompiledFunctionObject {
        name: None,
        num_params: 0,
        num_locals: 0,
        instructions: vec![],
        positions: vec![],
    });
    let closure = Rc::new(ClosureObject {
        function: Rc::clone(&compiled),
        free: vec![],
    });

    assert_eq!(Object::Null.hash_key(), None);
    assert_eq!(Object::Array(vec![int(1)]).hash_key(), None);
    assert_eq!(Object::Hash(vec![]).hash_key(), None);
    assert_eq!(Object::CompiledFunction(compiled).hash_key(), None);
    assert_eq!(Object::Closure(closure).hash_key(), None);
    assert_eq!(
        Object::Builtin(BuiltinObject {
            name: "len".to_string()
        })
        .hash_key(),
        None
    );
}

#[test]
fn inspect_formatting_is_deterministic() {
    let compiled_named = Object::CompiledFunction(Rc::new(CompiledFunctionObject {
        name: Some("sum".to_string()),
        num_params: 2,
        num_locals: 2,
        instructions: vec![1, 2, 3],
        positions: vec![(0, Position::new(1, 1))],
    }));
    let compiled_anon = Object::CompiledFunction(Rc::new(CompiledFunctionObject {
        name: None,
        num_params: 0,
        num_locals: 0,
        instructions: vec![],
        positions: vec![],
    }));
    let closure = Object::Closure(Rc::new(ClosureObject {
        function: Rc::new(CompiledFunctionObject {
            name: Some("sum".to_string()),
            num_params: 2,
            num_locals: 2,
            instructions: vec![1],
            positions: vec![(0, Position::new(1, 1))],
        }),
        free: vec![int(99)],
    }));
    let builtin = Object::Builtin(BuiltinObject {
        name: "len".to_string(),
    });

    assert_eq!(Object::Integer(123).inspect(), "123");
    assert_eq!(Object::Boolean(true).inspect(), "true");
    assert_eq!(Object::Boolean(false).inspect(), "false");
    assert_eq!(Object::String("abc".to_string()).inspect(), "abc");
    assert_eq!(Object::Null.inspect(), "null");

    assert_eq!(
        Object::Array(vec![int(1), Object::Boolean(true).rc()]).inspect(),
        "[1, true]"
    );
    assert_eq!(
        Object::Hash(vec![(str_obj("a"), int(1)), (str_obj("b"), int(2))]).inspect(),
        "{a: 1, b: 2}"
    );

    assert_eq!(compiled_named.inspect(), "<compiled fn:sum>");
    assert_eq!(compiled_anon.inspect(), "<compiled fn>");
    assert_eq!(closure.inspect(), "<closure>");
    assert_eq!(builtin.inspect(), "<builtin: len>");
}

#[test]
fn hash_inspect_preserves_pair_order() {
    let hash = Object::Hash(vec![
        (str_obj("first"), int(1)),
        (str_obj("second"), int(2)),
        (str_obj("third"), int(3)),
    ]);
    assert_eq!(hash.inspect(), "{first: 1, second: 2, third: 3}");
}

#[test]
fn object_ref_helpers_support_shared_ownership() {
    let shared = Object::Integer(42).rc();

    let array = Object::Array(vec![Rc::clone(&shared), Rc::clone(&shared)]);
    assert_eq!(array.inspect(), "[42, 42]");

    let hash = Object::Hash(vec![(
        Object::String("k".to_string()).rc(),
        Rc::clone(&shared),
    )]);
    assert_eq!(hash.inspect(), "{k: 42}");

    assert_eq!(*shared, Object::Integer(42));
}

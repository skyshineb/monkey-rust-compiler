use monkey_rust_compiler::ast::{BlockStatement, Expression, Identifier, Program, Statement};
use monkey_rust_compiler::position::Position;
use monkey_rust_compiler::pretty;

fn p(line: usize, col: usize) -> Position {
    Position::new(line, col)
}

#[test]
fn statement_and_expression_pos_helpers() {
    let expr = Expression::IntegerLiteral {
        value: 10,
        raw: "10".to_string(),
        pos: p(1, 5),
    };
    assert_eq!(expr.pos(), p(1, 5));

    let block = BlockStatement::new(vec![], p(2, 1));
    assert_eq!(block.pos(), p(2, 1));

    let stmt = Statement::Let {
        name: Identifier::new("x", p(1, 1)),
        value: expr.clone(),
        pos: p(1, 1),
    };
    assert_eq!(stmt.pos(), p(1, 1));

    let while_stmt = Statement::While {
        condition: Expression::BooleanLiteral {
            value: true,
            pos: p(3, 8),
        },
        body: BlockStatement::new(vec![Statement::Break { pos: p(4, 3) }], p(3, 14)),
        pos: p(3, 1),
    };
    assert_eq!(while_stmt.pos(), p(3, 1));
}

#[test]
fn ast_construction_stores_fields() {
    let id = Identifier::new("value", p(5, 7));
    assert_eq!(id.value, "value");
    assert_eq!(id.pos, p(5, 7));

    let stmt = Statement::Expression {
        expression: Expression::Identifier {
            value: "value".to_string(),
            pos: p(5, 7),
        },
        pos: p(5, 7),
    };

    let block = BlockStatement::new(vec![stmt.clone()], p(5, 5));
    assert_eq!(block.statements, vec![stmt.clone()]);
    assert_eq!(block.pos, p(5, 5));

    let program = Program::new(vec![stmt]);
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn deterministic_statement_and_expression_formatting() {
    let let_stmt = Statement::Let {
        name: Identifier::new("x", p(1, 1)),
        value: Expression::Infix {
            left: Box::new(Expression::Identifier {
                value: "a".to_string(),
                pos: p(1, 9),
            }),
            operator: "+".to_string(),
            right: Box::new(Expression::Identifier {
                value: "b".to_string(),
                pos: p(1, 13),
            }),
            pos: p(1, 11),
        },
        pos: p(1, 1),
    };
    assert_eq!(let_stmt.to_string(), "let x = (a + b);");

    let return_stmt = Statement::Return {
        value: Expression::Prefix {
            operator: "-".to_string(),
            right: Box::new(Expression::IntegerLiteral {
                value: 5,
                raw: "5".to_string(),
                pos: p(2, 10),
            }),
            pos: p(2, 9),
        },
        pos: p(2, 1),
    };
    assert_eq!(return_stmt.to_string(), "return (-5);");

    let expr_stmt = Statement::Expression {
        expression: Expression::Call {
            function: Box::new(Expression::Identifier {
                value: "add".to_string(),
                pos: p(3, 1),
            }),
            arguments: vec![
                Expression::IntegerLiteral {
                    value: 1,
                    raw: "1".to_string(),
                    pos: p(3, 5),
                },
                Expression::IntegerLiteral {
                    value: 2,
                    raw: "2".to_string(),
                    pos: p(3, 8),
                },
            ],
            pos: p(3, 1),
        },
        pos: p(3, 1),
    };
    assert_eq!(expr_stmt.to_string(), "add(1, 2);");
}

#[test]
fn deterministic_if_function_array_hash_index_and_loop_formatting() {
    let if_expr = Expression::If {
        condition: Box::new(Expression::Infix {
            left: Box::new(Expression::Identifier {
                value: "x".to_string(),
                pos: p(1, 5),
            }),
            operator: "<".to_string(),
            right: Box::new(Expression::Identifier {
                value: "y".to_string(),
                pos: p(1, 9),
            }),
            pos: p(1, 7),
        }),
        consequence: BlockStatement::new(
            vec![Statement::Expression {
                expression: Expression::Identifier {
                    value: "x".to_string(),
                    pos: p(1, 13),
                },
                pos: p(1, 13),
            }],
            p(1, 11),
        ),
        alternative: Some(BlockStatement::new(
            vec![Statement::Expression {
                expression: Expression::Identifier {
                    value: "y".to_string(),
                    pos: p(1, 24),
                },
                pos: p(1, 24),
            }],
            p(1, 22),
        )),
        pos: p(1, 1),
    };
    assert_eq!(if_expr.to_string(), "if ((x < y)) { x; } else { y; }");

    let fn_expr = Expression::FunctionLiteral {
        parameters: vec![Identifier::new("x", p(2, 4)), Identifier::new("y", p(2, 7))],
        body: BlockStatement::new(
            vec![Statement::Expression {
                expression: Expression::Infix {
                    left: Box::new(Expression::Identifier {
                        value: "x".to_string(),
                        pos: p(2, 13),
                    }),
                    operator: "+".to_string(),
                    right: Box::new(Expression::Identifier {
                        value: "y".to_string(),
                        pos: p(2, 17),
                    }),
                    pos: p(2, 15),
                },
                pos: p(2, 13),
            }],
            p(2, 10),
        ),
        pos: p(2, 1),
    };
    assert_eq!(fn_expr.to_string(), "fn(x, y) { (x + y); }");

    let array_expr = Expression::ArrayLiteral {
        elements: vec![
            Expression::IntegerLiteral {
                value: 1,
                raw: "1".to_string(),
                pos: p(3, 2),
            },
            Expression::IntegerLiteral {
                value: 2,
                raw: "2".to_string(),
                pos: p(3, 5),
            },
        ],
        pos: p(3, 1),
    };
    assert_eq!(array_expr.to_string(), "[1, 2]");

    let hash_expr = Expression::HashLiteral {
        pairs: vec![
            (
                Expression::StringLiteral {
                    value: "a".to_string(),
                    pos: p(4, 2),
                },
                Expression::IntegerLiteral {
                    value: 1,
                    raw: "1".to_string(),
                    pos: p(4, 7),
                },
            ),
            (
                Expression::StringLiteral {
                    value: "b".to_string(),
                    pos: p(4, 10),
                },
                Expression::IntegerLiteral {
                    value: 2,
                    raw: "2".to_string(),
                    pos: p(4, 15),
                },
            ),
        ],
        pos: p(4, 1),
    };
    assert_eq!(hash_expr.to_string(), "{\"a\": 1, \"b\": 2}");

    let index_expr = Expression::Index {
        left: Box::new(Expression::Identifier {
            value: "arr".to_string(),
            pos: p(5, 1),
        }),
        index: Box::new(Expression::IntegerLiteral {
            value: 1,
            raw: "1".to_string(),
            pos: p(5, 5),
        }),
        pos: p(5, 1),
    };
    assert_eq!(index_expr.to_string(), "(arr[1])");

    let while_stmt = Statement::While {
        condition: Expression::BooleanLiteral {
            value: true,
            pos: p(6, 8),
        },
        body: BlockStatement::new(
            vec![
                Statement::Continue { pos: p(6, 14) },
                Statement::Break { pos: p(6, 24) },
            ],
            p(6, 12),
        ),
        pos: p(6, 1),
    };
    assert_eq!(while_stmt.to_string(), "while (true) { continue; break; }");
}

#[test]
fn hash_literal_format_preserves_pair_order() {
    let hash_expr = Expression::HashLiteral {
        pairs: vec![
            (
                Expression::StringLiteral {
                    value: "first".to_string(),
                    pos: p(1, 1),
                },
                Expression::IntegerLiteral {
                    value: 1,
                    raw: "1".to_string(),
                    pos: p(1, 10),
                },
            ),
            (
                Expression::StringLiteral {
                    value: "second".to_string(),
                    pos: p(1, 13),
                },
                Expression::IntegerLiteral {
                    value: 2,
                    raw: "2".to_string(),
                    pos: p(1, 23),
                },
            ),
        ],
        pos: p(1, 1),
    };

    assert_eq!(hash_expr.to_string(), "{\"first\": 1, \"second\": 2}");
}

#[test]
fn program_formatting_is_deterministic() {
    let program = Program::new(vec![
        Statement::Let {
            name: Identifier::new("x", p(1, 5)),
            value: Expression::IntegerLiteral {
                value: 1,
                raw: "1".to_string(),
                pos: p(1, 9),
            },
            pos: p(1, 1),
        },
        Statement::Return {
            value: Expression::Identifier {
                value: "x".to_string(),
                pos: p(2, 8),
            },
            pos: p(2, 1),
        },
    ]);

    assert_eq!(program.to_string(), "let x = 1;\nreturn x;");
}

#[test]
fn pretty_wrapper_matches_program_display() {
    let program = Program::new(vec![Statement::Break { pos: p(1, 1) }]);
    assert_eq!(pretty::format_ast(&program), program.to_string());
}

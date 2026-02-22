use monkey_rust_compiler::ast::{Expression, Program, Statement};
use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::parser::Parser;
use monkey_rust_compiler::position::Position;

fn parse(input: &str) -> (Program, Vec<String>) {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    let errors = parser.errors().iter().map(ToString::to_string).collect();
    (program, errors)
}

fn assert_no_errors(input: &str, errors: &[String]) {
    assert!(
        errors.is_empty(),
        "expected no parser errors for input:\n{input}\nerrors:\n{}",
        errors.join("\n")
    );
}

fn parse_single_expression(input: &str) -> Expression {
    let (program, errors) = parse(input);
    assert_no_errors(input, &errors);
    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Statement::Expression { expression, .. } => expression.clone(),
        other => panic!("expected expression statement, got {other:?}"),
    }
}

#[test]
fn parses_statement_forms_and_positions() {
    let (let_program, let_errors) = parse("let x = 5;");
    assert_no_errors("let x = 5;", &let_errors);
    assert_eq!(let_program.statements.len(), 1);
    match &let_program.statements[0] {
        Statement::Let { name, value, pos } => {
            assert_eq!(name.value, "x");
            assert_eq!(name.pos, Position::new(1, 5));
            assert_eq!(*pos, Position::new(1, 1));
            match value {
                Expression::IntegerLiteral { value, raw, pos } => {
                    assert_eq!(*value, 5);
                    assert_eq!(raw, "5");
                    assert_eq!(*pos, Position::new(1, 9));
                }
                other => panic!("expected integer literal, got {other:?}"),
            }
        }
        other => panic!("expected let statement, got {other:?}"),
    }

    let (return_program, return_errors) = parse("return 5;");
    assert_no_errors("return 5;", &return_errors);
    match &return_program.statements[0] {
        Statement::Return { value, pos } => {
            assert_eq!(*pos, Position::new(1, 1));
            assert!(matches!(value, Expression::IntegerLiteral { value: 5, .. }));
        }
        other => panic!("expected return statement, got {other:?}"),
    }

    let (while_program, while_errors) = parse("while (x < 10) { let x = x + 1; }");
    assert_no_errors("while (x < 10) { let x = x + 1; }", &while_errors);
    match &while_program.statements[0] {
        Statement::While {
            condition,
            body,
            pos,
        } => {
            assert_eq!(*pos, Position::new(1, 1));
            assert!(matches!(
                condition,
                Expression::Infix { operator, .. } if operator == "<"
            ));
            assert_eq!(body.pos, Position::new(1, 16));
            assert_eq!(body.statements.len(), 1);
            assert!(matches!(body.statements[0], Statement::Let { .. }));
        }
        other => panic!("expected while statement, got {other:?}"),
    }

    let (ctrl_program, ctrl_errors) = parse("break; continue;");
    assert_no_errors("break; continue;", &ctrl_errors);
    assert!(matches!(
        ctrl_program.statements[0],
        Statement::Break { .. }
    ));
    assert!(matches!(
        ctrl_program.statements[1],
        Statement::Continue { .. }
    ));

    let (expr_program, expr_errors) = parse("x + 1\ny + 2;");
    assert_no_errors("x + 1\\ny + 2;", &expr_errors);
    assert_eq!(expr_program.statements.len(), 2);
    assert!(matches!(
        expr_program.statements[0],
        Statement::Expression { .. }
    ));
    assert!(matches!(
        expr_program.statements[1],
        Statement::Expression { .. }
    ));
}

#[test]
fn parses_nested_while_and_if_blocks() {
    let (program, errors) = parse("while (x < 10) { if (x < 5) { x; } let x = x + 1; }");
    assert_no_errors("nested while/if", &errors);
    assert_eq!(program.statements.len(), 1);

    let outer = match &program.statements[0] {
        Statement::While { body, .. } => body,
        other => panic!("expected outer while, got {other:?}"),
    };
    assert_eq!(outer.statements.len(), 2);
    assert!(matches!(outer.statements[0], Statement::Expression { .. }));
    assert!(matches!(outer.statements[1], Statement::Let { .. }));
}

#[test]
fn parses_nested_while_statements() {
    let (program, errors) = parse("while (x < 10) { while (y < 5) { y; } x; }");
    assert_no_errors("nested while", &errors);
    assert_eq!(program.statements.len(), 1);

    let outer = match &program.statements[0] {
        Statement::While { body, .. } => body,
        other => panic!("expected outer while, got {other:?}"),
    };
    assert_eq!(outer.statements.len(), 2);
    match &outer.statements[0] {
        Statement::While { condition, .. } => {
            assert!(matches!(
                condition,
                Expression::Infix { operator, .. } if operator == "<"
            ));
        }
        other => panic!("expected inner while, got {other:?}"),
    }
}

#[test]
fn parses_break_and_continue_with_optional_semicolons() {
    let (program, errors) = parse("while (true) { break; continue; break continue }");
    assert_no_errors("while (true) { break; continue; break continue }", &errors);
    assert_eq!(program.statements.len(), 1);
    let body = match &program.statements[0] {
        Statement::While { body, .. } => body,
        other => panic!("expected while statement, got {other:?}"),
    };
    assert_eq!(body.statements.len(), 4);
    assert!(matches!(body.statements[0], Statement::Break { .. }));
    assert!(matches!(body.statements[1], Statement::Continue { .. }));
    assert!(matches!(body.statements[2], Statement::Break { .. }));
    assert!(matches!(body.statements[3], Statement::Continue { .. }));
}

#[test]
fn parser_ignores_hash_comments_via_lexer_tokens() {
    let input = "let x = 10; # keep x\n# entire line ignored\nreturn x;\n";
    let (program, errors) = parse(input);
    assert_no_errors(input, &errors);
    assert_eq!(program.statements.len(), 2);
    assert!(matches!(program.statements[0], Statement::Let { .. }));
    assert!(matches!(program.statements[1], Statement::Return { .. }));
}

#[test]
fn parser_ignores_empty_semicolons_top_level_and_in_blocks() {
    let (program, errors) = parse(";");
    assert_no_errors(";", &errors);
    assert_eq!(program.statements.len(), 0);

    let (program, errors) = parse(";;;");
    assert_no_errors(";;;", &errors);
    assert_eq!(program.statements.len(), 0);

    let (program, errors) = parse("; let x = 5; ; x; ;");
    assert_no_errors("; let x = 5; ; x; ;", &errors);
    assert_eq!(program.statements.len(), 2);
    assert!(matches!(program.statements[0], Statement::Let { .. }));
    assert!(matches!(
        program.statements[1],
        Statement::Expression { .. }
    ));

    let (program, errors) = parse("if (true) { ; ; 10; ; }");
    assert_no_errors("if (true) { ; ; 10; ; }", &errors);
    assert_eq!(program.statements.len(), 1);
    let if_expr = match &program.statements[0] {
        Statement::Expression { expression, .. } => expression,
        other => panic!("expected expression statement, got {other:?}"),
    };
    let consequence = match if_expr {
        Expression::If { consequence, .. } => consequence,
        other => panic!("expected if expression, got {other:?}"),
    };
    assert_eq!(consequence.statements.len(), 1);
}

#[test]
fn parses_prefix_expressions() {
    let cases = [
        ("!5", "(!5);"),
        ("-15", "(-15);"),
        ("!true", "(!true);"),
        ("-(1 + 2)", "(-(1 + 2));"),
    ];

    for (input, expected) in cases {
        let (program, errors) = parse(input);
        assert_no_errors(input, &errors);
        assert_eq!(program.to_string(), expected, "input={input}");
    }
}

#[test]
fn parses_infix_expressions_including_logical() {
    let cases = [
        ("5 + 5", "(5 + 5);"),
        ("5 - 5", "(5 - 5);"),
        ("5 * 5", "(5 * 5);"),
        ("5 / 5", "(5 / 5);"),
        ("5 > 5", "(5 > 5);"),
        ("5 < 5", "(5 < 5);"),
        ("5 == 5", "(5 == 5);"),
        ("5 != 5", "(5 != 5);"),
        ("5 <= 5", "(5 <= 5);"),
        ("5 >= 5", "(5 >= 5);"),
        ("true && false", "(true && false);"),
        ("true || false", "(true || false);"),
    ];

    for (input, expected) in cases {
        let (program, errors) = parse(input);
        assert_no_errors(input, &errors);
        assert_eq!(program.to_string(), expected, "input={input}");
    }
}

#[test]
fn respects_operator_precedence() {
    let cases = [
        ("-a * b", "((-a) * b);"),
        ("!-a", "(!(-a));"),
        ("a + b + c", "((a + b) + c);"),
        ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f);"),
        ("3 + 4; -5 * 5", "(3 + 4);\n((-5) * 5);"),
        ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4));"),
        ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4));"),
        ("true || false && true", "(true || (false && true));"),
        ("a + add(b * c) + d", "((a + add((b * c))) + d);"),
        ("a[1 + 1] * b", "((a[(1 + 1)]) * b);"),
        ("add(a, b)[0]", "(add(a, b)[0]);"),
    ];

    for (input, expected) in cases {
        let (program, errors) = parse(input);
        assert_no_errors(input, &errors);
        assert_eq!(program.to_string(), expected, "input={input}");
    }
}

#[test]
fn parses_if_if_else_and_else_if() {
    let (if_program, if_errors) = parse("if (x < y) { x }");
    assert_no_errors("if (x < y) { x }", &if_errors);
    assert_eq!(if_program.to_string(), "if ((x < y)) { x; };");

    let (if_else_program, if_else_errors) = parse("if (x < y) { x } else { y }");
    assert_no_errors("if/else", &if_else_errors);
    assert_eq!(
        if_else_program.to_string(),
        "if ((x < y)) { x; } else { y; };"
    );

    let expr = parse_single_expression("if (x < y) { x } else if (y < z) { y }");
    match expr {
        Expression::If { alternative, .. } => {
            let alt = alternative.expect("expected else alternative block");
            assert_eq!(alt.statements.len(), 1);
            match &alt.statements[0] {
                Statement::Expression { expression, .. } => {
                    assert!(matches!(expression, Expression::If { .. }));
                }
                other => panic!("expected nested if expression statement, got {other:?}"),
            }
        }
        other => panic!("expected if expression, got {other:?}"),
    }
}

#[test]
fn parses_function_literals_and_calls() {
    let (empty_fn_program, empty_fn_errors) = parse("fn() {}");
    assert_no_errors("fn() {}", &empty_fn_errors);
    assert_eq!(empty_fn_program.to_string(), "fn() {};");

    let (fn_program, fn_errors) = parse("fn(x, y) { x + y; }");
    assert_no_errors("fn(x, y) { x + y; }", &fn_errors);
    assert_eq!(fn_program.to_string(), "fn(x, y) { (x + y); };");

    let (call_program, call_errors) = parse("add(1, 2 * 3, 4 + 5)");
    assert_no_errors("add(1, 2 * 3, 4 + 5)", &call_errors);
    assert_eq!(call_program.to_string(), "add(1, (2 * 3), (4 + 5));");
}

#[test]
fn parses_function_parameter_variants() {
    let cases = [
        ("fn() {};", Vec::<&str>::new()),
        ("fn(x) {};", vec!["x"]),
        ("fn(x, y, z) {};", vec!["x", "y", "z"]),
    ];

    for (input, expected_params) in cases {
        let expr = parse_single_expression(input);
        match expr {
            Expression::FunctionLiteral { parameters, .. } => {
                let got = parameters
                    .iter()
                    .map(|p| p.value.as_str())
                    .collect::<Vec<_>>();
                assert_eq!(got, expected_params, "input={input}");
            }
            other => panic!("expected function literal, got {other:?}"),
        }
    }
}

#[test]
fn parses_call_parameter_variants() {
    let cases = [
        ("func()", Vec::<&str>::new()),
        ("func(3 - 1)", vec!["(3 - 1)"]),
        ("func(1, 2 * 3 + 1, 98)", vec!["1", "((2 * 3) + 1)", "98"]),
    ];

    for (input, expected_args) in cases {
        let expr = parse_single_expression(input);
        match expr {
            Expression::Call { arguments, .. } => {
                let got = arguments
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                assert_eq!(got, expected_args, "input={input}");
            }
            other => panic!("expected call expression, got {other:?}"),
        }
    }
}

#[test]
fn parses_arrays_hashes_and_index_expressions() {
    let (arr_empty, err_empty) = parse("[]");
    assert_no_errors("[]", &err_empty);
    assert_eq!(arr_empty.to_string(), "[];");

    let (arr_expr, arr_err) = parse("[1, 2 * 2, 3 + 3]");
    assert_no_errors("[1, 2 * 2, 3 + 3]", &arr_err);
    assert_eq!(arr_expr.to_string(), "[1, (2 * 2), (3 + 3)];");

    let (hash_empty, hash_empty_err) = parse("{}");
    assert_no_errors("{}", &hash_empty_err);
    assert_eq!(hash_empty.to_string(), "{};");

    let (hash_program, hash_errors) = parse("{\"one\": 1, \"two\": 2}");
    assert_no_errors("hash", &hash_errors);
    assert_eq!(hash_program.to_string(), "{\"one\": 1, \"two\": 2};");

    let expr = parse_single_expression("{\"one\": 1, \"two\": 2}");
    match expr {
        Expression::HashLiteral { pairs, .. } => {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].0.to_string(), "\"one\"");
            assert_eq!(pairs[1].0.to_string(), "\"two\"");
        }
        other => panic!("expected hash literal, got {other:?}"),
    }

    let (idx_program, idx_errors) = parse("myArray[1 + 1]");
    assert_no_errors("myArray[1 + 1]", &idx_errors);
    assert_eq!(idx_program.to_string(), "(myArray[(1 + 1)]);");
}

#[test]
fn parses_hash_literals_with_various_key_types_and_expression_pairs() {
    let cases = [
        ("{\"one\": 1, \"two\": 2, \"three\": 3}", 3usize),
        ("{}", 0usize),
        ("{1: 100}", 1usize),
        ("{true: \"assa\", false: \"false\"}", 2usize),
    ];

    for (input, expected_len) in cases {
        let expr = parse_single_expression(input);
        match expr {
            Expression::HashLiteral { pairs, .. } => assert_eq!(pairs.len(), expected_len),
            other => panic!("expected hash literal, got {other:?}"),
        }
    }

    let expr = parse_single_expression("{5 - 2: 0 + 1, 10 - 8: 15 / 5}");
    match expr {
        Expression::HashLiteral { pairs, .. } => {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].0.to_string(), "(5 - 2)");
            assert_eq!(pairs[0].1.to_string(), "(0 + 1)");
            assert_eq!(pairs[1].0.to_string(), "(10 - 8)");
            assert_eq!(pairs[1].1.to_string(), "(15 / 5)");
        }
        other => panic!("expected hash literal, got {other:?}"),
    }
}

#[test]
fn accumulates_parse_errors_without_panicking() {
    let cases = ["let = 5;", "if (x < ) { x }", "fn(,x) {}", "x = x + 1;"];

    for input in cases {
        let (program, errors) = parse(input);
        assert!(
            !errors.is_empty(),
            "expected parse errors for input {input}, got program: {}",
            program
        );

        for err in &errors {
            assert!(
                err.contains(':'),
                "error should include position, got: {err}"
            );
        }
    }
}

#[test]
fn reports_no_prefix_parse_error_for_unexpected_rparen() {
    let (_program, errors) = parse(")");
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].contains("no prefix parse function for RParen"),
        "unexpected error: {}",
        errors[0]
    );
}

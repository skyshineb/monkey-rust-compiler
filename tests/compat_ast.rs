mod common;

use common::{assert_or_update_golden, fixture_cases, golden_for, read_text, render_ast};

#[test]
fn compat_ast_golden() {
    for fixture in fixture_cases("tests/fixtures/ast", "monkey") {
        let source = read_text(&fixture);
        let actual = render_ast(&source);
        let golden = golden_for(&fixture, "ast");
        assert_or_update_golden(&actual, &golden);
    }
}

mod common;

use common::{assert_or_update_golden, fixture_cases, golden_for, read_text, render_tokens};

#[test]
fn compat_tokens_golden() {
    for fixture in fixture_cases("tests/fixtures/tokens", "monkey") {
        let source = read_text(&fixture);
        let actual = render_tokens(&source);
        let golden = golden_for(&fixture, "tokens");
        assert_or_update_golden(&actual, &golden);
    }
}

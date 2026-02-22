mod common;

use common::{
    assert_or_update_golden, fixture_cases, golden_for, read_text, render_repl_transcript,
};

#[test]
fn compat_repl_golden() {
    for fixture in fixture_cases("tests/fixtures/repl", "repl") {
        let transcript = read_text(&fixture);
        let actual = render_repl_transcript(&transcript);
        let golden = golden_for(&fixture, "repl");
        assert_or_update_golden(&actual, &golden);
    }
}

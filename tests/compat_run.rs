mod common;

use common::{assert_or_update_golden, fixture_cases, golden_for, read_text, render_run};

#[test]
fn compat_run_golden() {
    for fixture in fixture_cases("tests/fixtures/run", "monkey") {
        let source = read_text(&fixture);
        let actual = render_run(&source);
        let golden = golden_for(&fixture, "run");
        assert_or_update_golden(&actual, &golden);
    }
}

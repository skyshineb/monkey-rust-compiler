# Compatibility Golden Tests

Fixture-driven compatibility tests live under `tests/compat_*.rs` and use files in `tests/fixtures/`.

## Fixture layout

- `tests/fixtures/tokens/*.monkey` -> `*.tokens.golden`
- `tests/fixtures/ast/*.monkey` -> `*.ast.golden`
- `tests/fixtures/run/*.monkey` -> `*.run.golden`
- `tests/fixtures/repl/*.repl` -> `*.repl.golden`

Java parity fixtures (Step 21):

- `tests/fixtures/conformance/run/*.monkey`
- `tests/fixtures/conformance/tokens/*.monkey`
- `tests/fixtures/conformance/ast/*.monkey`

## Running tests

- Full suite: `cargo test`
- Compatibility-only: `cargo test compat_`
- Conformance parity: `make conformance`
- Local quality gate: `make check`
- Release gate: `make release-check`

## Updating goldens

Goldens are never updated during normal test runs.

To regenerate intentionally:

```bash
UPDATE_GOLDENS=1 cargo test compat_
# or
make goldens-update
```

Then re-run without `UPDATE_GOLDENS` to verify snapshots are stable.

## Rules

- Normalize line endings only (`\r\n` -> `\n`) and enforce one trailing newline.
- Do not change expected output without checking `COMPATIBILITY.md` and `PROTOCOL.md` first.
- Use conformance parity checks to cross-validate Java↔Rust behavior when Java reference is configured.

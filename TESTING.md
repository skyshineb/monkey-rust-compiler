# Compatibility Golden Tests

Fixture-driven compatibility tests live under `tests/compat_*.rs` and use files in `tests/fixtures/`.

## Fixture layout

- `tests/fixtures/tokens/*.monkey` -> `*.tokens.golden`
- `tests/fixtures/ast/*.monkey` -> `*.ast.golden`
- `tests/fixtures/run/*.monkey` -> `*.run.golden`
- `tests/fixtures/repl/*.repl` -> `*.repl.golden`

## Running tests

- Full suite: `cargo test`
- Compatibility-only: `cargo test compat_`

## Updating goldens

Goldens are never updated during normal test runs.

To regenerate intentionally:

```bash
UPDATE_GOLDENS=1 cargo test compat_
```

Then re-run without `UPDATE_GOLDENS` to verify the new snapshots are stable.

## Rules

- Normalize line endings only (`\r\n` -> `\n`) and enforce one trailing newline.
- Do not change expected output without checking `COMPATIBILITY.md` and `PROTOCOL.md` first.

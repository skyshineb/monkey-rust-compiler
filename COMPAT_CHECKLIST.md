# Compatibility Checklist

Run these before and after protocol-visible changes:

```bash
cargo fmt --all --check
cargo test
cargo test compat_
```

## Golden workflow

- Goldens live under `tests/fixtures/**`.
- Normal test runs must not mutate goldens.
- Regenerate only with:

```bash
UPDATE_GOLDENS=1 cargo test compat_
```

## Critical invariants

- Builtin order is stable: `len`, `first`, `last`, `rest`, `push`, `puts`.
- `&&` / `||` are short-circuit and return booleans.
- Top-level `break`/`continue` produce `INVALID_CONTROL_FLOW` runtime errors.
- `--tokens` includes positions and EOF token line.
- `--ast` output remains deterministic.
- Runtime errors include position and deterministic stack trace root frame `at <repl>(0 args) @ 1:1`.
- REPL remains stateful and supports `:help`, `:tokens`, `:ast`, `:env`, `:quit`, `:exit`.

If expected output changes, confirm it is required by `COMPATIBILITY.md`/`PROTOCOL.md` before updating goldens.

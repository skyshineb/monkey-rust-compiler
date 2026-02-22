# Java↔Rust Conformance Harness

Step 21 adds a parity layer in addition to Rust goldens:

- Step 19: deterministic Rust output snapshots (`tests/fixtures/*/*.golden`)
- Step 21: Java↔Rust output parity (`tests/fixtures/conformance/*`)

## Environment variables

- `MONKEY_JAVA_REF_CMD` (required for Java parity)
  - Example: `java -jar ./java-ref/monkey.jar`
- `MONKEY_RUST_BIN` (optional)
  - Override Rust binary path for parity tests.
- `MONKEY_JAVA_REF_HAS_TOKENS` (optional, default `1`)
  - Set `0` to skip tokens parity mode.
- `MONKEY_JAVA_REF_HAS_AST` (optional, default `1`)
  - Set `0` to skip AST parity mode.

## Running conformance checks

```bash
make conformance
```

Or directly:

```bash
MONKEY_JAVA_REF_CMD="java -jar path/to/ref.jar" cargo test conformance_ -- --nocapture
```

Without `MONKEY_JAVA_REF_CMD`, parity tests skip with a deterministic message.

## Fixture layout

- `tests/fixtures/conformance/run/*.monkey`
- `tests/fixtures/conformance/tokens/*.monkey`
- `tests/fixtures/conformance/ast/*.monkey`

## Normalization rules

Before comparing Java and Rust outputs, harness normalizes:

1. line endings (`\r\n` -> `\n`)
2. per-line trailing whitespace
3. final newline to exactly one trailing `\n`
4. conservative stacktrace path normalization if host paths leak into output

Normalization does not remove semantic content (values, error kinds/messages, positions, stack frames).

## Mismatch triage

When Java and Rust disagree:

1. check `COMPATIBILITY.md` and `PROTOCOL.md`
2. determine whether mismatch is Java behavior difference, Rust bug, or doc ambiguity
3. if Rust bug: apply minimal fix, then intentionally update affected Rust goldens (Step 19) if needed

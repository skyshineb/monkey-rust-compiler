# AGENTS.md

## Project goal
Implement a Monkey language compiler + VM in Rust that is compatible with the behavior defined in:
- `PROTOCOL.md` (source spec)
- `COMPATIBILITY.md` (engineering compatibility contract)

Compatibility with the Java implementation is more important than "improvements" or refactors.

---

## Mandatory workflow for every task
Before writing code:
1. Read `COMPATIBILITY.md`.
2. Identify the relevant compatibility rules for the task.
3. Implement the feature in Rust.
4. Add or update tests that verify those rules.
5. Run tests and report results.

Do not mark a task complete unless compatibility is verified by tests.

---

## Compatibility rules
- Do not silently change semantics, error types, error formatting, or CLI output.
- If behavior differs from `COMPATIBILITY.md`, treat it as a bug unless explicitly instructed otherwise.
- If a behavior is unclear, inspect `PROTOCOL.md` and existing tests. If still unclear, document the ambiguity and choose the safest compatible behavior.

---

## Required response format for task completion
Every task response MUST include a section:

### Compatibility Verification
- Relevant rules:
  - ...
- Tests added/updated:
  - ...
- Validation run:
  - `cargo test ...`
- Result:
  - pass/fail
- Ambiguities / risks:
  - ...

---

## Testing policy
- Prefer normal unit/integration tests for semantics.
- Every new feature must include at least one compatibility-focused test.

---

## Rust engineering conventions
- Use stable Rust.
- Prefer simple, explicit code over abstraction-heavy design.
- Preserve source positions (line/col) through lexer -> parser -> compiler -> VM for error reporting.
- Keep modules small and focused.
- Avoid introducing dependencies unless they clearly reduce risk or complexity.

---

## Suggested module layout
- `token.rs`
- `lexer.rs`
- `ast.rs`
- `parser.rs`
- `object.rs`
- `bytecode.rs`
- `compiler.rs`
- `symbol_table.rs`
- `vm.rs`
- `builtins.rs`
- `repl.rs`
- `main.rs`

(Exact layout may evolve, but keep concerns separated.)

---

## Commands to run before finishing a task
- `cargo fmt`
- `cargo clippy -- -D warnings` (if clippy is enabled)
- `cargo test`

If a command fails, report the failure and why.

---

## Non-goals unless explicitly requested
- New language features
- Syntax changes
- Performance tuning that risks semantic drift
- Output formatting changes not required by compatibility

---

## Priority order
1. Correctness / compatibility
2. Tests
3. Clear error behavior
4. Maintainable code
5. Performance
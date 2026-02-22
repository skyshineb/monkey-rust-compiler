# COMPATIBILITY.md

## Scope and purpose

This document is a **compatibility contract** for implementing a Rust compiler+VM that must match the **current Java implementation behavior**.

- This contract is derived primarily from `PROTOCOL.md`, with spot checks against Java source for deterministic output/error behavior.
- Compatibility-critical rules use **MUST**.
- Guidance that is useful but not required uses **SHOULD**.
- This contract describes behavior, not architecture: the Rust implementation MAY differ internally but MUST preserve externally observable behavior.

---

## Feature matrix (lexer / parser / runtime / builtins / REPL / CLI)

| Area | Contract |
|---|---|
| Lexer | MUST recognize the current token set and keywords exactly (`IDENT`, `INT`, `STRING`, operators including `<= >= && ||`, delimiters, `fn/let/true/false/if/else/return/while/break/continue`). Unknown characters MUST produce `ILLEGAL`. |
| Lexer: trivia | MUST skip whitespace and `#` line comments. |
| Lexer: strings | MUST lex `"..."` raw contents until next `"` or EOF; escape sequences are not interpreted into special characters by the lexer. |
| Lexer: positions | MUST attach line/column to tokens; `--tokens` output MUST include positions. |
| Parser | MUST support: `let`, `return`, `while`, `break`, `continue`, expression statements, conditionals, functions, calls, arrays, hashes, indexing, prefix/infix/grouping. |
| Precedence | MUST preserve precedence: `||` < `&&` < equality (`== !=`) < relational (`< > <= >=`) < additive (`+ -`) < multiplicative (`* /`) < prefix < call < index. |
| Parse errors | MUST accumulate parse errors as strings and skip evaluation when parse errors exist. |
| Runtime values | MUST support `INTEGER`, `BOOLEAN`, `STRING`, `NULL`, `ARRAY`, `HASH`, `FUNCTION`, `BUILTIN`, plus internal `RETURN/BREAK/CONTINUE` behavior. |
| Truthiness | MUST treat only `false` and `null` as falsey; all else truthy. |
| Functions/closures | MUST capture lexical environment (closure semantics), evaluate args left-to-right, and error when calling non-callables. |
| Arrays/hashes | MUST return `null` for missing/out-of-range index lookup; array indices MUST be integers; hash keys MUST be hashable. |
| Builtins | MUST expose exactly: `len`, `first`, `last`, `rest`, `push`, `puts`. Names and behavior MUST match protocol semantics. |
| REPL | MUST be stateful across inputs; MUST support multiline completeness buffering and meta commands `:help`, `:tokens`, `:ast`, `:env`, `:quit`, `:exit`. |
| CLI | MUST support modes: `run`, `bench`, `--tokens`, `--ast`; MUST preserve usage shape and exit codes. |

---

## Behavioral edge cases (important semantic traps)

- **Unary minus on null**
  - MUST preserve: `-null` evaluates to `null` (not an error).
  - Example:
    ```monkey
    -null
    # => null
    ```

- **Logical short-circuiting**
  - `&&` MUST short-circuit when left side is falsey.
  - `||` MUST short-circuit when left side is truthy.
  - Both operators MUST return Monkey booleans (`true`/`false`) based on truthiness, not arbitrary operand values.
  - Examples:
    ```monkey
    false && unknown_identifier   # => false (no UNKNOWN_IDENTIFIER error)
    true  || unknown_identifier   # => true  (no UNKNOWN_IDENTIFIER error)
    ```

- **Loop control validity**
  - `break` and `continue` MUST only be valid inside `while` loops.
  - Outside a loop, MUST raise runtime error type `INVALID_CONTROL_FLOW`.
  - Example:
    ```monkey
    break;
    # => runtime error INVALID_CONTROL_FLOW
    ```

- **Missing key/index behavior**
  - Missing array index (negative or out-of-bounds) MUST evaluate to `null`.
  - Missing hash key MUST evaluate to `null`.
  - Examples:
    ```monkey
    [1, 2][10]      # => null
    [1, 2][-1]      # => null
    {"x": 1}["y"] # => null
    ```

- **String operator restrictions**
  - String `+` MUST concatenate.
  - Other string infix operators MUST raise `UNSUPPORTED_OPERATION`.

- **Conditional fallthrough**
  - `if` without `else` MUST yield `null` when condition is falsey.

- **Identifier lookup order**
  - MUST resolve in order: lexical environment chain → builtin table → `UNKNOWN_IDENTIFIER` error.

---

## Error contract (error types, formatting, positions, stack trace expectations)

### Runtime error types (protocol names)

Compatibility implementation MUST preserve these runtime error type names and semantics:

- `TYPE_MISMATCH`
- `UNKNOWN_IDENTIFIER`
- `NOT_CALLABLE`
- `WRONG_ARGUMENT_COUNT`
- `INVALID_ARGUMENT_TYPE`
- `INVALID_CONTROL_FLOW`
- `INVALID_INDEX`
- `UNHASHABLE`
- `DIVISION_BY_ZERO`
- `UNSUPPORTED_OPERATION`

### Runtime formatting contract

- Single-line format MUST be:
  - `Error[TYPE] at line:col: message`
- Multiline format MUST include:
  - first line = single-line format
  - then `Stack trace:` line
  - then stack frames, ending with root frame:
    - `at <repl>(0 args) @ 1:1`
- Runtime errors MUST carry source position (`line:column`).

### Parse error contract

- Parser MUST accumulate (not throw immediately) parse errors as strings.
- CLI parse failure MUST prefix:
  - `Parse errors in <path>:`
  - followed by `- <error>` lines.

### Stack trace expectations

- Function calls SHOULD record function name + call site + arg count in frames.
- Root `<repl>(0 args) @ 1:1` frame MUST appear in formatted multiline runtime error output.

---

## Output contract (`--tokens`, `--ast`, REPL output)

- `--tokens <path>` MUST print one token per line as:
  - `TYPE('literal') @ line:col`
  - and MUST include EOF token line.
- `--ast <path>` MUST print parser string rendering for the program.
- REPL success path MUST print `inspect()` value of evaluated result.
- REPL parse errors MUST print monkey-face banner and per-error lines.
- REPL runtime errors MUST print formatted multiline runtime error block.
- REPL `:tokens` inline mode MUST print `TOKENS:` heading and indented token lines.
- REPL `:ast` inline mode MUST print `AST:` heading and parsed string (or parse errors if invalid).
- REPL `:env` MUST print `ENV:` and sorted current-scope bindings (`(empty)` when none).

---

## CLI contract (modes, exit codes, deterministic error prefixes)

### Modes

The implementation MUST support exactly:

- `run <path>`
- `bench <path>`
- `--tokens <path>`
- `--ast <path>`

Usage form MUST be:

- `monkey [run <path> | bench <path> | --tokens <path> | --ast <path>]`

### Exit codes

- `0` = success
- `1` = runtime failure / parse failure / file load failure / invalid path
- `2` = CLI usage error (wrong arity or unknown mode)

### Deterministic stderr prefixes

- Parse errors MUST start with: `Parse errors in <path>:`
- Runtime errors MUST start with: `Runtime error in <path>:`
- Bench mode MUST append timing line to stderr:
  - `Execution time: <n.nn> ms` (decimal formatting acceptable as current protocol)

### REPL vs CLI entry behavior

- No args MUST start REPL session.
- With args, command MUST run in CLI mode and return appropriate exit code.

---

## Definition of Done checklist for compatibility

Use this as CI/manual gate:

- [ ] Token/keyword set matches protocol exactly (including `&&`, `||`, `while`, `break`, `continue`).
- [ ] Precedence/order matches protocol exactly.
- [ ] Truthiness rules match (`false` + `null` falsey only).
- [ ] `-null` returns `null`.
- [ ] `&&`/`||` are short-circuit and return booleans.
- [ ] `break`/`continue` outside loops raise `INVALID_CONTROL_FLOW`.
- [ ] Array out-of-range and missing hash key both return `null`.
- [ ] Builtins present with exact names: `len`, `first`, `last`, `rest`, `push`, `puts`.
- [ ] Runtime error types and formatting match contract, including stack trace root frame.
- [ ] `--tokens` includes positions and EOF line.
- [ ] `--ast` output matches parser rendering behavior.
- [ ] REPL meta commands and multiline buffering behavior are compatible.
- [ ] CLI modes, usage text shape, and exit codes are compatible.
- [ ] Parse/runtime CLI error prefixes are deterministic and contract-compliant.

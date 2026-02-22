# Monkey Language Protocol (Current Implementation)

This document describes the **actual behavior implemented in this repository** for the Monkey language runtime, parser, REPL, and CLI.

---

## 1) Language scope and execution model

- Monkey is an interpreted language with a C-like surface syntax.
- Execution flow:
  1. Lex source text into tokens.
  2. Parse tokens into an AST (Pratt parser).
  3. Evaluate AST nodes to runtime objects.
- The main interfaces are:
  - **REPL mode** (default when no CLI args are provided)
  - **Script mode** (`run`, `bench`, `--tokens`, `--ast`)

The evaluator uses lexical environments and supports closures.

---

## 2) Lexical grammar and tokens

### 2.1 Token categories

Implemented tokens include:

- Literals and identifiers:
  - `IDENT`, `INT`, `STRING`
- Assignment and arithmetic:
  - `=`, `+`, `-`, `*`, `/`
- Logical and comparison:
  - `!`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`
- Delimiters:
  - `,`, `;`, `:`, `(`, `)`, `{`, `}`, `[`, `]`
- Keywords:
  - `fn`, `let`, `true`, `false`, `if`, `else`, `return`, `while`, `break`, `continue`

Unknown characters are tokenized as `ILLEGAL`.

### 2.2 Identifiers and numbers

- Identifiers are scanned using Java identifier-start checks repeatedly.
- Integer literals are decimal digit sequences only.
- No float literal support.

### 2.3 Strings

- Strings are delimited by double quotes (`"..."`).
- Lexer reads until the next `"` or EOF.
- Escape sequences are **not interpreted by lexer** (no explicit unescaping phase).

### 2.4 Comments and whitespace

- Whitespace is skipped.
- Line comments begin with `#` and continue to end-of-line.

### 2.5 Source positions

- Tokens carry line/column location metadata.
- Runtime errors report source positions and call stack frames.

---

## 3) Parsing and precedence

### 3.1 Statement forms

Implemented top-level/block statements:

- `let <ident> = <expr>;`
- `return <expr>;`
- `while (<expr>) { <statements> }`
- `break;`
- `continue;`
- Expression statements (`<expr>;`)

Semicolons are optional in several contexts where parser checks allow omission.

### 3.2 Expression forms

Implemented expressions:

- Identifier, integer literal, boolean literal, string literal
- Prefix: `!expr`, `-expr`
- Infix arithmetic/comparison/logical operators
- Grouping: `(expr)`
- Conditional: `if (cond) { ... } else { ... }`
- Function literal: `fn(<params>) { ... }`
- Function call: `fnExpr(arg1, arg2, ...)`
- Array literal: `[e1, e2, ...]`
- Hash literal: `{ key1: value1, key2: value2, ... }`
- Index expression: `container[index]`

### 3.3 Operator precedence (low → high)

1. `||`
2. `&&`
3. `==`, `!=`
4. `<`, `>`, `<=`, `>=`
5. `+`, `-`
6. `*`, `/`
7. Prefix (`!`, unary `-`)
8. Call `()`
9. Index `[]`

### 3.4 Parse error model

- Parser accumulates errors and returns them as a list.
- Example style: expected next token mismatches or missing prefix parse function.
- Evaluation is skipped when parse errors exist.

---

## 4) Runtime object model

Core runtime object variants include:

- `INTEGER`
- `BOOLEAN`
- `STRING`
- `NULL`
- `ARRAY`
- `HASH`
- `FUNCTION` (user-defined)
- `BUILTIN` (native functions)
- Internal control wrappers:
  - `RETURN`
  - `BREAK`
  - `CONTINUE`

Environment stores identifier bindings with lexical parent chaining.

---

## 5) Evaluation semantics

### 5.1 Truthiness

- `false` and `null` are falsey.
- Everything else is truthy.

### 5.2 Arithmetic and comparison

For integer/integer operands:

- `+`, `-`, `*`, `/` (integer division)
- `<`, `>`, `<=`, `>=`, `==`, `!=`
- Division by zero raises runtime error `DIVISION_BY_ZERO`.

### 5.3 String operations

- Supported infix operation: string concatenation via `+`.
- Other string infix operators raise `UNSUPPORTED_OPERATION`.

### 5.4 Boolean and mixed-type comparisons

- `&&` and `||` evaluate by truthiness and return Monkey booleans.
- Both are short-circuiting.
- Unsupported type/operator combinations produce `TYPE_MISMATCH`.

### 5.5 Prefix operators

- `!expr` returns negated truthiness as boolean.
- `-expr` supports integers.
- Unary minus on `null` returns `null`.
- Unary minus on unsupported types raises `TYPE_MISMATCH`.

### 5.6 Conditionals

- `if (condition) { consequence } else { alternative }`
- If condition false and no `else`, result is `null`.

### 5.7 Variables and assignment

- `let` binds names in current environment.
- Identifier lookup:
  1. lexical environment chain
  2. built-in function table
  3. else runtime error `UNKNOWN_IDENTIFIER`

### 5.8 Functions and closures

- Functions are first-class values.
- Functions capture definition-time environment (closures).
- Calls create an enclosed environment for parameters.
- `return` exits function body; at program top-level, final unwrapped value is returned.
- Calling non-function values raises `NOT_CALLABLE`.

### 5.9 Arrays

- Arrays are ordered lists of values.
- Indexing requires integer index.
- Out-of-range index returns `null`.

### 5.10 Hashes

- Hash literals evaluate keys and values before insertion.
- Hash keys must be hashable (`MonkeyHashable` enforced).
- Missing key lookup returns `null`.

### 5.11 Loops and control flow

- `while (condition) { body }` loops while condition is truthy.
- `break` exits nearest loop and loop expression evaluates to `null` at break point.
- `continue` skips to next loop iteration.
- `break`/`continue` outside loop raise `INVALID_CONTROL_FLOW`.

---

## 6) Built-in functions

Implemented built-ins:

1. `len(x)`
   - string → length
   - array → size
   - otherwise type mismatch error
2. `first(arr)`
   - first element, or `null` for empty array
3. `last(arr)`
   - last element, or `null` for empty array
4. `rest(arr)`
   - new array with all but first element, or `null` for empty array
5. `push(arr, value)`
   - returns new array with appended value
6. `puts(args...)`
   - prints each arg’s `inspect()` to stdout and returns `null`

Arity/type checks are enforced and surfaced as runtime errors.

---

## 7) Runtime error protocol

Errors are represented with:

- `type` (enum):
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
- `message`
- `position` (line:column)
- `stackFrames`

Formatting:

- Single-line: `Error[TYPE] at line:col: message`
- Multiline includes a stack trace and root `<repl>(0 args) @ 1:1` frame.

---

## 8) REPL protocol

Default command with no CLI args starts REPL.

### 8.1 Session behavior

- Stateful session: bindings persist between inputs.
- Multiline accumulation: incomplete constructs are buffered until complete.
- Completeness checks track braces `{}`, parentheses `()`, brackets `[]`, and string quote state.

### 8.2 Meta commands

Meta commands are only accepted when multiline buffer is empty:

- `:help` — command help
- `:tokens [input]` — token dump for inline input or next complete input
- `:ast [input]` — AST string for inline input or next complete input
- `:env` — print current scope bindings
- `:quit` / `:exit` — terminate session

Unknown meta command prints guidance.

### 8.3 REPL output behavior

- Parse errors print monkey-face banner and one error per line.
- Runtime errors print formatted multiline runtime error.
- Successful evaluation prints `inspect()` of result.

---

## 9) CLI protocol

Usage contract:

`monkey [run <path> | bench <path> | --tokens <path> | --ast <path>]`

### 9.1 Modes

- `run <path>`: evaluate file and print resulting value
- `bench <path>`: same as run + prints execution time to stderr
- `--tokens <path>`: print token stream with positions
- `--ast <path>`: print AST rendering

### 9.2 Exit codes

- `0`: success
- `1`: runtime, parse, or load failure
- `2`: usage error (bad args/command)

### 9.3 Deterministic error text

- Parse errors: `Parse errors in <path>:` with bullet lines
- Runtime errors: `Runtime error in <path>:` plus formatted runtime block
- File/path errors: concise path-specific message

---

## 10) Known limitations in current implementation

- No floating-point numeric type.
- No module system / imports.
- String escape processing is limited (lexer reads raw contents until `"`).
- Unicode handling is not full-featured (noted in project TODOs).

---

## 11) Minimal compatibility checklist for future changes

A change should preserve this protocol unless intentionally versioned:

- Token set and keyword behavior
- Precedence ordering
- Truthiness and short-circuit rules
- Null-on-missing behavior for array/hash indexing
- Built-in function names, arity, and return conventions
- Runtime error typing and multiline formatting
- REPL meta-command semantics and statefulness
- CLI mode names, exit codes, and message structure

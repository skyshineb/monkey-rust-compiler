# monkey-rust-compiler

A compatibility-focused Monkey compiler + VM implementation in Rust.

Behavior is locked against:
- `COMPATIBILITY.md`
- `PROTOCOL.md`

## Build

```bash
cargo build
```

## CLI usage

```text
Usage: monkey [run <path> | bench <path> | --tokens <path> | --ast <path>]
```

Examples:

```bash
cargo run -- run examples/hello.monkey
cargo run -- --tokens examples/control_flow.monkey
cargo run -- --ast examples/closures.monkey
cargo run -- bench bench/b1.monkey
```

## REPL

Start REPL:

```bash
cargo run --
# or
cargo run -- repl
```

REPL meta commands:
- `:help`
- `:tokens [input]`
- `:ast [input]`
- `:env`
- `:quit`
- `:exit`

The REPL session is stateful across inputs.

## Tests and quality gates

```bash
cargo test
make check
make compat
```

Golden updates are explicit only:

```bash
make goldens-update
# or
UPDATE_GOLDENS=1 cargo test compat_
```

## Release readiness

```bash
make release-check
```

## Benchmarks

```bash
make bench
# or pass a compiled monkey binary path:
./scripts/bench.sh ./target/debug/monkey
```

## Repository layout

- `src/` runtime/compiler/parser/CLI implementation
- `tests/` unit, integration, and compatibility golden suites
- `examples/` runnable Monkey programs
- `bench/` benchmark programs
- `scripts/` developer and CI helper scripts

## Compatibility note

Do not change protocol-visible behavior without validating against `COMPATIBILITY.md`, `PROTOCOL.md`, and compatibility goldens.

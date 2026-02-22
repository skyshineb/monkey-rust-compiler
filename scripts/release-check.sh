#!/usr/bin/env bash
set -euo pipefail

echo "==> release check: quality gates"
./scripts/ci-check.sh

echo "==> release check: CLI smoke"
cargo run -- run examples/hello.monkey >/dev/null
cargo run -- --tokens examples/control_flow.monkey >/dev/null
cargo run -- --ast examples/closures.monkey >/dev/null

echo "==> release check complete"

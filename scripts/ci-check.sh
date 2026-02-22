#!/usr/bin/env bash
set -euo pipefail

echo "==> fmt check"
cargo fmt --all --check

echo "==> tests"
cargo test

echo "==> compatibility tests"
cargo test compat_

echo "==> conformance parity checks"
./scripts/conformance-check.sh

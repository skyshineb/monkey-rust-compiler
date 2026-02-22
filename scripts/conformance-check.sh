#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${MONKEY_JAVA_REF_CMD:-}" ]]; then
  echo "conformance: MONKEY_JAVA_REF_CMD is not set; skipping Java parity checks"
  exit 0
fi

echo "conformance: running Java↔Rust parity tests"
cargo test conformance_ -- --nocapture

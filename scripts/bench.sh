#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_PATH="${1:-}"
BENCH_FILES=("$ROOT_DIR/bench/b1.monkey" "$ROOT_DIR/bench/b2.monkey" "$ROOT_DIR/bench/b3.monkey")

run_once() {
  local file="$1"
  if [[ -n "$BIN_PATH" ]]; then
    "$BIN_PATH" bench "$file"
  else
    cargo run --quiet -- bench "$file"
  fi
}

echo "Running Monkey benchmarks (3 rounds each)"
for file in "${BENCH_FILES[@]}"; do
  echo "--- $(basename "$file") ---"
  for round in 1 2 3; do
    echo "round $round"
    run_once "$file"
  done
  echo
 done

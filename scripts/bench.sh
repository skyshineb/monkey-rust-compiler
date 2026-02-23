#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_PATH="${1:-${MONKEY_RUST_BIN:-}}"
ROUNDS="${BENCH_ROUNDS:-3}"
BENCH_FILTER="${BENCH_FILTER:-}"
BENCH_FILES=(
  "$ROOT_DIR/bench/b1.monkey"
  "$ROOT_DIR/bench/b2.monkey"
  "$ROOT_DIR/bench/b3.monkey"
  "$ROOT_DIR/bench/b4.monkey"
  "$ROOT_DIR/bench/b5.monkey"
)

run_once() {
  local file="$1"
  if [[ -n "$BIN_PATH" ]]; then
    "$BIN_PATH" bench "$file"
  else
    cargo run --quiet --release -- bench "$file"
  fi
}

echo "Running Monkey benchmarks (${ROUNDS} rounds each, release profile)"
for file in "${BENCH_FILES[@]}"; do
  if [[ -n "$BENCH_FILTER" && "$(basename "$file")" != *"$BENCH_FILTER"* ]]; then
    continue
  fi
  echo "--- $(basename "$file") ---"
  for (( round=1; round<=ROUNDS; round++ )); do
    echo "round $round"
    run_once "$file"
  done
  echo
done

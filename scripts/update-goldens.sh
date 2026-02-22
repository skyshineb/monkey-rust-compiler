#!/usr/bin/env bash
set -euo pipefail

echo "==> updating compatibility goldens"
UPDATE_GOLDENS=1 cargo test compat_

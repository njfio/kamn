#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec python3 "$ROOT_DIR/scripts/ci/superseded_script_inventory.py" check \
  --repo-root "$ROOT_DIR" \
  "$@"

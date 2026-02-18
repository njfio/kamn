#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec python3 "$ROOT_DIR/scripts/ci/stale_script_reference_detector.py" \
  --repo-root "$ROOT_DIR" \
  "$@"

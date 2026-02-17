#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"

exec python3 "$KAMN_ROOT/scripts/lib/exec_dispatch.py" \
  --registry "$KAMN_ROOT/scripts/lib/exec_registry.json" \
  --invoked-path "$0" \
  -- \
  "$@"

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec python3 "$ROOT_DIR/scripts/runtime/test_run_sqlite_crash_restart_local_heavy_lane.py"

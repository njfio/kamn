#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec bash "$ROOT_DIR/scripts/ci/generate_test_harness_loc_report.sh" \
  --scripts-root "$ROOT_DIR/scripts/kolme" \
  "$@"

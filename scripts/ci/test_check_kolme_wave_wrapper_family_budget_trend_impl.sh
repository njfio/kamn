#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec bash "$ROOT_DIR/scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh" \
  --family kolme \
  "$@"

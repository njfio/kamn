#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/runtime/run_runtime_snapshot_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/runtime/run_runtime_snapshot_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected runtime snapshot fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected runtime snapshot deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "runtime snapshot contract lane tests passed." "$TMP_OUT"; then
  echo "expected runtime snapshot contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_runtime_snapshot_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute runtime snapshot fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "performance_file_snapshot_store_recovery_deep_lane_large_payload -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to run ignored snapshot recovery stress test" >&2
  exit 1
fi

echo "runtime snapshot contract lane script tests passed."

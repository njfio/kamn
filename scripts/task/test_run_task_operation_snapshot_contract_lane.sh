#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/task/run_task_operation_snapshot_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/task/run_task_operation_snapshot_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected task operation snapshot fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected task operation snapshot deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "task operation snapshot contract lane tests passed." "$TMP_OUT"; then
  echo "expected task operation snapshot contract lane success marker" >&2
  exit 1
fi

if ! grep -q "task_state_machine" "$FAST_SCRIPT"; then
  echo "expected task operation snapshot fast-lane to include task state machine contract tests" >&2
  exit 1
fi

if ! grep -q "task_escrow_transition_contracts" "$FAST_SCRIPT"; then
  echo "expected task operation snapshot fast-lane to include task/escrow transition contract tests" >&2
  exit 1
fi

if ! grep -Fq "run_task_operation_snapshot_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute task operation snapshot fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "performance_task_operation_snapshot_store_deep_lane_stress -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to run ignored task operation snapshot stress test" >&2
  exit 1
fi

echo "task operation snapshot contract lane script tests passed."

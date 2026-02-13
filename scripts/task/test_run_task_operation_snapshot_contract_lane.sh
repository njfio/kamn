#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/task/run_task_operation_snapshot_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/task/run_task_operation_snapshot_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/task/task_operation_snapshot_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/task_task_operation_snapshot_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected task operation snapshot fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected task operation snapshot deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected task operation snapshot shared contract-lane module to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "task operation snapshot contract lane tests passed." "$TMP_OUT"; then
  echo "expected task operation snapshot contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected task operation snapshot contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected task operation snapshot contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected task operation snapshot wrapper to resolve task manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "task_operation_snapshot_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected task operation snapshot manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -q "task_state_machine" "$SHARED_CONTRACT"; then
  echo "expected task operation snapshot shared contract module to include task state machine contract tests" >&2
  exit 1
fi

if ! grep -q "task_escrow_transition_contracts" "$SHARED_CONTRACT"; then
  echo "expected task operation snapshot shared contract module to include task/escrow transition contract tests" >&2
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

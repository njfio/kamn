#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/guard/run_durable_guard_recovery_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/guard/run_durable_guard_recovery_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/guard/durable_guard_recovery_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/guard_durable_guard_recovery_contract_lane.json"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected durable guard recovery fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected durable guard recovery deep-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected durable guard recovery shared contract module to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected durable guard recovery contract-lane manifest to exist" >&2
  exit 1
fi

if ! grep -q "run_manifest_lane.sh" "$FAST_SCRIPT"; then
  echo "expected durable guard fast-lane wrapper to delegate via manifest runner" >&2
  exit 1
fi
if ! grep -q "guard_durable_guard_recovery_contract_lane.json" "$FAST_SCRIPT"; then
  echo "expected durable guard fast-lane wrapper to reference guard manifest" >&2
  exit 1
fi
if ! grep -q "durable_guard_recovery_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected durable guard manifest to dispatch to shared contract module" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "durable guard recovery contract lane tests passed." "$TMP_OUT"; then
  echo "expected durable guard recovery contract lane success marker" >&2
  exit 1
fi

if ! grep -q "unit_delivery_guard_snapshot_rejects_schema_mismatch" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include delivery schema mismatch unit coverage" >&2
  exit 1
fi

if ! grep -q "unit_channel_policy_snapshot_rejects_schema_mismatch" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include channel policy schema mismatch unit coverage" >&2
  exit 1
fi

if ! grep -q "integration_durable_guard_recovery_matrix_restores_delivery_and_retention_invariants" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include integration recovery matrix coverage" >&2
  exit 1
fi

if ! grep -q "performance_durable_guard_recovery_contract_lane_budget" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include PR budget performance coverage" >&2
  exit 1
fi

if ! grep -q "integration_file_bundle_restore_preserves_invariants" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include durable guard snapshot store integration coverage" >&2
  exit 1
fi

if ! grep -q "performance_bundle_contract_lane_budget" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include durable guard snapshot store performance budget coverage" >&2
  exit 1
fi

if ! grep -q "release_gonogo_checklist_docs" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include release checklist docs coverage" >&2
  exit 1
fi

if ! grep -q "message_delivery_guards_docs" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include message delivery docs coverage" >&2
  exit 1
fi

if ! grep -q "channel_permissions_retention_docs" "$SHARED_CONTRACT"; then
  echo "expected durable guard recovery contract lane to include channel permissions docs coverage" >&2
  exit 1
fi

if ! grep -Fq "run_durable_guard_recovery_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute durable guard recovery fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "performance_durable_guard_recovery_matrix_deep_lane -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to run ignored durable guard recovery matrix performance test" >&2
  exit 1
fi

if ! grep -q "performance_bundle_store_deep_lane_stress -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to run ignored durable guard snapshot store performance test" >&2
  exit 1
fi

echo "durable guard recovery contract lane script tests passed."

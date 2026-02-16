#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme contract-lane dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

lane_wrappers=(
  "scripts/runtime/run_concurrency_state_mutation_deep_lane.sh"
  "scripts/runtime/run_runtime_snapshot_deep_lane.sh"
  "scripts/runtime/run_zk_witness_mutation_deep_lane.sh"
  "scripts/runtime/run_live_network_partition_reconnect_deep_lane.sh"
  "scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh"
  "scripts/runtime/run_failover_sync_drill_deep_lane.sh"
  "scripts/runtime/run_watchdog_proof_consensus_deep_lane.sh"
  "scripts/task/run_task_operation_snapshot_deep_lane.sh"
  "scripts/task/run_federated_delegation_settlement_deep_lane.sh"
)

for wrapper_rel_path in "${lane_wrappers[@]}"; do
  wrapper_path="$ROOT_DIR/$wrapper_rel_path"
  wrapper_name="$(basename "$wrapper_path")"

  if [ ! -x "$wrapper_path" ]; then
    echo "expected lightweight wrapper to be executable: $wrapper_path" >&2
    exit 1
  fi

  if [ ! -L "$wrapper_path" ]; then
    echo "expected lightweight wrapper to be a symlink to shared dispatcher: $wrapper_path" >&2
    exit 1
  fi

  manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$wrapper_name" --resolve-manifest-path)"
  if [ ! -f "$manifest_path" ]; then
    echo "expected dispatcher to resolve existing manifest for $wrapper_name: $manifest_path" >&2
    exit 1
  fi
done

set +e
unknown_wrapper_output="$(
  bash "$DISPATCHER" --lane-wrapper run_missing_non_kolme_wave19_lightweight_contract_lane.sh --resolve-manifest-path 2>&1
)"
unknown_wrapper_code=$?
set -e

if [ "$unknown_wrapper_code" -eq 0 ]; then
  echo "expected non-Kolme dispatcher to fail for unknown wave-19 lightweight wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^dispatch_status=fail$'; then
  echo "expected deterministic dispatcher fallback status marker for unknown wave-19 lightweight wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^fallback_reason_taxonomy_version=kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1$'; then
  echo "expected deterministic dispatcher fallback taxonomy marker for unknown wave-19 lightweight wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^fallback_reason_codes_csv=dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped$'; then
  echo "expected deterministic dispatcher fallback reason code set marker for unknown wave-19 lightweight wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^fallback_reason_code=dispatcher_unknown_wrapper$'; then
  echo "expected deterministic dispatcher fallback reason code marker for unknown wave-19 lightweight wrapper" >&2
  exit 1
fi

echo "non-Kolme wave-19 lightweight contract lane dispatcher wrapper matrix tests passed."

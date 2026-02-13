#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/runtime/run_runtime_snapshot_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/runtime_snapshot_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_runtime_snapshot_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/runtime/run_runtime_snapshot_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected runtime snapshot fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected runtime snapshot deep-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected runtime snapshot shared contract module to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "runtime snapshot contract lane tests passed." "$TMP_OUT"; then
  echo "expected runtime snapshot contract lane success marker" >&2
  exit 1
fi

if ! grep -q "functional_runtime_backpressure_classifies_queue_saturation" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include backpressure saturation functional coverage" >&2
  exit 1
fi

if ! grep -q "regression_runtime_backpressure_rejects_capacity_overflow_sample" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include backpressure overflow regression coverage" >&2
  exit 1
fi

if ! grep -q "functional_authenticated_peer_frame_roundtrips_wire_and_signature" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include authenticated peer frame roundtrip coverage" >&2
  exit 1
fi

if ! grep -q "regression_forged_or_unauthorized_peer_frame_is_rejected" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include forged/unauthorized peer frame regression coverage" >&2
  exit 1
fi

if ! grep -q "regression_replayed_peer_frame_nonce_is_rejected" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include replay nonce regression coverage" >&2
  exit 1
fi

if ! grep -q "runtime_watchdog_attestation_docs" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include runtime watchdog attestation docs coverage" >&2
  exit 1
fi

if ! grep -q "live_network_wave_docs" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include live-network wave docs coverage" >&2
  exit 1
fi

if ! grep -q "test_run_invariant_fuzz_concurrency_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include combined invariant/fuzz/concurrency contract coverage" >&2
  exit 1
fi

if ! grep -q "test_check_invariant_fuzz_concurrency_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include invariant/fuzz/concurrency policy checker coverage" >&2
  exit 1
fi

if ! grep -q "test_run_zk_witness_mutation_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include zk witness mutation fast-lane coverage" >&2
  exit 1
fi

if ! grep -q "test_run_zk_witness_mutation_deep_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include zk witness mutation deep-lane coverage" >&2
  exit 1
fi

if ! grep -q "test_run_processor_proof_admission_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include processor proof admission contract coverage" >&2
  exit 1
fi

if ! grep -q "test_generate_processor_proof_admission_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include processor proof admission evidence bundle coverage" >&2
  exit 1
fi

if ! grep -q "test_generate_watchdog_proof_consensus_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include watchdog proof consensus evidence bundle coverage" >&2
  exit 1
fi

if ! grep -q "test_run_watchdog_proof_consensus_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include watchdog proof consensus contract lane coverage" >&2
  exit 1
fi

if ! grep -q "test_run_watchdog_proof_consensus_deep_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include watchdog proof consensus deep-lane coverage" >&2
  exit 1
fi

if ! grep -q "test_select_failover_sync_drill_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include failover/sync selector contract coverage" >&2
  exit 1
fi

if ! grep -q "test_run_failover_sync_drill_preflight_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include failover/sync preflight lane coverage" >&2
  exit 1
fi

if ! grep -q "test_run_failover_sync_drill_deep_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include failover/sync deep lane coverage" >&2
  exit 1
fi

if ! grep -q "test_run_failover_sync_drill_suite.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include failover/sync suite coverage" >&2
  exit 1
fi

if ! grep -q "test_select_live_network_partition_reconnect_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include partition/reconnect lane selector coverage" >&2
  exit 1
fi

if ! grep -q "test_run_live_network_partition_reconnect_smoke_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include partition/reconnect smoke lane coverage" >&2
  exit 1
fi

if ! grep -q "test_run_live_network_partition_reconnect_deep_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include partition/reconnect deep lane coverage" >&2
  exit 1
fi

if ! grep -q "test_check_live_network_partition_reconnect_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include partition/reconnect policy checker coverage" >&2
  exit 1
fi

if ! grep -q "test_run_live_network_partition_reconnect_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include partition/reconnect contract lane coverage" >&2
  exit 1
fi

if ! grep -q "test_generate_live_network_pilot_artifact_summary.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include live-network pilot artifact summary coverage" >&2
  exit 1
fi

if ! grep -q "test_run_live_network_pilot_deep_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include live-network pilot deep lane coverage" >&2
  exit 1
fi

if ! grep -q "test_run_live_network_pilot_deep_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include live-network pilot deep contract coverage" >&2
  exit 1
fi

if ! grep -q "run_live_network_smoke_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected runtime snapshot contract lane to include live-network smoke contract lane coverage" >&2
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

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected runtime snapshot wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected runtime snapshot wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected runtime snapshot wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "runtime_snapshot_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected runtime snapshot manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "runtime snapshot contract lane script tests passed."

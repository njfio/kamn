#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_contract_lane_dispatch.sh"
DELETION_MANIFEST="$ROOT_DIR/fixtures/ci/superseded_script_deletion_manifest.json"

is_superseded_deleted_wrapper() {
  local wrapper_rel="$1"
  python3 - "$DELETION_MANIFEST" "$wrapper_rel" <<'PY'
import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
wrapper_rel = sys.argv[2]
if not manifest_path.is_file():
    raise SystemExit(1)
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
for entry in payload.get("deletions", []):
    if isinstance(entry, dict) and entry.get("script_path") == wrapper_rel:
        raise SystemExit(0)
raise SystemExit(1)
PY
}

if [ ! -x "$DISPATCHER" ]; then
  echo "expected Kolme contract-lane dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

lane_wrappers=(
  "run_block_fallback_reconciliation_contract_lane.sh"
  "run_continuous_runtime_commit_contract_lane.sh"
  "run_did_lifecycle_chain_adapter_contract_lane.sh"
  "run_fast_gate_native_api_parity_contract_lane.sh"
  "run_local_bootstrap_health_checks_contract_lane.sh"
  "run_local_e2e_integration_contract_lane.sh"
  "run_local_heavy_validation_matrix_contract_lane.sh"
  "run_local_kamn_live_runtime_integration_contract_lane.sh"
  "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"
  "run_local_live_provider_runtime_integration_contract_lane.sh"
  "run_local_kolme_live_deployment_preflight_contract_lane.sh"
  "run_local_live_node_validation_bundle_contract_lane.sh"
  "run_local_kolme_fork_bootstrap_readiness_contract_lane.sh"
  "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh"
  "run_local_kolme_fork_portability_preflight_contract_lane.sh"
  "run_local_kolme_fork_process_lifecycle_contract_lane.sh"
  "run_local_kolme_fork_profile_preflight_contract_lane.sh"
  "run_local_kolme_fork_real_process_contract_lane.sh"
  "run_local_kolme_fork_rust_test_matrix_contract_lane.sh"
  "run_local_kolme_fork_self_test_contract_lane.sh"
  "run_local_kolme_live_api_conformance_contract_lane.sh"
  "run_managed_signer_backend_slo_policy_contract_lane.sh"
  "run_managed_signer_backend_slo_telemetry_contract_lane.sh"
  "run_managed_signer_startup_live_validation_contract_lane.sh"
  "run_message_proof_anchoring_contract_lane.sh"
  "run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
  "run_local_native_api_parity_live_proof_contract_lane.sh"
  "run_local_signed_to_kolme_demo_contract_lane.sh"
  "run_nonce_broadcast_parity_contract_lane.sh"
  "run_notifications_consumer_contract_lane.sh"
  "run_onchain_lifecycle_evidence_contract_lane.sh"
  "run_runtime_commit_adapter_contract_lane.sh"
  "run_runtime_commit_contract_lane.sh"
  "run_runtime_commit_replay_contract_lane.sh"
  "run_signature_parity_contract_lane.sh"
  "run_snapshot_drift_contract_lane.sh"
  "run_triadic_devnet_smoke_contract_lane.sh"
  "run_version_compatibility_contract_lane.sh"
)

for wrapper in "${lane_wrappers[@]}"; do
  wrapper_path="$ROOT_DIR/scripts/kolme/$wrapper"
  wrapper_rel="scripts/kolme/$wrapper"

  if [ -e "$wrapper_path" ]; then
    if [ ! -x "$wrapper_path" ]; then
      echo "expected lane wrapper to be executable: $wrapper_path" >&2
      exit 1
    fi

    if [ ! -L "$wrapper_path" ]; then
      echo "expected lane wrapper to be a symlink to shared dispatcher: $wrapper_path" >&2
      exit 1
    fi
  elif ! is_superseded_deleted_wrapper "$wrapper_rel"; then
    echo "expected lane wrapper to exist or be cataloged as superseded/deleted: $wrapper_path" >&2
    exit 1
  fi

  manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$wrapper" --resolve-manifest-path)"
  if [ ! -f "$manifest_path" ]; then
    echo "expected dispatcher to resolve existing manifest for $wrapper: $manifest_path" >&2
    exit 1
  fi
done

if bash "$DISPATCHER" --lane-wrapper run_missing_contract_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected dispatcher to fail for unknown lane wrapper" >&2
  exit 1
fi

echo "Kolme contract lane dispatcher wrapper matrix tests passed."

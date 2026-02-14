#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
WRAPPER_NAME="$SCRIPT_NAME"
RESOLVE_MANIFEST_ONLY=0

usage() {
  cat <<'EOF'
Usage:
  bash scripts/kolme/run_contract_lane_dispatch.sh --lane-wrapper <wrapper-name> [--resolve-manifest-path] [-- <lane-args...>]

Wrapper compatibility mode:
  scripts/kolme/run_<lane>_contract_lane.sh [lane-args...]
EOF
}

if [[ "$SCRIPT_NAME" == "run_contract_lane_dispatch.sh" ]]; then
  while (($# > 0)); do
    case "$1" in
      --lane-wrapper)
        if (($# < 2)); then
          echo "missing value for --lane-wrapper" >&2
          usage
          exit 1
        fi
        WRAPPER_NAME="$2"
        shift 2
        ;;
      --resolve-manifest-path)
        RESOLVE_MANIFEST_ONLY=1
        shift
        ;;
      --)
        shift
        break
        ;;
      *)
        echo "unknown dispatcher argument: $1" >&2
        usage
        exit 1
        ;;
    esac
  done

  if [[ -z "$WRAPPER_NAME" || "$WRAPPER_NAME" == "run_contract_lane_dispatch.sh" ]]; then
    echo "--lane-wrapper is required when invoking the dispatcher directly" >&2
    usage
    exit 1
  fi
fi

resolve_manifest_name() {
  case "$1" in
    run_block_fallback_reconciliation_contract_lane.sh) echo "kolme_block_fallback_reconciliation_contract_lane.json" ;;
    run_fast_gate_native_api_parity_contract_lane.sh) echo "kolme_fast_gate_native_api_parity_contract_lane.json" ;;
    run_local_bootstrap_health_checks_contract_lane.sh) echo "kolme_local_bootstrap_health_checks_contract_lane.json" ;;
    run_local_e2e_integration_contract_lane.sh) echo "kolme_local_e2e_integration_contract_lane.json" ;;
    run_local_heavy_validation_matrix_contract_lane.sh) echo "kolme_local_heavy_validation_matrix_contract_lane.json" ;;
    run_local_kamn_live_runtime_integration_contract_lane.sh) echo "kolme_local_kamn_live_runtime_integration_contract_lane.json" ;;
    run_local_kamn_live_runtime_real_node_profile_contract_lane.sh) echo "kolme_local_kamn_live_runtime_real_node_profile_contract_lane.json" ;;
    run_local_live_provider_runtime_integration_contract_lane.sh) echo "kolme_local_live_provider_runtime_integration_contract_lane.json" ;;
    run_local_kolme_live_deployment_preflight_contract_lane.sh) echo "kolme_local_kolme_live_deployment_preflight_contract_lane.json" ;;
    run_local_live_node_validation_bundle_contract_lane.sh) echo "kolme_local_live_node_validation_bundle_contract_lane.json" ;;
    run_local_kolme_fork_bootstrap_readiness_contract_lane.sh) echo "kolme_local_kolme_fork_bootstrap_readiness_contract_lane.json" ;;
    run_local_kolme_fork_checkout_bootstrap_contract_lane.sh) echo "kolme_local_kolme_fork_checkout_bootstrap_contract_lane.json" ;;
    run_local_kolme_fork_portability_preflight_contract_lane.sh) echo "kolme_local_fork_portability_preflight_contract_lane.json" ;;
    run_local_kolme_fork_process_lifecycle_contract_lane.sh) echo "kolme_local_kolme_fork_process_lifecycle_contract_lane.json" ;;
    run_local_kolme_fork_profile_preflight_contract_lane.sh) echo "kolme_local_fork_profile_preflight_contract_lane.json" ;;
    run_local_kolme_fork_real_process_contract_lane.sh) echo "kolme_local_kolme_fork_real_process_contract_lane.json" ;;
    run_local_kolme_fork_rust_test_matrix_contract_lane.sh) echo "kolme_local_fork_rust_test_matrix_contract_lane.json" ;;
    run_local_kolme_fork_self_test_contract_lane.sh) echo "kolme_local_fork_self_test_contract_lane.json" ;;
    run_local_kolme_live_api_conformance_contract_lane.sh) echo "kolme_local_kolme_live_api_conformance_contract_lane.json" ;;
    run_managed_signer_backend_slo_policy_contract_lane.sh) echo "kolme_managed_signer_backend_slo_policy_contract_lane.json" ;;
    run_managed_signer_backend_slo_telemetry_contract_lane.sh) echo "kolme_managed_signer_backend_slo_telemetry_contract_lane.json" ;;
    run_managed_signer_startup_live_validation_contract_lane.sh) echo "kolme_managed_signer_startup_live_validation_contract_lane.json" ;;
    run_local_runtime_commit_live_finality_evidence_contract_lane.sh) echo "kolme_local_runtime_commit_live_finality_evidence_contract_lane.json" ;;
    run_local_native_api_parity_live_proof_contract_lane.sh) echo "kolme_local_native_api_parity_live_proof_contract_lane.json" ;;
    run_local_signed_to_kolme_demo_contract_lane.sh) echo "kolme_local_signed_to_kolme_demo_contract_lane.json" ;;
    run_nonce_broadcast_parity_contract_lane.sh) echo "kolme_nonce_broadcast_parity_contract_lane.json" ;;
    run_notifications_consumer_contract_lane.sh) echo "kolme_notifications_consumer_contract_lane.json" ;;
    run_runtime_commit_adapter_contract_lane.sh) echo "kolme_runtime_commit_adapter_contract_lane.json" ;;
    run_runtime_commit_contract_lane.sh) echo "kolme_runtime_commit_contract_lane.json" ;;
    run_runtime_commit_replay_contract_lane.sh) echo "kolme_runtime_commit_replay_contract_lane.json" ;;
    run_signature_parity_contract_lane.sh) echo "kolme_signature_parity_contract_lane.json" ;;
    run_snapshot_drift_contract_lane.sh) echo "kolme_snapshot_drift_contract_lane.json" ;;
    run_triadic_devnet_smoke_contract_lane.sh) echo "kolme_triadic_devnet_smoke_contract_lane.json" ;;
    run_version_compatibility_contract_lane.sh) echo "kolme_version_compatibility_contract_lane.json" ;;
    *)
      return 1
      ;;
  esac
}

MANIFEST_FILE="$(resolve_manifest_name "$WRAPPER_NAME" || true)"
if [[ -z "$MANIFEST_FILE" ]]; then
  echo "unknown lane wrapper for dispatch: $WRAPPER_NAME" >&2
  exit 1
fi

MANIFEST_PATH="$ROOT_DIR/scripts/framework/manifests/$MANIFEST_FILE"
if [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "resolved manifest does not exist: $MANIFEST_PATH" >&2
  exit 1
fi

if [[ "$RESOLVE_MANIFEST_ONLY" -eq 1 ]]; then
  echo "$MANIFEST_PATH"
  exit 0
fi

exec bash "$ROOT_DIR/scripts/framework/run_manifest_lane.sh" \
  --manifest "$MANIFEST_PATH" \
  --phase contract \
  -- \
  "$@"

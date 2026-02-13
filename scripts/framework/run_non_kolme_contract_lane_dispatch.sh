#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
WRAPPER_NAME="$SCRIPT_NAME"
RESOLVE_MANIFEST_ONLY=0

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/framework/run_non_kolme_contract_lane_dispatch.sh --lane-wrapper <wrapper-name> [--resolve-manifest-path] [-- <lane-args...>]

Wrapper compatibility mode:
  scripts/governance/run_<lane>_contract_lane.sh [lane-args...]
USAGE
}

if [[ "$SCRIPT_NAME" == "run_non_kolme_contract_lane_dispatch.sh" ]]; then
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

  if [[ -z "$WRAPPER_NAME" || "$WRAPPER_NAME" == "run_non_kolme_contract_lane_dispatch.sh" ]]; then
    echo "--lane-wrapper is required when invoking the dispatcher directly" >&2
    usage
    exit 1
  fi
fi

resolve_manifest_name() {
  case "$1" in
    run_backend_session_auth_freshness_contract_lane.sh) echo "dashboard_backend_session_auth_freshness_contract_lane.json" ;;
    run_dashboard_stale_error_budget_contract_lane.sh) echo "dashboard_stale_error_budget_contract_lane.json" ;;
    run_durable_guard_recovery_contract_lane.sh) echo "guard_durable_guard_recovery_contract_lane.json" ;;
    run_launch_canary_contract_lane.sh) echo "canary_launch_canary_contract_lane.json" ;;
    run_post_cutover_slo_contract_lane.sh) echo "canary_post_cutover_slo_contract_lane.json" ;;
    run_classification_redaction_contract_lane.sh) echo "compliance_classification_redaction_contract_lane.json" ;;
    run_dsar_legal_hold_contract_lane.sh) echo "compliance_dsar_legal_hold_contract_lane.json" ;;
    run_reputation_dispute_contract_lane.sh) echo "reputation_dispute_contract_lane.json" ;;
    run_governance_lifecycle_rollback_contract_lane.sh) echo "governance_lifecycle_rollback_contract_lane.json" ;;
    run_governance_simulation_contract_lane.sh) echo "governance_simulation_contract_lane.json" ;;
    run_quorum_attestation_replay_contract_lane.sh) echo "governance_quorum_attestation_replay_contract_lane.json" ;;
    run_soc2_control_evidence_contract_lane.sh) echo "compliance_soc2_control_evidence_contract_lane.json" ;;
    run_stake_slash_risk_contract_lane.sh) echo "governance_stake_slash_risk_contract_lane.json" ;;
    run_token_launch_handoff_contract_lane.sh) echo "token_launch_handoff_contract_lane.json" ;;
    run_treasury_disbursement_contract_lane.sh) echo "treasury_disbursement_contract_lane.json" ;;
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

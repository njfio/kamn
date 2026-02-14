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
    run_bridge_adapter_conformance_contract_lane.sh) echo "bridge_bridge_adapter_conformance_contract_lane.json" ;;
    run_bridge_credentialed_contract_lane.sh) echo "bridge_bridge_credentialed_contract_lane.json" ;;
    run_bridge_ingress_relay_contract_lane.sh) echo "bridge_bridge_ingress_relay_contract_lane.json" ;;
    run_bridge_outbound_quorum_contract_lane.sh) echo "bridge_bridge_outbound_quorum_contract_lane.json" ;;
    run_bridge_replay_redaction_contract_lane.sh) echo "bridge_bridge_replay_redaction_contract_lane.json" ;;
    run_cutover_rollback_contract_lane.sh) echo "cutover_cutover_rollback_contract_lane.json" ;;
    run_dr_evidence_contract_lane.sh) echo "deploy_dr_evidence_contract_lane.json" ;;
    run_deployment_slo_rollback_lane.sh) echo "deploy_deployment_slo_rollback_lane.json" ;;
    run_backend_session_auth_freshness_contract_lane.sh) echo "dashboard_backend_session_auth_freshness_contract_lane.json" ;;
    run_backend_session_auth_freshness_lane.sh) echo "dashboard_backend_session_auth_freshness_lane.json" ;;
    run_cross_chain_outbound_intent_contract_lane.sh) echo "bridge_cross_chain_outbound_intent_contract_lane.json" ;;
    run_dashboard_stale_error_budget_contract_lane.sh) echo "dashboard_stale_error_budget_contract_lane.json" ;;
    run_dashboard_stale_error_budget_lane.sh) echo "dashboard_stale_error_budget_lane.json" ;;
    run_did_registry_contract_lane.sh) echo "did_did_registry_contract_lane.json" ;;
    run_durable_guard_recovery_contract_lane.sh) echo "guard_durable_guard_recovery_contract_lane.json" ;;
    run_federated_did_handshake_contract_lane.sh) echo "did_federated_did_handshake_contract_lane.json" ;;
    run_launch_canary_contract_lane.sh) echo "canary_launch_canary_contract_lane.json" ;;
    run_localhost_bridge_demo_evidence_contract_lane.sh) echo "bridge_localhost_bridge_demo_evidence_contract_lane.json" ;;
    run_localhost_bridge_relay_demo_contract_lane.sh) echo "bridge_localhost_bridge_relay_demo_contract_lane.json" ;;
    run_post_cutover_slo_contract_lane.sh) echo "canary_post_cutover_slo_contract_lane.json" ;;
    run_classification_redaction_contract_lane.sh) echo "compliance_classification_redaction_contract_lane.json" ;;
    run_classification_redaction_lane.sh) echo "compliance_classification_redaction_lane.json" ;;
    run_dsar_legal_hold_contract_lane.sh) echo "compliance_dsar_legal_hold_contract_lane.json" ;;
    run_channel_lifecycle_contract_lane.sh) echo "channel_channel_lifecycle_contract_lane.json" ;;
    run_channel_policy_contract_lane.sh) echo "channel_channel_policy_contract_lane.json" ;;
    run_dashboard_shell_determinism_matrix_contract_lane.sh) echo "frontend_dashboard_shell_determinism_matrix_contract_lane.json" ;;
    run_dashboard_shell_determinism_matrix_lane.sh) echo "frontend_dashboard_shell_determinism_matrix_lane.json" ;;
    run_a2a_mcp_conformance_contract_lane.sh) echo "message_a2a_mcp_conformance_contract_lane.json" ;;
    run_didcomm_envelope_compatibility_contract_lane.sh) echo "message_didcomm_envelope_compatibility_contract_lane.json" ;;
    run_group_sender_replay_ratchet_contract_lane.sh) echo "message_group_sender_replay_ratchet_contract_lane.json" ;;
    run_key_hierarchy_invariant_contract_lane.sh) echo "message_key_hierarchy_invariant_contract_lane.json" ;;
    run_processor_proof_artifact_contract_lane.sh) echo "message_processor_proof_artifact_contract_lane.json" ;;
    run_message_lifecycle_contract_lane.sh) echo "message_message_lifecycle_contract_lane.json" ;;
    run_reputation_dispute_contract_lane.sh) echo "reputation_dispute_contract_lane.json" ;;
    run_reputation_recovery_contract_lane.sh) echo "reputation_reputation_recovery_contract_lane.json" ;;
    run_reputation_signal_quarantine_contract_lane.sh) echo "reputation_reputation_signal_quarantine_contract_lane.json" ;;
    run_weighted_decay_contract_lane.sh) echo "reputation_weighted_decay_contract_lane.json" ;;
    run_channel_retention_redaction_contract_lane.sh) echo "channel_channel_retention_redaction_contract_lane.json" ;;
    run_settlement_reconciliation_contract_lane.sh) echo "escrow_settlement_reconciliation_contract_lane.json" ;;
    run_federated_delegation_settlement_contract_lane.sh) echo "task_federated_delegation_settlement_contract_lane.json" ;;
    run_deployment_slo_rollback_contract_lane.sh) echo "deploy_deployment_slo_rollback_contract_lane.json" ;;
    run_service_endpoint_canonicalization_contract_lane.sh) echo "did_service_endpoint_canonicalization_contract_lane.json" ;;
    run_localhost_signed_demo_contract_lane.sh) echo "sdk_localhost_signed_demo_contract_lane.json" ;;
    run_rust_live_transport_contract_lane.sh) echo "sdk_rust_live_transport_contract_lane.json" ;;
    run_governance_lifecycle_rollback_contract_lane.sh) echo "governance_lifecycle_rollback_contract_lane.json" ;;
    run_governance_lifecycle_rollback_lane.sh) echo "governance_lifecycle_rollback_lane.json" ;;
    run_governance_simulation_contract_lane.sh) echo "governance_simulation_contract_lane.json" ;;
    run_quorum_attestation_replay_guard_lane.sh) echo "governance_quorum_attestation_replay_guard_lane.json" ;;
    run_gonogo_evidence_contract_lane.sh) echo "deploy_gonogo_evidence_contract_lane.json" ;;
    run_mainnet_cutover_contract_lane.sh) echo "cutover_mainnet_cutover_contract_lane.json" ;;
    run_quorum_attestation_replay_contract_lane.sh) echo "governance_quorum_attestation_replay_contract_lane.json" ;;
    run_example_fixture_drift_contract_lane.sh) echo "sdk_example_fixture_drift_contract_lane.json" ;;
    run_live_transport_parity_contract_lane.sh) echo "sdk_live_transport_parity_contract_lane.json" ;;
    run_live_transport_replay_tamper_contract_lane.sh) echo "sdk_live_transport_replay_tamper_contract_lane.json" ;;
    run_live_transport_replay_tamper_fast_lane.sh) echo "sdk_live_transport_replay_tamper_fast_lane.json" ;;
    run_live_transport_smoke_parity_contract_lane.sh) echo "sdk_live_transport_smoke_parity_contract_lane.json" ;;
    run_live_transport_smoke_parity_lane.sh) echo "sdk_live_transport_smoke_parity_lane.json" ;;
    run_lifecycle_operator_binding_contract_lane.sh) echo "did_lifecycle_operator_binding_contract_lane.json" ;;
    run_localhost_signed_integration_contract_lane.sh) echo "sdk_localhost_signed_integration_contract_lane.json" ;;
    run_multikey_algorithm_policy_contract_lane.sh) echo "did_multikey_algorithm_policy_contract_lane.json" ;;
    run_sdk_schema_compatibility_contract_lane.sh) echo "sdk_schema_compatibility_contract_lane.json" ;;
    run_signer_emulator_contract_lane.sh) echo "signer_signer_emulator_contract_lane.json" ;;
    run_signer_incident_recovery_deep_lane.sh) echo "signer_signer_incident_recovery_deep_lane.json" ;;
    run_signer_incident_recovery_contract_lane.sh) echo "signer_signer_incident_recovery_contract_lane.json" ;;
    run_signer_incident_recovery_lane.sh) echo "signer_signer_incident_recovery_lane.json" ;;
    run_signer_provider_deep_lane.sh) echo "signer_signer_provider_deep_lane.json" ;;
    run_signer_policy_contract_lane.sh) echo "signer_signer_policy_contract_lane.json" ;;
    run_kamn_core_rustdoc_artifact_contract_lane.sh) echo "ci_kamn_core_rustdoc_artifact_contract_lane.json" ;;
    run_test_harness_loc_soft_budget_contract_lane.sh) echo "ci_test_harness_loc_soft_budget_contract_lane.json" ;;
    run_kolme_test_harness_loc_soft_budget_contract_lane.sh) echo "ci_kolme_test_harness_loc_soft_budget_contract_lane.json" ;;
    run_secure_provider_key_lifecycle_contract_lane.sh) echo "signer_secure_provider_key_lifecycle_contract_lane.json" ;;
    run_staging_rehearsal_contract_lane.sh) echo "deploy_staging_rehearsal_contract_lane.json" ;;
    run_task_operation_snapshot_contract_lane.sh) echo "task_task_operation_snapshot_contract_lane.json" ;;
    run_concurrency_state_mutation_contract_lane.sh) echo "runtime_concurrency_state_mutation_contract_lane.json" ;;
    run_failover_sync_drill_preflight_contract_lane.sh) echo "runtime_failover_sync_drill_preflight_contract_lane.json" ;;
    run_input_mutation_contract_lane.sh) echo "runtime_input_mutation_contract_lane.json" ;;
    run_input_mutation_coverage_guided_contract_lane.sh) echo "runtime_input_mutation_coverage_guided_contract_lane.json" ;;
    run_invariant_fuzz_concurrency_contract_lane.sh) echo "runtime_invariant_fuzz_concurrency_contract_lane.json" ;;
    run_lifecycle_property_contract_lane.sh) echo "runtime_lifecycle_property_contract_lane.json" ;;
    run_live_network_partition_reconnect_contract_lane.sh) echo "runtime_live_network_partition_reconnect_contract_lane.json" ;;
    run_live_network_pilot_deep_contract_lane.sh) echo "runtime_live_network_pilot_deep_contract_lane.json" ;;
    run_live_network_smoke_contract_lane.sh) echo "runtime_live_network_smoke_contract_lane.json" ;;
    run_processor_proof_admission_contract_lane.sh) echo "runtime_processor_proof_admission_contract_lane.json" ;;
    run_runtime_snapshot_contract_lane.sh) echo "runtime_runtime_snapshot_contract_lane.json" ;;
    run_watchdog_proof_consensus_contract_lane.sh) echo "runtime_watchdog_proof_consensus_contract_lane.json" ;;
    run_zk_witness_mutation_contract_lane.sh) echo "runtime_zk_witness_mutation_contract_lane.json" ;;
    run_soc2_control_evidence_contract_lane.sh) echo "compliance_soc2_control_evidence_contract_lane.json" ;;
    run_stake_slash_risk_contract_lane.sh) echo "governance_stake_slash_risk_contract_lane.json" ;;
    run_telegram_ingress_contract_lane.sh) echo "bridge_telegram_ingress_contract_lane.json" ;;
    run_token_launch_handoff_contract_lane.sh) echo "token_launch_handoff_contract_lane.json" ;;
    run_treasury_disbursement_contract_lane.sh) echo "treasury_disbursement_contract_lane.json" ;;
    *)
      return 1
      ;;
  esac
}

resolve_phase_name() {
  case "$1" in
    run_live_transport_replay_tamper_fast_lane.sh) echo "run" ;;
    run_live_transport_smoke_parity_lane.sh) echo "run" ;;
    run_quorum_attestation_replay_guard_lane.sh) echo "run" ;;
    run_governance_lifecycle_rollback_lane.sh) echo "run" ;;
    run_dashboard_shell_determinism_matrix_lane.sh) echo "run" ;;
    run_deployment_slo_rollback_lane.sh) echo "run" ;;
    run_classification_redaction_lane.sh) echo "run" ;;
    run_backend_session_auth_freshness_lane.sh) echo "run" ;;
    run_dashboard_stale_error_budget_lane.sh) echo "run" ;;
    run_signer_incident_recovery_deep_lane.sh) echo "deep" ;;
    run_signer_provider_deep_lane.sh) echo "deep" ;;
    run_signer_incident_recovery_lane.sh) echo "run" ;;
    *) echo "contract" ;;
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

PHASE_NAME="$(resolve_phase_name "$WRAPPER_NAME")"

exec bash "$ROOT_DIR/scripts/framework/run_manifest_lane.sh" \
  --manifest "$MANIFEST_PATH" \
  --phase "$PHASE_NAME" \
  -- \
  "$@"

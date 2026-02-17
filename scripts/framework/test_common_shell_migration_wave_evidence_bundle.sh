#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

TARGETS=(
  "scripts/bridge/test_generate_bridge_adapter_conformance_evidence_bundle.sh"
  "scripts/bridge/test_generate_bridge_replay_redaction_evidence_bundle.sh"
  "scripts/bridge/test_generate_cross_chain_outbound_intent_evidence_bundle.sh"
  "scripts/bridge/test_generate_localhost_bridge_demo_evidence_bundle.sh"
  "scripts/canary/test_generate_post_cutover_slo_evidence_bundle.sh"
  "scripts/channel/test_generate_channel_retention_redaction_evidence_bundle.sh"
  "scripts/compliance/test_generate_dsar_legal_hold_evidence_bundle.sh"
  "scripts/compliance/test_generate_soc2_control_evidence_bundle.sh"
  "scripts/cutover/test_generate_cutover_rollback_evidence_bundle.sh"
  "scripts/did/test_generate_federated_did_handshake_evidence_bundle.sh"
  "scripts/did/test_generate_lifecycle_operator_binding_evidence_bundle.sh"
  "scripts/escrow/test_generate_settlement_reconciliation_evidence_bundle.sh"
  "scripts/governance/test_generate_governance_simulation_evidence_bundle.sh"
  "scripts/governance/test_generate_stake_slash_risk_evidence_bundle.sh"
  "scripts/kolme/test_generate_fork_compatibility_evidence.sh"
  "scripts/message/test_generate_group_sender_replay_ratchet_evidence_bundle.sh"
  "scripts/message/test_generate_key_lifecycle_invariant_evidence_bundle.sh"
  "scripts/message/test_generate_processor_proof_artifact_evidence_bundle.sh"
  "scripts/reputation/test_generate_weighted_decay_property_evidence_bundle.sh"
  "scripts/runtime/test_generate_live_network_pilot_artifact_summary.sh"
  "scripts/sdk/test_generate_live_transport_replay_tamper_evidence_bundle.sh"
  "scripts/signer/test_generate_secure_provider_key_lifecycle_evidence_bundle.sh"
  "scripts/task/test_generate_federated_delegation_settlement_evidence_bundle.sh"
  "scripts/token/test_generate_token_launch_handoff_evidence_bundle.sh"
  "scripts/treasury/test_generate_treasury_disbursement_evidence_bundle.sh"
)

failures=0
for rel in "${TARGETS[@]}"; do
  path="$ROOT_DIR/$rel"
  if [[ ! -f "$path" ]]; then
    echo "missing target script: $rel" >&2
    failures=$((failures + 1))
    continue
  fi
  if ! grep -q "scripts/lib/common.sh" "$path"; then
    echo "expected common.sh sourcing in $rel" >&2
    failures=$((failures + 1))
  fi
  if grep -q '^ROOT_DIR=' "$path"; then
    echo "legacy ROOT_DIR bootstrap still present in $rel" >&2
    failures=$((failures + 1))
  fi
  if grep -q '^extract_value() {' "$path"; then
    echo "duplicated extract_value helper still present in $rel" >&2
    failures=$((failures + 1))
  fi
  if grep -q '^assert_eq() {' "$path"; then
    echo "duplicated assert_eq helper still present in $rel" >&2
    failures=$((failures + 1))
  fi
done

if [[ "$failures" -ne 0 ]]; then
  echo "common shell migration wave contract test failed with $failures violation(s)." >&2
  exit 1
fi

echo "common shell migration wave contract test passed."

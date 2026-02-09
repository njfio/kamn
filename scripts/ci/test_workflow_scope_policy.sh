#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
DEEP_WORKFLOW="$ROOT_DIR/.github/workflows/ci-deep-validate.yml"

if ! grep -Fq "steps.scope.outputs.run_deploy_preflight_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected deploy preflight scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_preflight_topology.sh" "$FAST_WORKFLOW"; then
  echo "expected deploy preflight topology tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_generate_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected deploy bundle generator tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected go/no-go evidence bundle tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected go/no-go evidence contract lane script tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_generate_staging_rehearsal_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected staging rehearsal bundle tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_run_staging_rehearsal_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected staging rehearsal contract lane script tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_generate_dr_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected DR evidence bundle tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_run_dr_evidence_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected DR evidence contract lane script tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_frontend_dashboard_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected frontend dashboard scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/frontend/test_dashboard_package.sh" "$FAST_WORKFLOW"; then
  echo "expected frontend dashboard test command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_dashboard_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected dashboard contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/frontend/test_dashboard_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected dashboard contract test command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_python_live_transport_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected python live transport scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "python3 -m unittest tests/python/test_sdk.py" "$FAST_WORKFLOW"; then
  echo "expected python live transport lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_typescript_live_transport_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected typescript live transport scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "npm --prefix packages/kamn-sdk test" "$FAST_WORKFLOW"; then
  echo "expected typescript live transport lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_rust_live_transport_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected rust live transport scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/sdk/run_rust_live_transport_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected rust live transport lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_live_transport_parity_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected live transport parity scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/sdk/run_live_transport_parity_contract_lane.sh --languages \"\${{ steps.scope.outputs.live_transport_parity_languages }}\"" "$FAST_WORKFLOW"; then
  echo "expected live transport parity lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "steps.scope.outputs.run_live_transport_parity_rust_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected rust setup/cache parity subset guard in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_signer_emulator_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected signer emulator contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/signer/run_signer_emulator_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected signer emulator contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_did_registry_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected did registry contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/did/run_did_registry_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected did registry contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_federated_did_handshake_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected federated DID handshake contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/did/test_generate_federated_did_handshake_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected federated DID handshake generator tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/did/test_run_federated_did_handshake_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected federated DID handshake contract lane tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/did/test_run_federated_did_handshake_matrix.sh" "$FAST_WORKFLOW"; then
  echo "expected federated DID handshake matrix tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/did/test_run_federated_did_handshake_deep_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected federated DID handshake deep lane tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_kolme_snapshot_drift_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected Kolme snapshot drift contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/kolme/test_check_snapshot_drift.sh" "$FAST_WORKFLOW"; then
  echo "expected Kolme snapshot drift checker tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/kolme/test_run_snapshot_drift_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected Kolme snapshot drift contract lane tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_kolme_version_compatibility_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected Kolme version compatibility contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/kolme/test_validate_version_compatibility.sh" "$FAST_WORKFLOW"; then
  echo "expected Kolme version compatibility validator tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/kolme/test_run_version_compatibility_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected Kolme version compatibility contract lane tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_kolme_triadic_devnet_smoke_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected Kolme triadic devnet smoke contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/kolme/test_validate_triadic_devnet_smoke.sh" "$FAST_WORKFLOW"; then
  echo "expected Kolme triadic devnet smoke validator tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected Kolme triadic devnet smoke contract lane tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_federated_delegation_settlement_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected federated delegation settlement contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/task/test_generate_federated_delegation_settlement_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected federated delegation settlement generator tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected federated delegation settlement contract lane tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/task/test_run_federated_delegation_settlement_matrix.sh" "$FAST_WORKFLOW"; then
  echo "expected federated delegation settlement matrix tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/task/test_run_federated_delegation_settlement_deep_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected federated delegation settlement deep lane tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_runtime_snapshot_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected runtime snapshot contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/runtime/run_runtime_snapshot_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected runtime snapshot contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_message_lifecycle_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected message lifecycle contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/message/run_message_lifecycle_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected message lifecycle contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_channel_lifecycle_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected channel lifecycle contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/channel/run_channel_lifecycle_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected channel lifecycle contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_task_operation_snapshot_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected task operation snapshot contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/task/run_task_operation_snapshot_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected task operation snapshot contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_durable_guard_recovery_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected durable guard recovery contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/guard/run_durable_guard_recovery_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected durable guard recovery contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_settlement_reconciliation_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected settlement reconciliation contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/escrow/run_settlement_reconciliation_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected settlement reconciliation contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_soc2_control_evidence_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected SOC2 control evidence contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/compliance/test_generate_soc2_control_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected SOC2 control evidence bundle tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/compliance/test_run_soc2_control_evidence_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected SOC2 control evidence contract lane script tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/compliance/test_run_soc2_control_evidence_replay_matrix.sh" "$FAST_WORKFLOW"; then
  echo "expected SOC2 control evidence replay matrix tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_dsar_legal_hold_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected DSAR legal-hold contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/compliance/test_generate_dsar_legal_hold_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected DSAR legal-hold evidence bundle tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/compliance/test_run_dsar_legal_hold_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected DSAR legal-hold contract lane script tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/compliance/test_run_dsar_legal_hold_matrix.sh" "$FAST_WORKFLOW"; then
  echo "expected DSAR legal-hold matrix tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_governance_simulation_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected governance simulation contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/governance/test_generate_governance_simulation_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected governance simulation evidence bundle tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/governance/test_run_governance_simulation_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected governance simulation contract lane script tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/governance/test_run_governance_simulation_matrix.sh" "$FAST_WORKFLOW"; then
  echo "expected governance simulation matrix tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_governance_stake_slash_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected governance stake/slash contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/governance/test_generate_stake_slash_risk_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected governance stake/slash evidence bundle tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/governance/test_run_stake_slash_risk_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected governance stake/slash contract lane script tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/governance/test_run_stake_slash_risk_matrix.sh" "$FAST_WORKFLOW"; then
  echo "expected governance stake/slash matrix tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_reputation_decay_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected weighted decay anti-gaming contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/reputation/run_weighted_decay_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected weighted decay anti-gaming contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_reputation_dispute_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected reputation dispute contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/reputation/test_generate_reputation_dispute_evidence_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected reputation dispute evidence bundle tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/reputation/test_run_reputation_dispute_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected reputation dispute contract lane script tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/reputation/test_run_reputation_dispute_matrix.sh" "$FAST_WORKFLOW"; then
  echo "expected reputation dispute matrix tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_token_launch_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected token launch contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/token/run_token_launch_handoff_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected token launch contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_treasury_disbursement_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected treasury disbursement contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/treasury/run_treasury_disbursement_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected treasury disbursement contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_mainnet_cutover_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected mainnet cutover contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/cutover/run_mainnet_cutover_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected mainnet cutover contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/cutover/run_cutover_rollback_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected cutover rollback contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "if: steps.scope.outputs.run_launch_canary_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected launch canary contract scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/canary/run_launch_canary_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected launch canary contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/canary/run_post_cutover_slo_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected post-cutover SLO contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "steps.scope.outputs.run_channel_lifecycle_contract_tests == 'true' || steps.scope.outputs.run_task_operation_snapshot_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected channel/task lifecycle rust setup/cache condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "steps.scope.outputs.run_federated_delegation_settlement_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected federated delegation settlement rust setup/cache condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "steps.scope.outputs.run_settlement_reconciliation_contract_tests == 'true' || steps.scope.outputs.run_token_launch_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected token launch contract rust setup/cache condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "steps.scope.outputs.run_token_launch_contract_tests == 'true' || steps.scope.outputs.run_treasury_disbursement_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected treasury disbursement contract rust setup/cache condition in ci-fast-gate.yml" >&2
  exit 1
fi

# Regression: #587
if ! grep -Fq "if: steps.scope.outputs.run_bridge_replay_harness == 'true'" "$FAST_WORKFLOW"; then
  echo "expected bridge replay harness scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/bridge/run_bridge_replay_matrix.sh --fixture fixtures/bridge_replay/replay_validation_cases.json --suites \"\${{ steps.scope.outputs.bridge_replay_suites }}\" --output-json bridge-replay-report.json" "$FAST_WORKFLOW"; then
  echo "expected bridge replay harness command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/bridge/run_bridge_credentialed_contract_lane.sh --skip-replay" "$FAST_WORKFLOW"; then
  echo "expected bridge credentialed contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/bridge/run_telegram_ingress_contract_lane.sh --skip-replay" "$FAST_WORKFLOW"; then
  echo "expected telegram ingress contract lane command in ci-fast-gate.yml" >&2
  exit 1
fi

# Regression: #689
if grep -Fq "run_sdk_parity_matrix" "$FAST_WORKFLOW"; then
  echo "fast gate must not execute sdk parity matrix directly" >&2
  exit 1
fi

if grep -Fq "run_live_network_pilot_deep_lane.sh" "$FAST_WORKFLOW"; then
  echo "fast gate must not execute live-network pilot deep lane directly" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/sdk/run_live_transport_parity_deep_lane.sh" "$DEEP_WORKFLOW"; then
  echo "expected scheduled live transport parity deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/guard/run_durable_guard_recovery_deep_lane.sh" "$DEEP_WORKFLOW"; then
  echo "expected scheduled durable guard recovery deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/escrow/run_settlement_reconciliation_deep_lane.sh" "$DEEP_WORKFLOW"; then
  echo "expected scheduled settlement reconciliation deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/compliance/run_soc2_control_evidence_deep_lane.sh --output-json soc2-control-evidence-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled SOC2 control evidence deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "soc2-control-evidence-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected SOC2 control evidence deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/compliance/run_dsar_legal_hold_deep_lane.sh --output-json dsar-legal-hold-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled DSAR legal-hold deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "dsar-legal-hold-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected DSAR legal-hold deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/did/run_federated_did_handshake_deep_lane.sh --output-json federated-did-handshake-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled federated DID handshake deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "federated-did-handshake-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected federated DID handshake deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/task/run_federated_delegation_settlement_deep_lane.sh --output-json federated-delegation-settlement-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled federated delegation settlement deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "federated-delegation-settlement-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected federated delegation settlement deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json kolme-version-compatibility-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled Kolme version compatibility replay deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "kolme-version-compatibility-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected Kolme version compatibility replay report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/governance/run_governance_simulation_deep_lane.sh --output-json governance-simulation-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled governance simulation deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "governance-simulation-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected governance simulation deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/governance/run_stake_slash_risk_deep_lane.sh --output-json governance-stake-slash-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled governance stake/slash deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "governance-stake-slash-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected governance stake/slash deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/reputation/run_weighted_decay_deep_lane.sh --output-json reputation-weighted-decay-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled weighted decay anti-gaming deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "reputation-weighted-decay-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected weighted decay anti-gaming deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/reputation/run_reputation_dispute_deep_lane.sh --output-json reputation-dispute-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled reputation dispute deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "reputation-dispute-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected reputation dispute deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/token/run_token_launch_handoff_deep_lane.sh --output-json token-launch-handoff-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled token launch handoff deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "token-launch-handoff-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected token launch handoff deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/cutover/run_cutover_rollback_deep_lane.sh --output-json cutover-rollback-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled cutover rollback deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "cutover-rollback-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected cutover rollback deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/canary/run_launch_canary_deep_lane.sh --output-json launch-canary-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled launch canary deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "launch-canary-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected launch canary deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/canary/run_post_cutover_slo_deep_lane.sh --output-json post-cutover-slo-report.json" "$DEEP_WORKFLOW"; then
  echo "expected scheduled post-cutover SLO deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "post-cutover-slo-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected post-cutover SLO deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/bridge/run_bridge_credentialed_deep_lane.sh --output-json bridge-credentialed-deep-report.json" "$DEEP_WORKFLOW"; then
  echo "expected bridge credentialed deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bridge-credentialed-deep-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected bridge credentialed deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name \"\${{ github.event_name }}\" --output-json live-network-pilot-report.json" "$DEEP_WORKFLOW"; then
  echo "expected live-network pilot deep lane command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "live-network-pilot-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected live-network pilot deep report artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json sdk-parity-report.json" "$DEEP_WORKFLOW"; then
  echo "expected sdk parity matrix command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "sdk-parity-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected sdk parity matrix artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

echo "workflow scope policy tests passed."

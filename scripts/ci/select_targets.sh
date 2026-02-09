#!/usr/bin/env bash
set -euo pipefail

write_output() {
  local key="$1"
  local value="$2"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "${key}<<EOF"
      echo "$value"
      echo "EOF"
    } >>"$GITHUB_OUTPUT"
  else
    printf '%s=%s\n' "$key" "$value"
  fi
}

append_summary() {
  if [ -z "${GITHUB_STEP_SUMMARY:-}" ]; then
    return
  fi

  {
    echo "### CI Scope Selection"
    echo "- Base reference: ${BASE_REF_DISPLAY}"
    echo "- Changed files: ${CHANGED_COUNT}"
    echo "- Docs only: ${DOCS_ONLY}"
    echo "- Run Rust checks: ${RUN_RUST}"
    echo "- Run CI tool checks: ${RUN_CI_TOOL_CHECKS}"
    echo "- Run deploy preflight checks: ${RUN_DEPLOY_PREFLIGHT_TESTS}"
    echo "- Run frontend dashboard tests: ${RUN_FRONTEND_DASHBOARD_TESTS}"
    echo "- Run dashboard contract tests: ${RUN_DASHBOARD_CONTRACT_TESTS}"
    echo "- Run signer emulator contract tests: ${RUN_SIGNER_EMULATOR_CONTRACT_TESTS}"
    echo "- Run DID registry contract tests: ${RUN_DID_REGISTRY_CONTRACT_TESTS}"
    echo "- Run federated DID handshake contract tests: ${RUN_FEDERATED_DID_HANDSHAKE_CONTRACT_TESTS}"
    echo "- Run Kolme snapshot drift contract tests: ${RUN_KOLME_SNAPSHOT_DRIFT_CONTRACT_TESTS}"
    echo "- Run Kolme version compatibility contract tests: ${RUN_KOLME_VERSION_COMPATIBILITY_CONTRACT_TESTS}"
    echo "- Run Kolme triadic devnet smoke contract tests: ${RUN_KOLME_TRIADIC_DEVNET_SMOKE_CONTRACT_TESTS}"
    echo "- Run federated delegation settlement contract tests: ${RUN_FEDERATED_DELEGATION_SETTLEMENT_CONTRACT_TESTS}"
    echo "- Run runtime snapshot contract tests: ${RUN_RUNTIME_SNAPSHOT_CONTRACT_TESTS}"
    echo "- Run message lifecycle contract tests: ${RUN_MESSAGE_LIFECYCLE_CONTRACT_TESTS}"
    echo "- Run channel lifecycle contract tests: ${RUN_CHANNEL_LIFECYCLE_CONTRACT_TESTS}"
    echo "- Run task operation snapshot contract tests: ${RUN_TASK_OPERATION_SNAPSHOT_CONTRACT_TESTS}"
    echo "- Run durable guard recovery contract tests: ${RUN_DURABLE_GUARD_RECOVERY_CONTRACT_TESTS}"
    echo "- Run settlement reconciliation contract tests: ${RUN_SETTLEMENT_RECONCILIATION_CONTRACT_TESTS}"
    echo "- Run SOC2 control evidence contract tests: ${RUN_SOC2_CONTROL_EVIDENCE_CONTRACT_TESTS}"
    echo "- Run DSAR legal-hold contract tests: ${RUN_DSAR_LEGAL_HOLD_CONTRACT_TESTS}"
    echo "- Run governance simulation contract tests: ${RUN_GOVERNANCE_SIMULATION_CONTRACT_TESTS}"
    echo "- Run governance stake/slash contract tests: ${RUN_GOVERNANCE_STAKE_SLASH_CONTRACT_TESTS}"
    echo "- Run reputation weighted decay contract tests: ${RUN_REPUTATION_DECAY_CONTRACT_TESTS}"
    echo "- Run reputation dispute contract tests: ${RUN_REPUTATION_DISPUTE_CONTRACT_TESTS}"
    echo "- Run token launch contract tests: ${RUN_TOKEN_LAUNCH_CONTRACT_TESTS}"
    echo "- Run treasury disbursement contract tests: ${RUN_TREASURY_DISBURSEMENT_CONTRACT_TESTS}"
    echo "- Run mainnet cutover contract tests: ${RUN_MAINNET_CUTOVER_CONTRACT_TESTS}"
    echo "- Run launch canary contract tests: ${RUN_LAUNCH_CANARY_CONTRACT_TESTS}"
    echo "- Run bridge replay harness: ${RUN_BRIDGE_REPLAY_HARNESS}"
    echo "- Run bridge replay deep lane: ${RUN_BRIDGE_REPLAY_DEEP_LANE}"
    echo "- Run localhost bridge demo evidence deep lane: ${RUN_LOCALHOST_BRIDGE_DEMO_EVIDENCE_DEEP_LANE}"
    echo "- Bridge replay suites: ${BRIDGE_REPLAY_SUITES}"
    echo "- Run Rust live transport contract tests: ${RUN_RUST_LIVE_TRANSPORT_CONTRACT_TESTS}"
    echo "- Run Python live transport contract tests: ${RUN_PYTHON_LIVE_TRANSPORT_CONTRACT_TESTS}"
    echo "- Run TypeScript live transport contract tests: ${RUN_TYPESCRIPT_LIVE_TRANSPORT_CONTRACT_TESTS}"
    echo "- Run cross-language live transport parity contract tests: ${RUN_LIVE_TRANSPORT_PARITY_CONTRACT_TESTS}"
    echo "- Run cross-language live transport parity rust contract tests: ${RUN_LIVE_TRANSPORT_PARITY_RUST_CONTRACT_TESTS}"
    echo "- Run localhost signed integration contract lane tests: ${RUN_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_LANE_TESTS}"
    echo "- Live transport parity languages: ${LIVE_TRANSPORT_PARITY_LANGUAGES}"
    echo "- Run SDK parity matrix: ${RUN_SDK_PARITY_MATRIX}"
    echo "- Run invariant harness: ${RUN_INVARIANT_HARNESS}"
    echo "- Test scope: ${TEST_SCOPE}"
    echo "- Critical path fallback: ${CRITICAL_PATH_CHANGED}"
    echo "- Unknown path fallback: ${UNKNOWN_RISK_CHANGED}"
    if [ -n "$CHANGED_MANIFESTS" ]; then
      echo "- Targeted manifests: ${CHANGED_MANIFESTS}"
    fi
  } >>"$GITHUB_STEP_SUMMARY"
}

find_manifest_for_path() {
  local path="$1"
  local dir

  dir="$(dirname "$path")"

  while [ "$dir" != "." ] && [ "$dir" != "/" ]; do
    if [ -f "$dir/Cargo.toml" ]; then
      printf '%s/Cargo.toml\n' "$dir"
      return 0
    fi
    local parent
    parent="$(dirname "$dir")"
    if [ "$parent" = "$dir" ]; then
      break
    fi
    dir="$parent"
  done

  if [ -f Cargo.toml ]; then
    printf 'Cargo.toml\n'
    return 0
  fi

  return 1
}

join_with_and() {
  local joined=""
  local part
  for part in "$@"; do
    if [ -z "$joined" ]; then
      joined="$part"
    else
      joined+=" && $part"
    fi
  done
  printf '%s\n' "$joined"
}

if git ls-files | grep -Eq '(^|/)Cargo.toml$'; then
  REPO_HAS_RUST=true
else
  REPO_HAS_RUST=false
fi

BASE_REF="${GITHUB_BASE_REF:-main}"
BASE_REF_DISPLAY="$BASE_REF"

if git rev-parse --verify "origin/${BASE_REF}" >/dev/null 2>&1; then
  BASE_COMMIT="$(git merge-base HEAD "origin/${BASE_REF}")"
elif git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
  BASE_COMMIT="$(git rev-parse HEAD~1)"
else
  BASE_COMMIT=""
fi

if [ -n "${BASE_COMMIT}" ]; then
  BASE_REF_DISPLAY="$BASE_COMMIT"
  mapfile -t CHANGED_FILES < <(git diff --name-only "${BASE_COMMIT}...HEAD" | sed '/^$/d')
else
  BASE_REF_DISPLAY="(initial-snapshot)"
  mapfile -t CHANGED_FILES < <(git ls-files | sed '/^$/d')
fi

if [ -n "${CI_CHANGED_FILES:-}" ]; then
  BASE_REF_DISPLAY="(env:CI_CHANGED_FILES)"
  mapfile -t CHANGED_FILES < <(printf '%s\n' "$CI_CHANGED_FILES" | sed '/^$/d')
fi

CHANGED_COUNT="${#CHANGED_FILES[@]}"
DOCS_ONLY=true
RUST_CHANGED=false
CI_INFRA_CHANGED=false
DEPLOY_SCRIPT_CHANGED=false
FRONTEND_DASHBOARD_CHANGED=false
DASHBOARD_CONTRACT_CHANGED=false
SIGNER_EMULATOR_CONTRACT_CHANGED=false
DID_REGISTRY_CONTRACT_CHANGED=false
FEDERATED_DID_HANDSHAKE_CONTRACT_CHANGED=false
KOLME_SNAPSHOT_DRIFT_CONTRACT_CHANGED=false
KOLME_VERSION_COMPATIBILITY_CONTRACT_CHANGED=false
KOLME_TRIADIC_DEVNET_SMOKE_CONTRACT_CHANGED=false
FEDERATED_DELEGATION_SETTLEMENT_CONTRACT_CHANGED=false
RUNTIME_SNAPSHOT_CONTRACT_CHANGED=false
MESSAGE_LIFECYCLE_CONTRACT_CHANGED=false
CHANNEL_LIFECYCLE_CONTRACT_CHANGED=false
TASK_OPERATION_SNAPSHOT_CONTRACT_CHANGED=false
DURABLE_GUARD_RECOVERY_CONTRACT_CHANGED=false
ESCROW_CONTRACT_CHANGED=false
SOC2_CONTROL_EVIDENCE_CONTRACT_CHANGED=false
DSAR_LEGAL_HOLD_CONTRACT_CHANGED=false
GOVERNANCE_SIMULATION_CONTRACT_CHANGED=false
GOVERNANCE_STAKE_SLASH_CONTRACT_CHANGED=false
REPUTATION_DECAY_CONTRACT_CHANGED=false
REPUTATION_DISPUTE_CONTRACT_CHANGED=false
TOKEN_LAUNCH_CONTRACT_CHANGED=false
TREASURY_DISBURSEMENT_CONTRACT_CHANGED=false
MAINNET_CUTOVER_CONTRACT_CHANGED=false
LAUNCH_CANARY_CONTRACT_CHANGED=false
BRIDGE_REPLAY_RELATED_CHANGED=false
BRIDGE_SUITE_ADAPTER=false
BRIDGE_SUITE_TELEGRAM=false
BRIDGE_SUITE_DISCORD=false
BRIDGE_SUITE_CROSS_CHAIN=false
SDK_RUST_LIVE_CHANGED=false
SDK_PYTHON_LIVE_CHANGED=false
SDK_TYPESCRIPT_LIVE_CHANGED=false
SDK_LIVE_PARITY_SCRIPT_CHANGED=false
SDK_SHARED_MATRIX_CHANGED=false
SDK_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_CHANGED=false
CRITICAL_PATH_CHANGED=false
UNKNOWN_RISK_CHANGED=false
FULL_SUITE=false
INVARIANT_RELATED_CHANGED=false

# manifest -> 1
# shellcheck disable=SC2034
declare -A MANIFESTS=()

for file in "${CHANGED_FILES[@]}"; do
  classified=false

  case "$file" in
    *.md|*.txt|docs/*)
      classified=true
      ;;
    *)
      DOCS_ONLY=false
      ;;
  esac

  case "$file" in
    *.rs|Cargo.toml|Cargo.lock|*/Cargo.toml|*/Cargo.lock|rust-toolchain.toml|.cargo/*)
      RUST_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    .github/workflows/*|scripts/ci/*)
      CI_INFRA_CHANGED=true
      CRITICAL_PATH_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    scripts/deploy/*)
      DEPLOY_SCRIPT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    packages/kamn-dashboard/*|scripts/frontend/*|docs/foundation/operator-dashboard-ui-mvp.md|docs/foundation/observability-slo-dashboards.md)
      FRONTEND_DASHBOARD_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/operator-dashboard-backend-apis.md|docs/foundation/operator-dashboard-ui-mvp.md|crates/kamn-core/tests/operator_dashboard_api_docs.rs|crates/kamn-core/tests/operator_dashboard_ui_docs.rs)
      DASHBOARD_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/signer_backend.rs|crates/kamn-core/tests/signer_backend.rs|crates/kamn-core/tests/signer_backend_docs.rs|docs/foundation/signer-backend-abstraction.md|scripts/signer/*)
      SIGNER_EMULATOR_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/did_registry.rs|crates/kamn-core/tests/did_registry_transactions.rs|crates/kamn-core/tests/did_registry_transactions_docs.rs|docs/foundation/did-registry-transactions.md|scripts/did/*did_registry*)
      DID_REGISTRY_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/did-method.md|scripts/did/*federated_did_handshake*|fixtures/federated_did_handshake/*|crates/kamn-core/tests/did_method_docs.rs)
      FEDERATED_DID_HANDSHAKE_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/research/kolme-upstream-compatibility.md|scripts/kolme/check_snapshot_drift.py|scripts/kolme/run_snapshot_drift_contract_lane.sh|scripts/kolme/test_check_snapshot_drift.sh|scripts/kolme/test_run_snapshot_drift_contract_lane.sh|fixtures/kolme_compatibility/snapshot_*)
      KOLME_SNAPSHOT_DRIFT_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/planning/kolme-integration-roadmap.md|docs/foundation/release-gonogo-checklist.md|scripts/kolme/validate_version_compatibility.py|scripts/kolme/run_version_compatibility_replay.py|scripts/kolme/run_version_compatibility_contract_lane.sh|scripts/kolme/run_version_compatibility_replay_deep_lane.sh|scripts/kolme/run_runtime_commit_contract_lane.sh|scripts/kolme/check_runtime_commit_replay_policy.py|scripts/kolme/run_runtime_commit_replay_tamper_matrix.py|scripts/kolme/run_runtime_commit_replay_contract_lane.sh|scripts/kolme/test_validate_version_compatibility.sh|scripts/kolme/test_check_runtime_commit_replay_policy.sh|scripts/kolme/test_run_version_compatibility_contract_lane.sh|scripts/kolme/test_run_runtime_commit_contract_lane.sh|scripts/kolme/test_run_runtime_commit_replay_contract_lane.sh|fixtures/kolme_compatibility/version_compatibility_cases.json|fixtures/kolme_commit/runtime_commit_request_cases.txt|fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json|crates/kamn-core/src/kolme_runtime_commit.rs|crates/kamn-core/tests/kolme_runtime_commit_client.rs|crates/kamn-core/tests/kolme_runtime_commit_finality.rs|crates/kamn-core/tests/kolme_integration_roadmap_docs.rs)
      KOLME_VERSION_COMPATIBILITY_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/planning/kolme-devnet-ops.md|scripts/kolme/validate_triadic_devnet_smoke.py|scripts/kolme/run_triadic_devnet_smoke.sh|scripts/kolme/run_triadic_devnet_smoke_contract_lane.sh|scripts/kolme/test_validate_triadic_devnet_smoke.sh|scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh|fixtures/kolme_compatibility/devnet_smoke_markers.json)
      KOLME_TRIADIC_DEVNET_SMOKE_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/runtime-network.md|docs/foundation/runtime-watchdog-attestation.md|docs/planning/live-network-wave.md|Makefile|crates/kamn-core/tests/runtime_network_docs.rs|crates/kamn-core/tests/runtime_watchdog_attestation_docs.rs|crates/kamn-core/tests/live_network_wave_docs.rs|scripts/runtime/*)
      RUNTIME_SNAPSHOT_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/message-lifecycle.md|crates/kamn-core/tests/message_lifecycle_docs.rs|scripts/message/*)
      MESSAGE_LIFECYCLE_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/channel-models.md|crates/kamn-core/tests/channel_models_docs.rs|scripts/channel/*)
      CHANNEL_LIFECYCLE_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/task-operations.md|crates/kamn-core/tests/task_operations_docs.rs|scripts/task/*task_operation_snapshot*)
      TASK_OPERATION_SNAPSHOT_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/task-operations.md|docs/foundation/release-gonogo-checklist.md|crates/kamn-core/tests/task_operations_docs.rs|crates/kamn-core/tests/release_gonogo_checklist_docs.rs|scripts/task/*federated_delegation_settlement*|fixtures/federated_task_delegation/*)
      FEDERATED_DELEGATION_SETTLEMENT_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/message_delivery_guards.rs|crates/kamn-core/src/channel_policies.rs|crates/kamn-core/src/durable_guard_store.rs|crates/kamn-core/tests/durable_guard_recovery_matrix.rs|crates/kamn-core/tests/durable_guard_snapshot_store.rs|docs/foundation/message-delivery-guards.md|docs/foundation/channel-permissions-retention.md|docs/foundation/release-gonogo-checklist.md|crates/kamn-core/tests/release_gonogo_checklist_docs.rs|scripts/guard/*)
      DURABLE_GUARD_RECOVERY_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/escrow.rs|crates/kamn-core/tests/escrow_lifecycle.rs|crates/kamn-core/tests/escrow_lifecycle_docs.rs|crates/kamn-core/tests/audit_export_interfaces_docs.rs|docs/foundation/escrow-lifecycle.md|docs/foundation/audit-export-interfaces.md|docs/foundation/release-gonogo-checklist.md|scripts/escrow/*|fixtures/escrow_reconciliation/*)
      ESCROW_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/audit-export-interfaces.md|docs/foundation/release-gonogo-checklist.md|crates/kamn-core/tests/audit_export_interfaces_docs.rs|crates/kamn-core/tests/release_gonogo_checklist_docs.rs|scripts/compliance/*soc2*|fixtures/compliance_soc2/*)
      SOC2_CONTROL_EVIDENCE_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/data-classification-tagging.md|docs/foundation/audit-export-interfaces.md|docs/foundation/release-gonogo-checklist.md|crates/kamn-core/tests/audit_export_interfaces_docs.rs|crates/kamn-core/tests/release_gonogo_checklist_docs.rs|crates/kamn-core/tests/data_classification_tagging_docs.rs|scripts/compliance/*dsar*|fixtures/compliance_dsar/*)
      DSAR_LEGAL_HOLD_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/governance-proposal-vote-execution.md|docs/foundation/release-gonogo-checklist.md|crates/kamn-core/src/governance_workflow.rs|crates/kamn-core/tests/governance_workflow.rs|crates/kamn-core/tests/governance_workflow_docs.rs|crates/kamn-core/tests/release_gonogo_checklist_docs.rs|scripts/governance/*governance_simulation*|fixtures/governance_simulation/*)
      GOVERNANCE_SIMULATION_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/governance-proposal-vote-execution.md|docs/foundation/release-gonogo-checklist.md|docs/foundation/validator-lifecycle-quorum-reconfiguration.md|crates/kamn-core/tests/governance_workflow_docs.rs|crates/kamn-core/tests/release_gonogo_checklist_docs.rs|crates/kamn-core/tests/validator_lifecycle_docs.rs|scripts/governance/*stake_slash*|fixtures/governance_stake_slash/*)
      GOVERNANCE_STAKE_SLASH_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/reputation-state-model.md|docs/foundation/trust-score-engine.md|crates/kamn-core/src/trust_score.rs|crates/kamn-core/tests/trust_score_engine.rs|crates/kamn-core/tests/trust_score_engine_docs.rs|crates/kamn-core/tests/reputation_state_model_docs.rs|scripts/reputation/*weighted_decay*|fixtures/reputation_decay/*)
      REPUTATION_DECAY_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/reputation-state-model.md|docs/foundation/reputation-signal-routing.md|docs/foundation/audit-export-interfaces.md|docs/foundation/release-gonogo-checklist.md|crates/kamn-core/tests/reputation_state_model_docs.rs|crates/kamn-core/tests/reputation_signal_routing_docs.rs|crates/kamn-core/tests/audit_export_interfaces_docs.rs|crates/kamn-core/tests/release_gonogo_checklist_docs.rs|scripts/reputation/*|fixtures/reputation_dispute/*)
      REPUTATION_DISPUTE_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/token.rs|crates/kamn-core/tests/token_config.rs|crates/kamn-core/tests/token_config_docs.rs|docs/foundation/token-config.md|docs/foundation/token-model.md|docs/foundation/release-gonogo-checklist.md|scripts/token/*|fixtures/token_launch/*)
      TOKEN_LAUNCH_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/tests/release_gonogo_checklist_docs.rs|docs/foundation/release-gonogo-checklist.md|scripts/treasury/*|fixtures/treasury_disbursement/*)
      TREASURY_DISBURSEMENT_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/mainnet-cutover-runbook.md|crates/kamn-core/tests/mainnet_cutover_runbook_docs.rs|scripts/cutover/*|fixtures/mainnet_cutover/*)
      MAINNET_CUTOVER_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    docs/foundation/release-gonogo-checklist.md|docs/foundation/observability-slo-dashboards.md|crates/kamn-core/tests/release_gonogo_checklist_docs.rs|scripts/canary/*|fixtures/launch_canary/*)
      LAUNCH_CANARY_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/bridge_adapter.rs|crates/kamn-core/tests/bridge_adapter.rs|crates/kamn-core/src/cross_chain_receipt.rs|crates/kamn-core/tests/cross_chain_receipt_finality.rs|crates/kamn-core/tests/cross_chain_bridge_adapters_docs.rs|docs/foundation/bridge-adapter-abstraction.md|docs/foundation/cross-chain-bridge-adapters.md|docs/foundation/bridge-quorum-runtime.md|scripts/bridge/*|fixtures/bridge_replay/*|fixtures/bridge_outbound_intent/*)
      BRIDGE_REPLAY_RELATED_CHANGED=true
      BRIDGE_SUITE_ADAPTER=true
      BRIDGE_SUITE_TELEGRAM=true
      BRIDGE_SUITE_DISCORD=true
      BRIDGE_SUITE_CROSS_CHAIN=true
      classified=true
      ;;
    crates/kamn-core/src/telegram_bridge.rs|crates/kamn-core/tests/telegram_bridge.rs)
      BRIDGE_REPLAY_RELATED_CHANGED=true
      BRIDGE_SUITE_ADAPTER=true
      BRIDGE_SUITE_TELEGRAM=true
      classified=true
      ;;
    crates/kamn-core/src/discord_bridge.rs|crates/kamn-core/tests/discord_bridge.rs)
      BRIDGE_REPLAY_RELATED_CHANGED=true
      BRIDGE_SUITE_ADAPTER=true
      BRIDGE_SUITE_DISCORD=true
      classified=true
      ;;
    crates/kamn-core/src/cross_chain_bridge.rs|crates/kamn-core/tests/cross_chain_bridge.rs)
      BRIDGE_REPLAY_RELATED_CHANGED=true
      BRIDGE_SUITE_ADAPTER=true
      BRIDGE_SUITE_CROSS_CHAIN=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-sdk/*|docs/foundation/rust-sdk-alpha.md|scripts/sdk/run_rust_live_transport_contract_lane.sh|scripts/sdk/run_rust_live_transport_deep_lane.sh|scripts/sdk/test_run_rust_live_transport_contract_lane.sh|scripts/sdk/run_local_e2e_demo.sh|scripts/sdk/test_run_local_e2e_demo.sh|scripts/sdk/run_localhost_signed_demo.sh|scripts/sdk/test_run_localhost_signed_demo.sh|scripts/sdk/run_tcp_signed_relay_demo.sh|scripts/sdk/test_run_tcp_signed_relay_demo.sh|scripts/sdk/run_tcp_failover_reconnect_matrix.sh|scripts/sdk/run_tcp_failover_reconnect_matrix.py|scripts/sdk/test_run_tcp_failover_reconnect_matrix.sh|fixtures/sdk_failover_reconnect/*)
      SDK_RUST_LIVE_CHANGED=true
      classified=true
      ;;
    kamn_sdk.py|tests/python/test_sdk.py|tests/python/test_sdk_live_transport_deep.py|docs/foundation/python-sdk-beta.md)
      SDK_PYTHON_LIVE_CHANGED=true
      classified=true
      ;;
    packages/kamn-sdk/*|docs/foundation/typescript-sdk-beta.md)
      SDK_TYPESCRIPT_LIVE_CHANGED=true
      classified=true
      ;;
    scripts/sdk/run_live_transport_parity_contract_lane.sh|scripts/sdk/run_live_transport_parity_deep_lane.sh|scripts/sdk/test_run_live_transport_parity_contract_lane.sh|scripts/sdk/run_transport_profile_parity_matrix.py|scripts/sdk/run_transport_profile_parity_matrix.sh|scripts/sdk/test_run_transport_profile_parity_matrix.sh|scripts/sdk/transport_profile_probe.py|scripts/sdk/transport_profile_probe.ts|scripts/sdk/run_transport_profile_probe_python.sh|scripts/sdk/run_transport_profile_probe_typescript.sh|scripts/sdk/run_transport_profile_probe_rust.sh)
      SDK_LIVE_PARITY_SCRIPT_CHANGED=true
      classified=true
      ;;
    fixtures/sdk_parity/*|scripts/sdk/run_sdk_parity_matrix.py|scripts/sdk/run_sdk_parity_matrix.sh|scripts/sdk/test_run_sdk_parity_matrix.sh|scripts/sdk/run_parity_python.sh|scripts/sdk/run_parity_rust.sh|scripts/sdk/run_parity_typescript.sh|scripts/sdk/register_case_runner.py|scripts/sdk/register_case_runner.ts)
      SDK_SHARED_MATRIX_CHANGED=true
      classified=true
      ;;
    scripts/sdk/run_localhost_signed_integration_harness.sh|scripts/sdk/test_run_localhost_signed_integration_harness.sh|scripts/sdk/check_localhost_signed_integration_evidence_policy.sh|scripts/sdk/test_check_localhost_signed_integration_evidence_policy.sh|scripts/sdk/run_localhost_signed_integration_contract_lane.sh|scripts/sdk/test_run_localhost_signed_integration_contract_lane.sh)
      SDK_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/invariants.rs|crates/kamn-core/src/transaction.rs|crates/kamn-core/src/smoke.rs|crates/kamn-core/tests/invariant_*|crates/kamn-core/tests/transaction_guards.rs|docs/foundation/invariants.md|docs/foundation/transaction-guards.md|scripts/ci/run_invariant_harness.sh|scripts/ci/test_run_invariant_harness.sh)
      INVARIANT_RELATED_CHANGED=true
      ;;
  esac

  case "$file" in
    Cargo.toml|Cargo.lock|rust-toolchain.toml|.cargo/*)
      FULL_SUITE=true
      ;;
  esac

  if [ "$DOCS_ONLY" = false ] && [ "$classified" = false ]; then
    UNKNOWN_RISK_CHANGED=true
  fi

  if manifest="$(find_manifest_for_path "$file" 2>/dev/null)"; then
    MANIFESTS["$manifest"]=1
  fi
done

if [ "$CRITICAL_PATH_CHANGED" = true ] || [ "$UNKNOWN_RISK_CHANGED" = true ]; then
  FULL_SUITE=true
fi

RUN_RUST=false
RUN_CI_TOOL_CHECKS=false
RUN_DEPLOY_PREFLIGHT_TESTS=false
RUN_FRONTEND_DASHBOARD_TESTS=false
RUN_DASHBOARD_CONTRACT_TESTS=false
RUN_SIGNER_EMULATOR_CONTRACT_TESTS=false
RUN_DID_REGISTRY_CONTRACT_TESTS=false
RUN_FEDERATED_DID_HANDSHAKE_CONTRACT_TESTS=false
RUN_KOLME_SNAPSHOT_DRIFT_CONTRACT_TESTS=false
RUN_KOLME_VERSION_COMPATIBILITY_CONTRACT_TESTS=false
RUN_KOLME_TRIADIC_DEVNET_SMOKE_CONTRACT_TESTS=false
RUN_FEDERATED_DELEGATION_SETTLEMENT_CONTRACT_TESTS=false
RUN_RUNTIME_SNAPSHOT_CONTRACT_TESTS=false
RUN_MESSAGE_LIFECYCLE_CONTRACT_TESTS=false
RUN_CHANNEL_LIFECYCLE_CONTRACT_TESTS=false
RUN_TASK_OPERATION_SNAPSHOT_CONTRACT_TESTS=false
RUN_DURABLE_GUARD_RECOVERY_CONTRACT_TESTS=false
RUN_SETTLEMENT_RECONCILIATION_CONTRACT_TESTS=false
RUN_SOC2_CONTROL_EVIDENCE_CONTRACT_TESTS=false
RUN_DSAR_LEGAL_HOLD_CONTRACT_TESTS=false
RUN_GOVERNANCE_SIMULATION_CONTRACT_TESTS=false
RUN_GOVERNANCE_STAKE_SLASH_CONTRACT_TESTS=false
RUN_REPUTATION_DECAY_CONTRACT_TESTS=false
RUN_REPUTATION_DISPUTE_CONTRACT_TESTS=false
RUN_TOKEN_LAUNCH_CONTRACT_TESTS=false
RUN_TREASURY_DISBURSEMENT_CONTRACT_TESTS=false
RUN_MAINNET_CUTOVER_CONTRACT_TESTS=false
RUN_LAUNCH_CANARY_CONTRACT_TESTS=false
RUN_BRIDGE_REPLAY_HARNESS=false
RUN_BRIDGE_REPLAY_DEEP_LANE=false
RUN_LOCALHOST_BRIDGE_DEMO_EVIDENCE_DEEP_LANE=false
RUN_RUST_LIVE_TRANSPORT_CONTRACT_TESTS=false
RUN_PYTHON_LIVE_TRANSPORT_CONTRACT_TESTS=false
RUN_TYPESCRIPT_LIVE_TRANSPORT_CONTRACT_TESTS=false
RUN_LIVE_TRANSPORT_PARITY_CONTRACT_TESTS=false
RUN_LIVE_TRANSPORT_PARITY_RUST_CONTRACT_TESTS=false
RUN_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_LANE_TESTS=false
LIVE_TRANSPORT_PARITY_LANGUAGES=""
RUN_SDK_PARITY_MATRIX=false
FMT_CMD=":"
CLIPPY_CMD=":"
TEST_CMD=":"
TEST_SCOPE="none"
CHANGED_MANIFESTS=""
RUN_INVARIANT_HARNESS=false
BRIDGE_REPLAY_SUITES=""

if [ "$REPO_HAS_RUST" = true ] && { [ "$RUST_CHANGED" = true ] || [ "$CI_INFRA_CHANGED" = true ] || [ "$FULL_SUITE" = true ]; }; then
  RUN_RUST=true

  mapfile -t manifest_list < <(printf '%s\n' "${!MANIFESTS[@]}" | sed '/^$/d' | sort)

  if [ "${#manifest_list[@]}" -gt 0 ]; then
    CHANGED_MANIFESTS="$(printf '%s, ' "${manifest_list[@]}" | sed 's/, $//')"
  fi

  FMT_CMD='cargo fmt --all --check'

  if [ "$FULL_SUITE" = true ]; then
    TEST_SCOPE="full"
    CLIPPY_CMD='cargo clippy --workspace --all-targets --all-features -- -D warnings'
    TEST_CMD='bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test --workspace --locked --all-features --no-fail-fast'
  elif [ "${#manifest_list[@]}" -gt 1 ] || { [ "${#manifest_list[@]}" -eq 1 ] && [ "${manifest_list[0]}" != "Cargo.toml" ]; }; then
    TEST_SCOPE="targeted"

    clippy_parts=()
    test_parts=()
    for manifest in "${manifest_list[@]}"; do
      clippy_parts+=("cargo clippy --all-targets --all-features --manifest-path '$manifest' -- -D warnings")
      test_parts+=("bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test --locked --all-features --manifest-path '$manifest' --no-fail-fast")
    done

    CLIPPY_CMD="$(join_with_and "${clippy_parts[@]}")"
    TEST_CMD="$(join_with_and "${test_parts[@]}")"
  else
    TEST_SCOPE="smoke"
    CLIPPY_CMD='cargo clippy --workspace --all-targets -- -D warnings'
    TEST_CMD='bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test --workspace --locked --all-features --lib --no-fail-fast'
  fi
fi

if [ "$RUN_RUST" = true ] && { [ "$INVARIANT_RELATED_CHANGED" = true ] || [ "$FULL_SUITE" = true ]; }; then
  RUN_INVARIANT_HARNESS=true
fi

if [ "$RUN_RUST" != true ] && [ "$DEPLOY_SCRIPT_CHANGED" = true ]; then
  RUN_DEPLOY_PREFLIGHT_TESTS=true
  TEST_SCOPE="deploy"
fi

if [ "$FRONTEND_DASHBOARD_CHANGED" = true ]; then
  RUN_FRONTEND_DASHBOARD_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="frontend"
  fi
fi

if [ "$DASHBOARD_CONTRACT_CHANGED" = true ]; then
  RUN_DASHBOARD_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="frontend-contract"
  fi
fi

if [ "$SIGNER_EMULATOR_CONTRACT_CHANGED" = true ]; then
  RUN_SIGNER_EMULATOR_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="signer-contract"
  fi
fi

if [ "$DID_REGISTRY_CONTRACT_CHANGED" = true ]; then
  RUN_DID_REGISTRY_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="did-contract"
  fi
fi

if [ "$FEDERATED_DID_HANDSHAKE_CONTRACT_CHANGED" = true ]; then
  RUN_FEDERATED_DID_HANDSHAKE_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="federated-did-contract"
  fi
fi

if [ "$KOLME_SNAPSHOT_DRIFT_CONTRACT_CHANGED" = true ]; then
  RUN_KOLME_SNAPSHOT_DRIFT_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="kolme-contract"
  fi
fi

if [ "$RUNTIME_SNAPSHOT_CONTRACT_CHANGED" = true ]; then
  RUN_RUNTIME_SNAPSHOT_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="runtime-contract"
  fi
fi

if [ "$MESSAGE_LIFECYCLE_CONTRACT_CHANGED" = true ]; then
  RUN_MESSAGE_LIFECYCLE_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="message-contract"
  fi
fi

if [ "$CHANNEL_LIFECYCLE_CONTRACT_CHANGED" = true ]; then
  RUN_CHANNEL_LIFECYCLE_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="channel-contract"
  fi
fi

if [ "$TASK_OPERATION_SNAPSHOT_CONTRACT_CHANGED" = true ]; then
  RUN_TASK_OPERATION_SNAPSHOT_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="task-contract"
  fi
fi

if [ "$DURABLE_GUARD_RECOVERY_CONTRACT_CHANGED" = true ]; then
  RUN_DURABLE_GUARD_RECOVERY_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="guard-contract"
  fi
fi

if [ "$FEDERATED_DELEGATION_SETTLEMENT_CONTRACT_CHANGED" = true ]; then
  RUN_FEDERATED_DELEGATION_SETTLEMENT_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="federated-task-contract"
  fi
fi

if [ "$KOLME_VERSION_COMPATIBILITY_CONTRACT_CHANGED" = true ]; then
  RUN_KOLME_VERSION_COMPATIBILITY_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="kolme-version-contract"
  fi
fi

if [ "$KOLME_TRIADIC_DEVNET_SMOKE_CONTRACT_CHANGED" = true ]; then
  RUN_KOLME_TRIADIC_DEVNET_SMOKE_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="kolme-devnet-contract"
  fi
fi

if [ "$ESCROW_CONTRACT_CHANGED" = true ]; then
  RUN_SETTLEMENT_RECONCILIATION_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="escrow-contract"
  fi
fi

if [ "$SOC2_CONTROL_EVIDENCE_CONTRACT_CHANGED" = true ]; then
  RUN_SOC2_CONTROL_EVIDENCE_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="soc2-contract"
  fi
fi

if [ "$DSAR_LEGAL_HOLD_CONTRACT_CHANGED" = true ]; then
  RUN_DSAR_LEGAL_HOLD_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="dsar-contract"
  fi
fi

if [ "$GOVERNANCE_SIMULATION_CONTRACT_CHANGED" = true ]; then
  RUN_GOVERNANCE_SIMULATION_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="governance-contract"
  fi
fi

if [ "$GOVERNANCE_STAKE_SLASH_CONTRACT_CHANGED" = true ]; then
  RUN_GOVERNANCE_STAKE_SLASH_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="governance-risk-contract"
  fi
fi

if [ "$REPUTATION_DECAY_CONTRACT_CHANGED" = true ]; then
  RUN_REPUTATION_DECAY_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="reputation-decay-contract"
  fi
fi

if [ "$REPUTATION_DISPUTE_CONTRACT_CHANGED" = true ]; then
  RUN_REPUTATION_DISPUTE_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="reputation-contract"
  fi
fi

if [ "$TOKEN_LAUNCH_CONTRACT_CHANGED" = true ]; then
  RUN_TOKEN_LAUNCH_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="token-contract"
  fi
fi

if [ "$TREASURY_DISBURSEMENT_CONTRACT_CHANGED" = true ]; then
  RUN_TREASURY_DISBURSEMENT_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="treasury-contract"
  fi
fi

if [ "$MAINNET_CUTOVER_CONTRACT_CHANGED" = true ]; then
  RUN_MAINNET_CUTOVER_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="cutover-contract"
  fi
fi

if [ "$LAUNCH_CANARY_CONTRACT_CHANGED" = true ]; then
  RUN_LAUNCH_CANARY_CONTRACT_TESTS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="canary-contract"
  fi
fi

if [ "$BRIDGE_REPLAY_RELATED_CHANGED" = true ]; then
  RUN_BRIDGE_REPLAY_HARNESS=true
  bridge_suite_parts=()
  if [ "$BRIDGE_SUITE_ADAPTER" = true ]; then
    bridge_suite_parts+=("bridge_adapter")
  fi
  if [ "$BRIDGE_SUITE_TELEGRAM" = true ]; then
    bridge_suite_parts+=("telegram_bridge")
  fi
  if [ "$BRIDGE_SUITE_DISCORD" = true ]; then
    bridge_suite_parts+=("discord_bridge")
  fi
  if [ "$BRIDGE_SUITE_CROSS_CHAIN" = true ]; then
    bridge_suite_parts+=("cross_chain_bridge")
  fi
  if [ "${#bridge_suite_parts[@]}" -eq 0 ]; then
    bridge_suite_parts=(
      "bridge_adapter"
      "telegram_bridge"
      "discord_bridge"
      "cross_chain_bridge"
    )
  fi
  BRIDGE_REPLAY_SUITES="$(printf '%s,' "${bridge_suite_parts[@]}" | sed 's/,$//')"
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="bridge"
  fi
fi

if [ "$BRIDGE_REPLAY_RELATED_CHANGED" = true ] && [ "${CI_ENABLE_BRIDGE_REPLAY_DEEP_LANE:-false}" = "true" ]; then
  RUN_BRIDGE_REPLAY_DEEP_LANE=true
  RUN_LOCALHOST_BRIDGE_DEMO_EVIDENCE_DEEP_LANE=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "bridge" ]; then
    TEST_SCOPE="bridge-deep"
  fi
fi

sdk_live_lane_count=0
sdk_live_languages=()
if [ "$SDK_RUST_LIVE_CHANGED" = true ]; then
  sdk_live_lane_count=$((sdk_live_lane_count + 1))
  sdk_live_languages+=("rust")
fi
if [ "$SDK_PYTHON_LIVE_CHANGED" = true ]; then
  sdk_live_lane_count=$((sdk_live_lane_count + 1))
  sdk_live_languages+=("python")
fi
if [ "$SDK_TYPESCRIPT_LIVE_CHANGED" = true ]; then
  sdk_live_lane_count=$((sdk_live_lane_count + 1))
  sdk_live_languages+=("typescript")
fi

if [ "$SDK_SHARED_MATRIX_CHANGED" = true ]; then
  RUN_SDK_PARITY_MATRIX=true
fi

if [ "$SDK_LIVE_PARITY_SCRIPT_CHANGED" = true ]; then
  RUN_LIVE_TRANSPORT_PARITY_CONTRACT_TESTS=true
  LIVE_TRANSPORT_PARITY_LANGUAGES="rust,python,typescript"
elif [ "$sdk_live_lane_count" -gt 1 ]; then
  RUN_LIVE_TRANSPORT_PARITY_CONTRACT_TESTS=true
  LIVE_TRANSPORT_PARITY_LANGUAGES="$(printf '%s,' "${sdk_live_languages[@]}" | sed 's/,$//')"
elif [ "$SDK_RUST_LIVE_CHANGED" = true ]; then
  RUN_RUST_LIVE_TRANSPORT_CONTRACT_TESTS=true
elif [ "$SDK_PYTHON_LIVE_CHANGED" = true ]; then
  RUN_PYTHON_LIVE_TRANSPORT_CONTRACT_TESTS=true
elif [ "$SDK_TYPESCRIPT_LIVE_CHANGED" = true ]; then
  RUN_TYPESCRIPT_LIVE_TRANSPORT_CONTRACT_TESTS=true
fi

if [ "$RUN_LIVE_TRANSPORT_PARITY_CONTRACT_TESTS" = true ] && [[ ",$LIVE_TRANSPORT_PARITY_LANGUAGES," == *,rust,* ]]; then
  RUN_LIVE_TRANSPORT_PARITY_RUST_CONTRACT_TESTS=true
fi

if [ "$SDK_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_CHANGED" = true ]; then
  RUN_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_LANE_TESTS=true
fi

if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
  if [ "$RUN_LIVE_TRANSPORT_PARITY_CONTRACT_TESTS" = true ]; then
    TEST_SCOPE="sdk-live-parity"
  elif [ "$RUN_RUST_LIVE_TRANSPORT_CONTRACT_TESTS" = true ]; then
    TEST_SCOPE="sdk-live-rust"
  elif [ "$RUN_PYTHON_LIVE_TRANSPORT_CONTRACT_TESTS" = true ]; then
    TEST_SCOPE="sdk-live-python"
  elif [ "$RUN_TYPESCRIPT_LIVE_TRANSPORT_CONTRACT_TESTS" = true ]; then
    TEST_SCOPE="sdk-live-typescript"
  elif [ "$RUN_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_LANE_TESTS" = true ]; then
    TEST_SCOPE="sdk-live-localhost-integration"
  elif [ "$RUN_SDK_PARITY_MATRIX" = true ]; then
    TEST_SCOPE="sdk"
  fi
fi

if [ "$CI_INFRA_CHANGED" = true ]; then
  RUN_CI_TOOL_CHECKS=true
fi

write_output "docs_only" "$DOCS_ONLY"
write_output "run_rust" "$RUN_RUST"
write_output "run_ci_tool_checks" "$RUN_CI_TOOL_CHECKS"
write_output "run_deploy_preflight_tests" "$RUN_DEPLOY_PREFLIGHT_TESTS"
write_output "run_frontend_dashboard_tests" "$RUN_FRONTEND_DASHBOARD_TESTS"
write_output "run_dashboard_contract_tests" "$RUN_DASHBOARD_CONTRACT_TESTS"
write_output "run_signer_emulator_contract_tests" "$RUN_SIGNER_EMULATOR_CONTRACT_TESTS"
write_output "run_did_registry_contract_tests" "$RUN_DID_REGISTRY_CONTRACT_TESTS"
write_output "run_federated_did_handshake_contract_tests" "$RUN_FEDERATED_DID_HANDSHAKE_CONTRACT_TESTS"
write_output "run_kolme_snapshot_drift_contract_tests" "$RUN_KOLME_SNAPSHOT_DRIFT_CONTRACT_TESTS"
write_output "run_kolme_version_compatibility_contract_tests" "$RUN_KOLME_VERSION_COMPATIBILITY_CONTRACT_TESTS"
write_output "run_kolme_triadic_devnet_smoke_contract_tests" "$RUN_KOLME_TRIADIC_DEVNET_SMOKE_CONTRACT_TESTS"
write_output "run_federated_delegation_settlement_contract_tests" "$RUN_FEDERATED_DELEGATION_SETTLEMENT_CONTRACT_TESTS"
write_output "run_runtime_snapshot_contract_tests" "$RUN_RUNTIME_SNAPSHOT_CONTRACT_TESTS"
write_output "run_message_lifecycle_contract_tests" "$RUN_MESSAGE_LIFECYCLE_CONTRACT_TESTS"
write_output "run_channel_lifecycle_contract_tests" "$RUN_CHANNEL_LIFECYCLE_CONTRACT_TESTS"
write_output "run_task_operation_snapshot_contract_tests" "$RUN_TASK_OPERATION_SNAPSHOT_CONTRACT_TESTS"
write_output "run_durable_guard_recovery_contract_tests" "$RUN_DURABLE_GUARD_RECOVERY_CONTRACT_TESTS"
write_output "run_settlement_reconciliation_contract_tests" "$RUN_SETTLEMENT_RECONCILIATION_CONTRACT_TESTS"
write_output "run_soc2_control_evidence_contract_tests" "$RUN_SOC2_CONTROL_EVIDENCE_CONTRACT_TESTS"
write_output "run_dsar_legal_hold_contract_tests" "$RUN_DSAR_LEGAL_HOLD_CONTRACT_TESTS"
write_output "run_governance_simulation_contract_tests" "$RUN_GOVERNANCE_SIMULATION_CONTRACT_TESTS"
write_output "run_governance_stake_slash_contract_tests" "$RUN_GOVERNANCE_STAKE_SLASH_CONTRACT_TESTS"
write_output "run_reputation_decay_contract_tests" "$RUN_REPUTATION_DECAY_CONTRACT_TESTS"
write_output "run_reputation_dispute_contract_tests" "$RUN_REPUTATION_DISPUTE_CONTRACT_TESTS"
write_output "run_token_launch_contract_tests" "$RUN_TOKEN_LAUNCH_CONTRACT_TESTS"
write_output "run_treasury_disbursement_contract_tests" "$RUN_TREASURY_DISBURSEMENT_CONTRACT_TESTS"
write_output "run_mainnet_cutover_contract_tests" "$RUN_MAINNET_CUTOVER_CONTRACT_TESTS"
write_output "run_launch_canary_contract_tests" "$RUN_LAUNCH_CANARY_CONTRACT_TESTS"
write_output "run_bridge_replay_harness" "$RUN_BRIDGE_REPLAY_HARNESS"
write_output "run_bridge_replay_deep_lane" "$RUN_BRIDGE_REPLAY_DEEP_LANE"
write_output "run_localhost_bridge_demo_evidence_deep_lane" "$RUN_LOCALHOST_BRIDGE_DEMO_EVIDENCE_DEEP_LANE"
write_output "bridge_replay_suites" "$BRIDGE_REPLAY_SUITES"
write_output "run_rust_live_transport_contract_tests" "$RUN_RUST_LIVE_TRANSPORT_CONTRACT_TESTS"
write_output "run_python_live_transport_contract_tests" "$RUN_PYTHON_LIVE_TRANSPORT_CONTRACT_TESTS"
write_output "run_typescript_live_transport_contract_tests" "$RUN_TYPESCRIPT_LIVE_TRANSPORT_CONTRACT_TESTS"
write_output "run_live_transport_parity_contract_tests" "$RUN_LIVE_TRANSPORT_PARITY_CONTRACT_TESTS"
write_output "run_live_transport_parity_rust_contract_tests" "$RUN_LIVE_TRANSPORT_PARITY_RUST_CONTRACT_TESTS"
write_output "run_localhost_signed_integration_contract_lane_tests" "$RUN_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_LANE_TESTS"
write_output "live_transport_parity_languages" "$LIVE_TRANSPORT_PARITY_LANGUAGES"
write_output "run_sdk_parity_matrix" "$RUN_SDK_PARITY_MATRIX"
write_output "run_invariant_harness" "$RUN_INVARIANT_HARNESS"
write_output "test_scope" "$TEST_SCOPE"
write_output "critical_path_changed" "$CRITICAL_PATH_CHANGED"
write_output "unknown_risk_changed" "$UNKNOWN_RISK_CHANGED"
write_output "changed_files" "$CHANGED_COUNT"
write_output "changed_manifests" "$CHANGED_MANIFESTS"
write_output "fmt_cmd" "$FMT_CMD"
write_output "clippy_cmd" "$CLIPPY_CMD"
write_output "test_cmd" "$TEST_CMD"

append_summary

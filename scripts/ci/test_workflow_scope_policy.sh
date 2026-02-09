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

if ! grep -Fq "steps.scope.outputs.run_channel_lifecycle_contract_tests == 'true' || steps.scope.outputs.run_task_operation_snapshot_contract_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected channel/task lifecycle rust setup/cache condition in ci-fast-gate.yml" >&2
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

if ! grep -Fq "bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json sdk-parity-report.json" "$DEEP_WORKFLOW"; then
  echo "expected sdk parity matrix command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -Fq "sdk-parity-report-\${{ github.run_id }}-\${{ github.run_attempt }}" "$DEEP_WORKFLOW"; then
  echo "expected sdk parity matrix artifact upload in ci-deep-validate.yml" >&2
  exit 1
fi

echo "workflow scope policy tests passed."

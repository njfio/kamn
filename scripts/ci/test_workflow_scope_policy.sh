#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"

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

if ! grep -Fq "bash scripts/sdk/run_live_transport_parity_contract_lane.sh" "$FAST_WORKFLOW"; then
  echo "expected live transport parity lane command in ci-fast-gate.yml" >&2
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

# Regression: #583
if ! grep -Fq "if: steps.scope.outputs.run_sdk_parity_matrix == 'true'" "$FAST_WORKFLOW"; then
  echo "expected sdk parity matrix scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json sdk-parity-report.json" "$FAST_WORKFLOW"; then
  echo "expected sdk parity matrix command in ci-fast-gate.yml" >&2
  exit 1
fi

echo "workflow scope policy tests passed."

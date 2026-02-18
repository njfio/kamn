#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
LEGACY_LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_test_harness_loc_soft_budget_contract_lane.sh"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
SHARED_IMPL="$ROOT_DIR/scripts/ci/test_harness_loc_soft_budget_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/ci_test_harness_loc_soft_budget_contract_lane.json"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ -e "$LEGACY_LANE_SCRIPT" ]; then
  echo "expected superseded generic soft-budget wrapper to be deleted: $LEGACY_LANE_SCRIPT" >&2
  exit 1
fi

test_harness_require_executable "$MANIFEST_RUNNER" "expected manifest lane runner to be executable"

test_harness_require_executable "$SHARED_IMPL" "expected generic soft-budget shared impl script to be executable"

test_harness_require_file "$STRATEGY_DOC" "expected CI strategy doc to exist"

test_harness_require_file "$COST_DOC" "expected CI cost/lane framework doc to exist"

REPORT_FILE="$TMP_DIR/test-harness-soft-budget-contract-report.json"

lane_output="$(
  bash "$MANIFEST_RUNNER" \
    --manifest "$MANIFEST_FILE" \
    --phase contract \
    -- \
    --output-json "$REPORT_FILE" \
    --max-runtime-seconds 120
)"

if ! printf '%s\n' "$lane_output" | grep -q '^test_harness_loc_soft_budget_contract_status=pass$'; then
  echo "expected contract lane pass marker" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^test_harness_loc_soft_budget_contract_within_decision=GO$'; then
  echo "expected GO decision marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^test_harness_loc_soft_budget_contract_exceeded_decision=WARN$'; then
  echo "expected WARN decision marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^test_harness_loc_soft_budget_contract_warn_decision=WARN$'; then
  echo "expected trend WARN decision marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^test_harness_loc_soft_budget_contract_fail_decision=NO-GO$'; then
  echo "expected trend NO-GO decision marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^test_harness_loc_soft_budget_contract_ci_smoke_lane_cost_profile=low$'; then
  echo "expected low-cost CI smoke lane marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^test_harness_loc_soft_budget_contract_ci_smoke_runtime_budget_status=within$'; then
  echo "expected bounded CI smoke runtime budget marker from contract lane" >&2
  exit 1
fi

test_harness_require_file "$REPORT_FILE" "expected generic soft-budget contract report to be emitted"

if ! grep -q '"schema_version": "kamn.ci.test-harness-loc-soft-budget-contract-report.v1"' "$REPORT_FILE"; then
  echo "expected contract report schema marker" >&2
  exit 1
fi

if ! grep -q '"combined_reason_code_contract": "pass"' "$REPORT_FILE"; then
  echo "expected combined reason-code contract to pass" >&2
  exit 1
fi

if ! grep -q '"trend_reason_code_contract": "pass"' "$REPORT_FILE"; then
  echo "expected trend reason-code contract to pass" >&2
  exit 1
fi

if ! grep -q '"ci_smoke_lane_cost_profile": "low"' "$REPORT_FILE"; then
  echo "expected low-cost CI smoke lane marker in contract report payload" >&2
  exit 1
fi

if ! grep -q '"ci_smoke_runtime_budget_status": "within"' "$REPORT_FILE"; then
  echo "expected bounded CI smoke runtime budget marker in contract report payload" >&2
  exit 1
fi

if ! grep -q '"reason_key": "test_harness_loc_soft_budget_contract_ok"' "$REPORT_FILE"; then
  echo "expected deterministic reason_key marker in contract report payload" >&2
  exit 1
fi

if ! grep -Fq 'run_manifest_lane.sh --manifest scripts/framework/manifests/ci_test_harness_loc_soft_budget_contract_lane.json --phase contract --output-json /tmp/test-harness-loc-soft-budget-contract-report.json' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include manifest-runner generic soft-budget contract lane marker" >&2
  exit 1
fi

if ! grep -Fq 'run_manifest_lane.sh --manifest scripts/framework/manifests/ci_test_harness_loc_soft_budget_contract_lane.json --phase contract --output-json /tmp/test-harness-loc-soft-budget-contract-report.json' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include manifest-runner generic soft-budget contract lane marker" >&2
  exit 1
fi

if ! grep -Fq "test_harness_loc_soft_budget_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected generic soft-budget manifest to dispatch shared impl script" >&2
  exit 1
fi

echo "Generic test harness LOC soft-budget manifest contract lane tests passed."

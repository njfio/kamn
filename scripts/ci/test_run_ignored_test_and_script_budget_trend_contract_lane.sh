#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
LEGACY_LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_ignored_test_and_script_budget_trend_contract_lane.sh"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
SHARED_IMPL="$ROOT_DIR/scripts/ci/ignored_test_and_script_budget_trend_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/ci_ignored_test_and_script_budget_trend_contract_lane.json"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ -e "$LEGACY_LANE_SCRIPT" ]; then
  echo "expected superseded ignored-test+script-budget wrapper to be deleted: $LEGACY_LANE_SCRIPT" >&2
  exit 1
fi

test_harness_require_executable "$MANIFEST_RUNNER" "expected manifest lane runner to be executable"

test_harness_require_executable "$SHARED_IMPL" "expected ignored-test+script-budget trend shared impl script to be executable"

test_harness_require_file "$STRATEGY_DOC" "expected CI strategy doc to exist"

test_harness_require_file "$COST_DOC" "expected CI cost/lane framework doc to exist"

REPORT_FILE="$TMP_DIR/ignored-test-script-soft-budget-trend-contract-report.json"

lane_output="$(
  bash "$MANIFEST_RUNNER" \
    --manifest "$MANIFEST_FILE" \
    --phase contract \
    -- \
    --output-json "$REPORT_FILE" \
    --max-runtime-seconds 120
)"

if ! printf '%s\n' "$lane_output" | grep -q '^ignored_test_script_budget_trend_contract_status=pass$'; then
  echo "expected contract lane pass marker" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^ignored_test_script_budget_trend_contract_ignored_inventory_decision=GO$'; then
  echo "expected ignored inventory GO decision marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^ignored_test_script_budget_trend_contract_script_within_decision=GO$'; then
  echo "expected script trend within GO decision marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^ignored_test_script_budget_trend_contract_script_fail_decision=NO-GO$'; then
  echo "expected script trend fail NO-GO decision marker from contract lane" >&2
  exit 1
fi

test_harness_require_file "$REPORT_FILE" "expected ignored-test+script-budget trend contract report to be emitted"

if ! grep -q '"schema_version": "kamn.ci.ignored-test-script-soft-budget-trend-contract-report.v1"' "$REPORT_FILE"; then
  echo "expected contract report schema marker" >&2
  exit 1
fi

if ! grep -q '"ignored_stale_metadata_reason_contract": "pass"' "$REPORT_FILE"; then
  echo "expected ignored stale metadata reason-code contract marker to pass" >&2
  exit 1
fi

if ! grep -q '"script_trend_fail_reason_contract": "pass"' "$REPORT_FILE"; then
  echo "expected script trend fail reason-code contract marker to pass" >&2
  exit 1
fi

if ! grep -Fq 'run_manifest_lane.sh --manifest scripts/framework/manifests/ci_ignored_test_and_script_budget_trend_contract_lane.json --phase contract --output-json /tmp/ignored-test-script-soft-budget-trend-contract-report.json' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include manifest-runner ignored-test+script-budget trend lane marker" >&2
  exit 1
fi

if ! grep -Fq 'run_manifest_lane.sh --manifest scripts/framework/manifests/ci_ignored_test_and_script_budget_trend_contract_lane.json --phase contract --output-json /tmp/ignored-test-script-soft-budget-trend-contract-report.json' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include manifest-runner ignored-test+script-budget trend lane marker" >&2
  exit 1
fi

if ! grep -Fq "ignored_test_and_script_budget_trend_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected ignored-test+script-budget trend manifest to dispatch shared impl script" >&2
  exit 1
fi

echo "Ignored-test and script soft-budget trend manifest contract lane tests passed."

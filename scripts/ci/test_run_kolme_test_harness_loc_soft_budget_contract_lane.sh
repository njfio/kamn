#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
LEGACY_LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_kolme_test_harness_loc_soft_budget_contract_lane.sh"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
SHARED_IMPL="$ROOT_DIR/scripts/ci/kolme_test_harness_loc_soft_budget_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/ci_kolme_test_harness_loc_soft_budget_contract_lane.json"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ -e "$LEGACY_LANE_SCRIPT" ]; then
  echo "expected superseded Kolme soft-budget wrapper to be deleted: $LEGACY_LANE_SCRIPT" >&2
  exit 1
fi

test_harness_require_executable "$MANIFEST_RUNNER" "expected manifest lane runner to be executable"

test_harness_require_executable "$SHARED_IMPL" "expected Kolme soft-budget shared impl script to be executable"

test_harness_require_file "$STRATEGY_DOC" "expected CI strategy doc to exist"

test_harness_require_file "$COST_DOC" "expected CI cost/lane framework doc to exist"

REPORT_FILE="$TMP_DIR/kolme-test-harness-soft-budget-contract-report.json"

lane_output="$(
  bash "$MANIFEST_RUNNER" \
    --manifest "$MANIFEST_FILE" \
    --phase contract \
    -- \
    --output-json "$REPORT_FILE" \
    --max-runtime-seconds 120
)"

if ! printf '%s\n' "$lane_output" | grep -q '^kolme_test_harness_loc_soft_budget_contract_status=pass$'; then
  echo "expected contract lane pass marker" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^kolme_test_harness_loc_soft_budget_contract_go_decision=GO$'; then
  echo "expected GO decision marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^kolme_test_harness_loc_soft_budget_contract_warn_decision=WARN$'; then
  echo "expected WARN decision marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^kolme_test_harness_loc_soft_budget_contract_fail_decision=NO-GO$'; then
  echo "expected NO-GO decision marker from contract lane" >&2
  exit 1
fi

test_harness_require_file "$REPORT_FILE" "expected Kolme soft-budget contract report to be emitted"

if ! grep -q '"schema_version": "kamn.ci.kolme-test-harness-loc-soft-budget-contract-report.v1"' "$REPORT_FILE"; then
  echo "expected contract report schema marker" >&2
  exit 1
fi

if ! grep -q '"combined_reason_code_contract": "pass"' "$REPORT_FILE"; then
  echo "expected combined reason-code contract to pass" >&2
  exit 1
fi

if ! grep -q '"command_surface_fail_reason_contract": "pass"' "$REPORT_FILE"; then
  echo "expected command-surface fail reason contract to pass" >&2
  exit 1
fi

if ! grep -Fq 'run_manifest_lane.sh --manifest scripts/framework/manifests/ci_kolme_test_harness_loc_soft_budget_contract_lane.json --phase contract --output-json /tmp/kolme-test-harness-loc-soft-budget-contract-report.json' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include manifest-runner Kolme soft-budget contract lane marker" >&2
  exit 1
fi

if ! grep -Fq 'run_manifest_lane.sh --manifest scripts/framework/manifests/ci_kolme_test_harness_loc_soft_budget_contract_lane.json --phase contract --output-json /tmp/kolme-test-harness-loc-soft-budget-contract-report.json' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include manifest-runner Kolme soft-budget contract lane marker" >&2
  exit 1
fi

if ! grep -Fq "kolme_test_harness_loc_soft_budget_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected Kolme soft-budget manifest to dispatch shared impl script" >&2
  exit 1
fi

echo "Kolme test harness LOC soft-budget manifest contract lane tests passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_test_harness_loc_soft_budget_contract_lane.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected generic soft-budget contract lane script to be executable" >&2
  exit 1
fi

if [ ! -f "$STRATEGY_DOC" ]; then
  echo "expected CI strategy doc to exist" >&2
  exit 1
fi

if [ ! -f "$COST_DOC" ]; then
  echo "expected CI cost/lane framework doc to exist" >&2
  exit 1
fi

REPORT_FILE="$TMP_DIR/test-harness-soft-budget-contract-report.json"

lane_output="$(
  bash "$LANE_SCRIPT" \
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

if [ ! -f "$REPORT_FILE" ]; then
  echo "expected generic soft-budget contract report to be emitted" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.ci.test-harness-loc-soft-budget-contract-report.v1"' "$REPORT_FILE"; then
  echo "expected contract report schema marker" >&2
  exit 1
fi

if ! grep -q '"combined_reason_code_contract": "pass"' "$REPORT_FILE"; then
  echo "expected combined reason-code contract to pass" >&2
  exit 1
fi

if ! grep -Fq 'run_test_harness_loc_soft_budget_contract_lane.sh --output-json /tmp/test-harness-loc-soft-budget-contract-report.json' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include generic soft-budget contract lane command marker" >&2
  exit 1
fi

if ! grep -Fq 'run_test_harness_loc_soft_budget_contract_lane.sh --output-json /tmp/test-harness-loc-soft-budget-contract-report.json' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include generic soft-budget contract lane command marker" >&2
  exit 1
fi

echo "Generic test harness LOC soft-budget contract lane tests passed."

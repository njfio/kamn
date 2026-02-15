#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_ignored_test_and_script_budget_trend_contract_lane.sh"
SHARED_IMPL="$ROOT_DIR/scripts/ci/ignored_test_and_script_budget_trend_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/ci_ignored_test_and_script_budget_trend_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected ignored-test+script-budget trend contract lane wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_IMPL" ]; then
  echo "expected ignored-test+script-budget trend shared impl script to be executable" >&2
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

REPORT_FILE="$TMP_DIR/ignored-test-script-soft-budget-trend-contract-report.json"

lane_output="$(
  bash "$LANE_SCRIPT" \
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

if [ ! -f "$REPORT_FILE" ]; then
  echo "expected ignored-test+script-budget trend contract report to be emitted" >&2
  exit 1
fi

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

if ! grep -Fq 'run_ignored_test_and_script_budget_trend_contract_lane.sh --output-json /tmp/ignored-test-script-soft-budget-trend-contract-report.json' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include ignored-test+script-budget trend contract lane command marker" >&2
  exit 1
fi

if ! grep -Fq 'run_ignored_test_and_script_budget_trend_contract_lane.sh --output-json /tmp/ignored-test-script-soft-budget-trend-contract-report.json' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include ignored-test+script-budget trend contract lane command marker" >&2
  exit 1
fi

if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected ignored-test+script-budget trend wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected ignored-test+script-budget trend wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected ignored-test+script-budget trend wrapper to resolve CI manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "ignored_test_and_script_budget_trend_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected ignored-test+script-budget trend manifest to dispatch shared impl script" >&2
  exit 1
fi

echo "Ignored-test and script soft-budget trend contract lane tests passed."

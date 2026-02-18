#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
LEGACY_LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_fast_gate_budget_delta_contract_lane.sh"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
SHARED_IMPL="$ROOT_DIR/scripts/ci/fast_gate_budget_delta_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/ci_fast_gate_budget_delta_contract_lane.json"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ -e "$LEGACY_LANE_SCRIPT" ]; then
  echo "expected superseded fast-gate budget-delta wrapper to be deleted: $LEGACY_LANE_SCRIPT" >&2
  exit 1
fi

test_harness_require_executable "$MANIFEST_RUNNER" "expected manifest lane runner to be executable"

test_harness_require_executable "$SHARED_IMPL" "expected fast-gate budget-delta shared impl script to be executable"

test_harness_require_file "$STRATEGY_DOC" "expected CI strategy doc to exist"

test_harness_require_file "$COST_DOC" "expected CI cost/lane framework doc to exist"

REPORT_FILE="$TMP_DIR/fast-gate-budget-delta-contract-report.json"

lane_output="$(
  bash "$MANIFEST_RUNNER" \
    --manifest "$MANIFEST_FILE" \
    --phase contract \
    -- \
    --output-json "$REPORT_FILE" \
    --max-runtime-seconds 120
)"

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_status=pass$'; then
  echo "expected contract lane pass marker" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_pass_status=pass$'; then
  echo "expected pass-path status marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_unwaived_status=fail$'; then
  echo "expected unwaived-overrun fail marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_waived_status=pass$'; then
  echo "expected waived-overrun pass marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_ratchet_unwaived_status=fail$'; then
  echo "expected ratchet-regression unwaived fail marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_ratchet_waived_status=pass$'; then
  echo "expected ratchet-regression exception pass marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_stale_threshold_status=fail$'; then
  echo "expected stale-threshold fail marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_corrupt_threshold_status=fail$'; then
  echo "expected corrupt-threshold fail marker from contract lane" >&2
  exit 1
fi

test_harness_require_file "$REPORT_FILE" "expected fast-gate budget-delta contract report to be emitted"

if ! grep -q '"schema_version": "kamn.ci.fast-gate-budget-delta-contract-report.v1"' "$REPORT_FILE"; then
  echo "expected contract report schema marker" >&2
  exit 1
fi

if ! grep -q '"trend_contract_status": "pass"' "$REPORT_FILE"; then
  echo "expected trend contract status marker" >&2
  exit 1
fi

if ! grep -q '"stale_threshold_guard_status": "pass"' "$REPORT_FILE"; then
  echo "expected stale-threshold guard status marker" >&2
  exit 1
fi

if ! grep -q '"corrupt_threshold_guard_status": "pass"' "$REPORT_FILE"; then
  echo "expected corrupt-threshold guard status marker" >&2
  exit 1
fi

if ! grep -q '"ratchet_unwaived_status": "fail"' "$REPORT_FILE"; then
  echo "expected ratchet unwaived status marker in contract report" >&2
  exit 1
fi

if ! grep -q '"ratchet_waived_status": "pass"' "$REPORT_FILE"; then
  echo "expected ratchet exception-applied status marker in contract report" >&2
  exit 1
fi

if ! grep -Fq 'run_manifest_lane.sh --manifest scripts/framework/manifests/ci_fast_gate_budget_delta_contract_lane.json --phase contract --output-json /tmp/fast-gate-budget-delta-contract-report.json' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include manifest-runner fast-gate budget-delta lane marker" >&2
  exit 1
fi

if ! grep -Fq 'reason_codes=fast_gate_delta_threshold_file_stale' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include stale-threshold reason-code marker" >&2
  exit 1
fi

if ! grep -Fq 'reason_codes=fast_gate_delta_threshold_file_corrupt' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include corrupt-threshold reason-code marker" >&2
  exit 1
fi

if ! grep -Fq 'reason_codes=fast_gate_delta_threshold_ratchet_regression_unwaived' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include ratchet-regression fail reason-code marker" >&2
  exit 1
fi

if ! grep -Fq 'reason_codes=fast_gate_delta_threshold_ratchet_exception_applied' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include ratchet-exception reason-code marker" >&2
  exit 1
fi

if ! grep -Fq 'run_manifest_lane.sh --manifest scripts/framework/manifests/ci_fast_gate_budget_delta_contract_lane.json --phase contract --output-json /tmp/fast-gate-budget-delta-contract-report.json' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include manifest-runner fast-gate budget-delta lane marker" >&2
  exit 1
fi

if ! grep -Fq 'refresh .ci/fast-gate-budget-delta.env baseline and threshold metadata' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include fast-gate threshold remediation guidance" >&2
  exit 1
fi

if ! grep -Fq '.ci/fast-gate-budget-delta-ratchet.env' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include ratchet baseline fixture marker" >&2
  exit 1
fi

if ! grep -Fq "fast_gate_budget_delta_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected fast-gate budget-delta manifest to dispatch shared impl script" >&2
  exit 1
fi

echo "Fast-gate budget-delta manifest contract lane tests passed."

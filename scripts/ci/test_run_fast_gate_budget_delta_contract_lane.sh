#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_fast_gate_budget_delta_contract_lane.sh"
SHARED_IMPL="$ROOT_DIR/scripts/ci/fast_gate_budget_delta_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/ci_fast_gate_budget_delta_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected fast-gate budget-delta contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_IMPL" ]; then
  echo "expected fast-gate budget-delta shared impl script to be executable" >&2
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

REPORT_FILE="$TMP_DIR/fast-gate-budget-delta-contract-report.json"

lane_output="$(
  bash "$LANE_SCRIPT" \
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

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_stale_threshold_status=fail$'; then
  echo "expected stale-threshold fail marker from contract lane" >&2
  exit 1
fi

if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_budget_delta_contract_corrupt_threshold_status=fail$'; then
  echo "expected corrupt-threshold fail marker from contract lane" >&2
  exit 1
fi

if [ ! -f "$REPORT_FILE" ]; then
  echo "expected fast-gate budget-delta contract report to be emitted" >&2
  exit 1
fi

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

if ! grep -Fq 'run_fast_gate_budget_delta_contract_lane.sh --output-json /tmp/fast-gate-budget-delta-contract-report.json' "$STRATEGY_DOC"; then
  echo "expected CI strategy doc to include fast-gate budget-delta contract lane command marker" >&2
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

if ! grep -Fq 'run_fast_gate_budget_delta_contract_lane.sh --output-json /tmp/fast-gate-budget-delta-contract-report.json' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include fast-gate budget-delta contract lane command marker" >&2
  exit 1
fi

if ! grep -Fq 'refresh .ci/fast-gate-budget-delta.env baseline and threshold metadata' "$COST_DOC"; then
  echo "expected CI cost/lane framework doc to include fast-gate threshold remediation guidance" >&2
  exit 1
fi

if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected fast-gate budget-delta wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected fast-gate budget-delta wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected fast-gate budget-delta wrapper to resolve CI manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "fast_gate_budget_delta_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected fast-gate budget-delta manifest to dispatch shared impl script" >&2
  exit 1
fi

echo "Fast-gate budget-delta contract lane tests passed."

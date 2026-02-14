#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_shutdown_abrupt_close_regression_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_shutdown_abrupt_close_regression_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

output_json=""
policy_output_json=""
max_seconds="${KAMN_SERVICE_API_SHUTDOWN_ABRUPT_CLOSE_REGRESSION_CONTRACT_MAX_SECONDS:-240}"
ci_fast_gate="PASS"
mode="dry-run"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --policy-output-json)
      policy_output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi
if [[ "$mode" != "dry-run" && "$mode" != "run" ]]; then
  echo "mode must be dry-run or run" >&2
  exit 1
fi

for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
for required_doc in "$STRATEGY_DOC" "$ROADMAP_DOC"; do
  if [ ! -f "$required_doc" ]; then
    echo "expected required documentation file '$required_doc'" >&2
    exit 1
  fi
done

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/service-api-shutdown-abrupt-close-regression-live-summary.json"
policy_report="$TMP_DIR/service-api-shutdown-abrupt-close-regression-live-policy.json"
tampered_report="$TMP_DIR/service-api-shutdown-abrupt-close-regression-live-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --output-json "$summary_report"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected service api shutdown abrupt-close regression validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api shutdown abrupt-close regression validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^lane_mode=$mode$"; then
  echo "expected service api shutdown abrupt-close regression validation lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^abrupt_close_guard_status=verified$'; then
  echo "expected service api shutdown abrupt-close regression abrupt-close marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_rejection_guard_status=verified$'; then
  echo "expected service api shutdown abrupt-close regression websocket marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^shutdown_timeout_guard_status=verified$'; then
  echo "expected service api shutdown abrupt-close regression timeout marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^replayed_signal_guard_status=verified$'; then
  echo "expected service api shutdown abrupt-close regression replayed-signal marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected service api shutdown abrupt-close regression fail-closed marker" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected service api shutdown abrupt-close regression policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api shutdown abrupt-close regression policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_shutdown_abrupt_close_regression_policy_status=verified$'; then
  echo "expected service api shutdown abrupt-close regression policy checker status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["abrupt_close_guard_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/service-api-shutdown-abrupt-close-regression-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e

if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered service api shutdown abrupt-close regression report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'service_api_shutdown_abrupt_close_regression_policy_marker_missing:abrupt_close_guard_status'; then
  echo "expected deterministic fail-closed reason for tampered service api shutdown abrupt-close regression report" >&2
  exit 1
fi

for required_ref in \
  "validate_service_api_shutdown_abrupt_close_regression_live.sh" \
  "check_service_api_shutdown_abrupt_close_regression_live_policy.sh" \
  "validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh" \
  "test_validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh" \
  "test_check_service_api_shutdown_abrupt_close_regression_live_policy.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "service api shutdown abrupt-close regression contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include service api shutdown abrupt-close regression contract-lane exclusion marker" >&2
  exit 1
fi

if ! grep -q "Task #3275" "$ROADMAP_DOC"; then
  echo "expected roadmap marker for Task #3275" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference service api shutdown abrupt-close regression contract lane script" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/check_service_api_shutdown_abrupt_close_regression_live_policy.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference service api shutdown abrupt-close regression policy checker script" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "service api shutdown abrupt-close regression contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/service-api-shutdown-abrupt-close-regression-live-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" "$mode" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
mode = sys.argv[6]

if summary_report.get("schema_version") != "kamn.runtime.service-api-shutdown-abrupt-close-regression-live-report.v1":
    raise SystemExit("unexpected service api shutdown abrupt-close regression live summary schema")
if policy_report.get("schema_version") != "kamn.runtime.service-api-shutdown-abrupt-close-regression-live-policy-report.v1":
    raise SystemExit("unexpected service api shutdown abrupt-close regression live policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected service api shutdown abrupt-close regression summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected service api shutdown abrupt-close regression policy final_decision=GO")

lane_report = {
    "schema_version": "kamn.runtime.service-api-shutdown-abrupt-close-regression-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "service_api_shutdown_abrupt_close_regression_contract_status": "verified",
    "service_api_shutdown_abrupt_close_regression_policy_status": policy_report.get(
        "service_api_shutdown_abrupt_close_regression_policy_status"
    ),
    "docs_contract_status": "verified",
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "service_api_shutdown_abrupt_close_regression_policy_marker_missing:abrupt_close_guard_status",
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
lane_report_file.write_text(json.dumps(lane_report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if [[ -n "$output_json" ]]; then
  cp "$lane_report" "$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  cp "$policy_report" "$policy_output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "lane_mode=$mode"
echo "service_api_shutdown_abrupt_close_regression_contract_status=verified"
echo "service_api_shutdown_abrupt_close_regression_policy_status=verified"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=service_api_shutdown_abrupt_close_regression_policy_marker_missing:abrupt_close_guard_status"
echo "performance_budget_status=verified"

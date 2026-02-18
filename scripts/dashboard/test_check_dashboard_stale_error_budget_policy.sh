#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/dashboard/run_dashboard_stale_error_budget_lane.sh"
CHECKER="$ROOT_DIR/scripts/dashboard/check_dashboard_stale_error_budget_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$LANE_SCRIPT" "expected dashboard stale/error budget lane script to be executable"

test_harness_require_executable "$CHECKER" "expected dashboard stale/error budget policy checker script to be executable"

go_report="$TMP_DIR/dashboard-stale-error-go.json"
KAMN_DASHBOARD_STALE_ERROR_SKIP_COMMANDS=true \
bash "$LANE_SCRIPT" --output-json "$go_report" >/dev/null

go_checker_output="$(bash "$CHECKER" --report-file "$go_report")"
if ! printf '%s\n' "$go_checker_output" | grep -q '^status=ok$'; then
  echo "expected dashboard stale/error budget policy checker status marker for GO report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^final_decision=GO$'; then
  echo "expected dashboard stale/error budget policy checker GO decision for GO report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^failed_checks=none$'; then
  echo "expected dashboard stale/error budget policy checker no failed checks for GO report" >&2
  exit 1
fi

error_no_go_report="$TMP_DIR/dashboard-stale-error-no-go.json"
set +e
KAMN_DASHBOARD_STALE_ERROR_SKIP_COMMANDS=true \
KAMN_DASHBOARD_STALE_ERROR_FORCE_ERROR_BUDGET_MISSING=true \
bash "$LANE_SCRIPT" --output-json "$error_no_go_report" >/dev/null 2>&1
error_no_go_code=$?
set -e

if [ "$error_no_go_code" -eq 0 ]; then
  echo "expected forced error-budget-missing dashboard stale/error lane run to fail closed" >&2
  exit 1
fi

error_no_go_checker_output="$(bash "$CHECKER" --report-file "$error_no_go_report")"
if ! printf '%s\n' "$error_no_go_checker_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected dashboard stale/error budget policy checker NO-GO decision for error-budget-missing report" >&2
  exit 1
fi
if ! printf '%s\n' "$error_no_go_checker_output" | grep -q 'error_budget_threshold_missing'; then
  echo "expected dashboard stale/error budget policy checker failed checks to include error_budget_threshold_missing" >&2
  exit 1
fi

tampered_report="$TMP_DIR/dashboard-stale-error-tampered.json"
cp "$go_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes"] = ["stale_data_threshold_missing"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered dashboard stale/error report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q 'reason_codes mismatch'; then
  echo "expected reason_codes mismatch failure for tampered dashboard stale/error report" >&2
  exit 1
fi

# Regression: #942
if ! printf '%s\n' "$tampered_output" | grep -q 'expected reason_codes'; then
  echo "expected explicit reason-code mismatch output for dashboard stale/error regression path" >&2
  exit 1
fi

echo "dashboard stale/error budget policy checker tests passed."

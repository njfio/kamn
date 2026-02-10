#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/frontend/run_dashboard_shell_determinism_matrix_lane.sh"
CHECKER="$ROOT_DIR/scripts/frontend/check_dashboard_shell_determinism_matrix_policy.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/frontend/dashboard_shell_determinism_matrix_policy_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected dashboard shell matrix lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected dashboard shell matrix policy checker script to be executable" >&2
  exit 1
fi

if ! grep -q 'dashboard_shell_determinism_matrix_policy_contract.py' "$CHECKER"; then
  echo "expected dashboard shell matrix policy checker wrapper to delegate to shared implementation" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared dashboard shell matrix policy checker implementation to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/dashboard-shell-matrix-go.json"
KAMN_FRONTEND_SHELL_MATRIX_SKIP_COMMANDS=true \
bash "$LANE_SCRIPT" --output-json "$go_report" >/dev/null

go_checker_output="$(bash "$CHECKER" --report-file "$go_report")"
if ! printf '%s\n' "$go_checker_output" | grep -q '^status=ok$'; then
  echo "expected dashboard shell matrix policy checker status marker for GO report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^final_decision=GO$'; then
  echo "expected dashboard shell matrix policy checker GO decision for GO report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^failed_checks=none$'; then
  echo "expected dashboard shell matrix policy checker no failed checks for GO report" >&2
  exit 1
fi

error_no_go_report="$TMP_DIR/dashboard-shell-matrix-error-no-go.json"
set +e
KAMN_FRONTEND_SHELL_MATRIX_SKIP_COMMANDS=true \
KAMN_FRONTEND_SHELL_MATRIX_FORCE_ERROR_STATE_MISSING=true \
bash "$LANE_SCRIPT" --output-json "$error_no_go_report" >/dev/null 2>&1
error_no_go_code=$?
set -e

if [ "$error_no_go_code" -eq 0 ]; then
  echo "expected forced error-state-missing dashboard shell matrix lane run to fail closed" >&2
  exit 1
fi

error_no_go_checker_output="$(bash "$CHECKER" --report-file "$error_no_go_report")"
if ! printf '%s\n' "$error_no_go_checker_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected dashboard shell matrix policy checker NO-GO decision for error-state-missing report" >&2
  exit 1
fi
if ! printf '%s\n' "$error_no_go_checker_output" | grep -q 'error_state_missing'; then
  echo "expected dashboard shell matrix policy checker failed checks to include error_state_missing" >&2
  exit 1
fi

tampered_report="$TMP_DIR/dashboard-shell-matrix-tampered.json"
cp "$go_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes"] = ["stale_critical_state_missing"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered dashboard shell matrix report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q 'reason_codes mismatch'; then
  echo "expected reason_codes mismatch failure for tampered dashboard shell matrix report" >&2
  exit 1
fi

# Regression: #943
if ! printf '%s\n' "$tampered_output" | grep -q 'expected reason_codes'; then
  echo "expected explicit reason-code mismatch output for dashboard shell matrix regression path" >&2
  exit 1
fi

echo "dashboard shell determinism matrix policy checker tests passed."

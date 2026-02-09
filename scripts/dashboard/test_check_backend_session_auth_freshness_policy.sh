#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/dashboard/run_backend_session_auth_freshness_lane.sh"
CHECKER="$ROOT_DIR/scripts/dashboard/check_backend_session_auth_freshness_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected dashboard backend session/auth freshness lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected dashboard backend session/auth freshness policy checker script to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/dashboard-backend-session-auth-freshness-go.json"
KAMN_DASHBOARD_BACKEND_SESSION_SKIP_COMMANDS=true \
bash "$LANE_SCRIPT" --output-json "$go_report" >/dev/null

go_checker_output="$(bash "$CHECKER" --report-file "$go_report")"
if ! printf '%s\n' "$go_checker_output" | grep -q '^status=ok$'; then
  echo "expected dashboard backend session/auth freshness policy checker status marker for GO report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^final_decision=GO$'; then
  echo "expected dashboard backend session/auth freshness policy checker GO decision for GO report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^failed_checks=none$'; then
  echo "expected dashboard backend session/auth freshness policy checker no failed checks for GO report" >&2
  exit 1
fi

freshness_no_go_report="$TMP_DIR/dashboard-backend-session-auth-freshness-freshness-no-go.json"
set +e
KAMN_DASHBOARD_BACKEND_SESSION_SKIP_COMMANDS=true \
KAMN_DASHBOARD_BACKEND_SESSION_FORCE_FRESHNESS_GUARD_MISSING=true \
bash "$LANE_SCRIPT" --output-json "$freshness_no_go_report" >/dev/null 2>&1
freshness_no_go_code=$?
set -e

if [ "$freshness_no_go_code" -eq 0 ]; then
  echo "expected forced freshness-guard-missing dashboard backend lane run to fail closed" >&2
  exit 1
fi

freshness_no_go_checker_output="$(bash "$CHECKER" --report-file "$freshness_no_go_report")"
if ! printf '%s\n' "$freshness_no_go_checker_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected dashboard backend session/auth freshness policy checker NO-GO decision for freshness-guard-missing report" >&2
  exit 1
fi
if ! printf '%s\n' "$freshness_no_go_checker_output" | grep -q 'freshness_guard_missing'; then
  echo "expected dashboard backend session/auth freshness policy checker failed checks to include freshness_guard_missing" >&2
  exit 1
fi

tampered_report="$TMP_DIR/dashboard-backend-session-auth-freshness-tampered-reason-codes.json"
cp "$go_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes"] = ["freshness_guard_missing"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered dashboard backend session/auth freshness report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q 'reason_codes mismatch'; then
  echo "expected reason_codes mismatch failure for tampered dashboard backend session/auth freshness report" >&2
  exit 1
fi

# Regression: #941
if ! printf '%s\n' "$tampered_output" | grep -q 'expected reason_codes'; then
  echo "expected explicit reason-code mismatch output for dashboard backend session/auth freshness regression path" >&2
  exit 1
fi

echo "dashboard backend session/auth freshness policy checker tests passed."

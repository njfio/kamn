#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/sdk/check_localhost_signed_integration_evidence_policy.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected localhost signed integration evidence policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected localhost signed integration contract lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/localhost-signed-integration-contract-report.json"
bash "$LANE_SCRIPT" --output-json "$report_file" >/dev/null

go_output="$(bash "$CHECKER" --report-file "$report_file")"
if ! printf '%s\n' "$go_output" | grep -Fq "status=ok"; then
  echo "expected localhost signed integration evidence policy checker success status" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "final_decision=GO"; then
  echo "expected localhost signed integration evidence policy checker final decision GO marker" >&2
  exit 1
fi

tampered_report="$TMP_DIR/localhost-signed-integration-contract-report.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["admission_reason_codes"] = ["tampered_reason_code"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered localhost signed integration report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -Fq "admission_reason_codes"; then
  echo "expected explicit admission reason-code policy error" >&2
  exit 1
fi

# Regression: #880
if ! printf '%s\n' "$tampered_output" | grep -Fq "stale_session_detected"; then
  echo "expected deterministic admission reason markers in localhost signed integration policy regression path" >&2
  exit 1
fi

expiry_tampered_report="$TMP_DIR/localhost-signed-integration-contract-report.expiry-tampered.json"
cp "$report_file" "$expiry_tampered_report"
python3 - "$expiry_tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["session_expired_reason_code"] = "tampered_expiry_reason"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
expiry_tampered_output="$(bash "$CHECKER" --report-file "$expiry_tampered_report" 2>&1)"
expiry_tampered_code=$?
set -e

if [ "$expiry_tampered_code" -eq 0 ]; then
  echo "expected session-expired reason drift report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$expiry_tampered_output" | grep -Fq "session_expired_reason_code"; then
  echo "expected explicit session expired reason-code policy error" >&2
  exit 1
fi

if ! printf '%s\n' "$expiry_tampered_output" | grep -Fq "session_expired_detected"; then
  echo "expected deterministic session-expired reason marker in policy regression path" >&2
  exit 1
fi

decision_tampered_report="$TMP_DIR/localhost-signed-integration-contract-report.decision-tampered.json"
cp "$report_file" "$decision_tampered_report"
python3 - "$decision_tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
decision_tampered_output="$(bash "$CHECKER" --report-file "$decision_tampered_report" 2>&1)"
decision_tampered_code=$?
set -e

if [ "$decision_tampered_code" -eq 0 ]; then
  echo "expected final-decision drift report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$decision_tampered_output" | grep -Fq "final_decision"; then
  echo "expected explicit final decision mismatch error from policy checker" >&2
  exit 1
fi

echo "localhost signed integration evidence policy checker tests passed."

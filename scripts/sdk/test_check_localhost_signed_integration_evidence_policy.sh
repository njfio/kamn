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

echo "localhost signed integration evidence policy checker tests passed."

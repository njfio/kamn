#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/message/run_didcomm_envelope_compatibility_replay.py"
CHECKER="$ROOT_DIR/scripts/message/check_didcomm_envelope_compatibility_policy.sh"
FIXTURE="$ROOT_DIR/fixtures/didcomm_envelope_compatibility/replay_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected DIDComm envelope compatibility replay runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected DIDComm envelope compatibility policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE" ]; then
  echo "expected DIDComm envelope compatibility fixture file to exist" >&2
  exit 1
fi

report_file="$TMP_DIR/didcomm-envelope-compatibility-report.json"
python3 "$RUNNER" --fixture "$FIXTURE" --output-json "$report_file" >/dev/null

policy_output="$(bash "$CHECKER" --report-file "$report_file")"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected DIDComm envelope policy checker status=ok" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected DIDComm envelope policy checker final_decision=GO" >&2
  exit 1
fi

tampered_report="$TMP_DIR/didcomm-envelope-compatibility-tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["case_results"][2]["decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered DIDComm envelope report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "case decision mismatch"; then
  echo "expected case decision mismatch failure for tampered DIDComm envelope report" >&2
  exit 1
fi

# Regression: #892
if ! printf '%s\n' "$tampered_output" | grep -q "vector_f1_missing_recipient_key"; then
  echo "expected explicit case identifier in DIDComm envelope tamper regression failure" >&2
  exit 1
fi

echo "DIDComm envelope compatibility policy checker tests passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/message/run_a2a_mcp_conformance_harness.py"
CHECKER="$ROOT_DIR/scripts/message/check_a2a_mcp_conformance_policy.sh"
FIXTURE="$ROOT_DIR/fixtures/a2a_mcp_conformance/replay_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected A2A/MCP conformance harness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected A2A/MCP conformance policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE" ]; then
  echo "expected A2A/MCP conformance fixture file to exist" >&2
  exit 1
fi

report_file="$TMP_DIR/a2a-mcp-conformance-report.json"
python3 "$RUNNER" --fixture "$FIXTURE" --output-json "$report_file" >/dev/null

policy_output="$(bash "$CHECKER" --report-file "$report_file")"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected A2A/MCP conformance policy checker status=ok" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected A2A/MCP conformance policy checker final_decision=GO" >&2
  exit 1
fi

tampered_report="$TMP_DIR/a2a-mcp-conformance-tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["case_results"][3]["decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered A2A/MCP conformance report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "case decision mismatch"; then
  echo "expected case decision mismatch failure for tampered A2A/MCP conformance report" >&2
  exit 1
fi

# Regression: #893
if ! printf '%s\n' "$tampered_output" | grep -q "request_with_mismatched_concept"; then
  echo "expected explicit case identifier in A2A/MCP tamper regression failure" >&2
  exit 1
fi

echo "A2A/MCP conformance policy checker tests passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/message/run_a2a_mcp_conformance_harness.py"
FIXTURE="$ROOT_DIR/fixtures/a2a_mcp_conformance/replay_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected A2A/MCP conformance harness runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE" ]; then
  echo "expected A2A/MCP conformance fixture file to exist" >&2
  exit 1
fi

output="$(
  python3 "$RUNNER" \
    --fixture "$FIXTURE" \
    --output-json "$TMP_REPORT"
)"

if ! printf '%s\n' "$output" | grep -q '^status=pass;'; then
  echo "expected A2A/MCP conformance harness runner to pass fixture matrix" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text(encoding="utf-8"))

if payload.get("schema_version") != "kamn.a2a_mcp.conformance-report.v1":
    raise SystemExit("unexpected A2A/MCP conformance report schema version")

if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for deterministic A2A/MCP conformance matrix")

cases = payload.get("case_results", [])
if not cases:
    raise SystemExit("expected non-empty A2A/MCP conformance case_results")

expected_case_ids = {
    "request_task_invoke_tool_call",
    "response_task_result_tool_result",
    "event_notify_notification",
    "request_with_mismatched_concept",
}
observed_ids = {case.get("case_id") for case in cases}
if observed_ids != expected_case_ids:
    raise SystemExit(f"unexpected A2A/MCP conformance case ids: {sorted(observed_ids)}")
PY

echo "A2A/MCP conformance harness runner tests passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/message/run_a2a_mcp_conformance_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected A2A/MCP conformance contract lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/a2a-mcp-conformance-contract-report.json"
output="$(
  bash "$SCRIPT" \
    --output-json "$report_file" \
    --skip-tests
)"

if ! printf '%s\n' "$output" | grep -q "A2A/MCP conformance contract lane tests passed."; then
  echo "expected success output from A2A/MCP conformance contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected A2A/MCP conformance contract lane to emit report" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.a2a_mcp.conformance-report.v1"' "$report_file"; then
  echo "expected A2A/MCP conformance report schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "a2a_mcp_conformance_reason_codes:GO:v1"' "$report_file"; then
  echo "expected A2A/MCP conformance reason key marker in report" >&2
  exit 1
fi

echo "A2A/MCP conformance contract lane script tests passed."

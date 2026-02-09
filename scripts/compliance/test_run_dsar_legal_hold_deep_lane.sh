#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/compliance/run_dsar_legal_hold_deep_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected DSAR legal-hold deep lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/dsar-legal-hold-report.json"
lane_output="$(bash "$DEEP_LANE" --output-json "$report_file")"

if ! printf '%s\n' "$lane_output" | grep -q "dsar legal-hold deep lane tests passed."; then
  echo "expected DSAR legal-hold deep lane success output" >&2
  exit 1
fi

if [ ! -s "$report_file" ]; then
  echo "expected DSAR legal-hold deep lane to emit a non-empty report artifact" >&2
  exit 1
fi

if ! grep -q '"status": "pass"' "$report_file"; then
  echo "expected DSAR legal-hold deep report to capture pass status" >&2
  exit 1
fi

echo "dsar legal-hold deep lane script tests passed."


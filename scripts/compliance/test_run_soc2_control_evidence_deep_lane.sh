#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
DEEP_LANE="$ROOT_DIR/scripts/compliance/run_soc2_control_evidence_deep_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$DEEP_LANE" "expected SOC2 control evidence deep lane script to be executable"

report_file="$TMP_DIR/soc2-control-evidence-report.json"
lane_output="$(bash "$DEEP_LANE" --output-json "$report_file")"

if ! printf '%s\n' "$lane_output" | grep -q "soc2 control evidence deep lane tests passed."; then
  echo "expected SOC2 control evidence deep lane success output" >&2
  exit 1
fi

if [ ! -s "$report_file" ]; then
  echo "expected SOC2 control evidence deep lane to emit a non-empty report artifact" >&2
  exit 1
fi

if ! grep -q '"status": "pass"' "$report_file"; then
  echo "expected SOC2 control evidence deep report to capture pass status" >&2
  exit 1
fi

echo "soc2 control evidence deep lane script tests passed."


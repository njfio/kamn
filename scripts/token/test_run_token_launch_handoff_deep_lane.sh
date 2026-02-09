#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/token/run_token_launch_handoff_deep_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected token launch handoff deep lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/token-launch-handoff-report.json"
lane_output="$(bash "$DEEP_LANE" --output-json "$report_file")"

if ! printf '%s\n' "$lane_output" | grep -q "token launch handoff deep lane tests passed."; then
  echo "expected token launch handoff deep lane success output" >&2
  exit 1
fi

if [ ! -s "$report_file" ]; then
  echo "expected token launch handoff deep lane to emit a non-empty report artifact" >&2
  exit 1
fi

if ! grep -q '"final_decision": "NO-GO"' "$report_file"; then
  echo "expected token launch handoff deep lane report to capture NO-GO scenario" >&2
  exit 1
fi

echo "token launch handoff deep lane script tests passed."

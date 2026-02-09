#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/governance/run_stake_slash_risk_deep_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected stake/slash risk deep lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/governance-stake-slash-report.json"
lane_output="$(bash "$DEEP_LANE" --output-json "$report_file")"

if ! printf '%s\n' "$lane_output" | grep -q "governance stake/slash deep lane tests passed."; then
  echo "expected stake/slash risk deep lane success output" >&2
  exit 1
fi

if [ ! -s "$report_file" ]; then
  echo "expected stake/slash risk deep lane to emit a non-empty report artifact" >&2
  exit 1
fi

if ! grep -q '"status": "pass"' "$report_file"; then
  echo "expected stake/slash risk deep report to capture pass status" >&2
  exit 1
fi

echo "stake/slash risk deep lane script tests passed."


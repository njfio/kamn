#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/compliance/run_dsar_legal_hold_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/compliance/run_dsar_legal_hold_deep_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected DSAR legal-hold contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected DSAR legal-hold deep lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "dsar legal-hold contract lane tests passed."; then
  echo "expected DSAR legal-hold contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_dsar_legal_hold_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected deep lane script to invoke DSAR contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "dsar-legal-hold-report.json" "$DEEP_LANE"; then
  echo "expected deep lane script to emit DSAR legal-hold report artifact" >&2
  exit 1
fi

echo "dsar legal-hold contract lane script tests passed."


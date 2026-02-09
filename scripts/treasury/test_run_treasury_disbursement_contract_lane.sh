#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/treasury/run_treasury_disbursement_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected treasury disbursement contract lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "treasury disbursement contract lane tests passed."; then
  echo "expected treasury disbursement contract lane success output" >&2
  exit 1
fi

echo "treasury disbursement contract lane script tests passed."

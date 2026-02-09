#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_snapshot_drift_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected Kolme snapshot drift contract lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "Kolme snapshot drift contract lane tests passed."; then
  echo "expected Kolme snapshot drift contract lane success marker" >&2
  exit 1
fi

echo "Kolme snapshot drift contract lane script tests passed."

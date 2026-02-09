#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_runtime_commit_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected Kolme runtime commit contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "kolme_runtime_commit_finality" "$CONTRACT_LANE"; then
  echo "expected Kolme runtime commit contract lane to include finality projection tests" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "Kolme runtime commit contract lane tests passed."; then
  echo "expected Kolme runtime commit contract lane success marker" >&2
  exit 1
fi

echo "Kolme runtime commit contract lane script tests passed."

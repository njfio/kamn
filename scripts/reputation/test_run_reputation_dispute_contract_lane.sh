#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/reputation/run_reputation_dispute_contract_lane.sh"

output="$(bash "$SCRIPT")"

if ! printf '%s\n' "$output" | grep -q "reputation dispute contract lane tests passed."; then
  echo "expected success output from reputation dispute contract lane" >&2
  exit 1
fi

echo "reputation dispute contract lane script tests passed."

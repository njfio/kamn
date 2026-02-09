#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/governance/run_stake_slash_risk_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/governance/run_stake_slash_risk_deep_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected stake/slash risk contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected stake/slash risk deep lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "stake/slash risk contract lane tests passed."; then
  echo "expected stake/slash risk contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_stake_slash_risk_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected deep lane script to invoke stake/slash contract lane checks first" >&2
  exit 1
fi

if ! grep -q "governance-stake-slash-report.json" "$DEEP_LANE"; then
  echo "expected deep lane script to emit stake/slash deep report artifact" >&2
  exit 1
fi

echo "stake/slash risk contract lane script tests passed."


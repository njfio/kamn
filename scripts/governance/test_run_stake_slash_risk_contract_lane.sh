#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/governance/run_stake_slash_risk_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/governance/run_stake_slash_risk_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/governance/stake_slash_risk_contract_lane_contract.py"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected stake/slash risk contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected stake/slash risk deep lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected stake/slash risk shared contract-lane module to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "stake/slash risk contract lane tests passed."; then
  echo "expected stake/slash risk contract lane success marker" >&2
  exit 1
fi

if ! grep -q "stake_slash_risk_contract_lane_contract.py" "$CONTRACT_LANE"; then
  echo "expected stake/slash contract lane wrapper to dispatch to shared module" >&2
  exit 1
fi

if ! grep -q "generate_stake_slash_risk_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected stake/slash shared contract-lane module to run evidence generator" >&2
  exit 1
fi

if ! grep -q "check_stake_slash_risk_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected stake/slash shared contract-lane module to run policy checker" >&2
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

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/governance/run_governance_simulation_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/governance/run_governance_simulation_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/governance/governance_simulation_contract_lane_contract.py"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected governance simulation contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected governance simulation deep lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected governance simulation shared contract-lane module to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "governance simulation contract lane tests passed."; then
  echo "expected governance simulation contract lane success marker" >&2
  exit 1
fi
if ! grep -q "governance_simulation_contract_lane_contract.py" "$CONTRACT_LANE"; then
  echo "expected governance simulation contract lane wrapper to dispatch to shared module" >&2
  exit 1
fi
if ! grep -q "generate_governance_simulation_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected governance simulation shared contract-lane module to run evidence generator" >&2
  exit 1
fi
if ! grep -q "check_governance_simulation_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected governance simulation shared contract-lane module to run policy checker" >&2
  exit 1
fi

if ! grep -Fq "run_governance_simulation_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected deep lane script to invoke governance contract lane checks first" >&2
  exit 1
fi

if ! grep -q "governance-simulation-report.json" "$DEEP_LANE"; then
  echo "expected deep lane script to emit governance simulation report artifact" >&2
  exit 1
fi

echo "governance simulation contract lane script tests passed."

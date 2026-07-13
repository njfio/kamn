#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
CONTRACT_WRAPPER="run_governance_simulation_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
DEEP_LANE="$ROOT_DIR/scripts/governance/run_governance_simulation_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/governance/governance_simulation_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/governance_simulation_contract_lane.json"

if [ ! -x "$MANIFEST_RUNNER" ]; then
  echo "expected manifest runner to be executable" >&2
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
if [ ! -f "$MANIFEST" ]; then
  echo "expected governance simulation manifest to exist" >&2
  exit 1
fi

lane_output="$(bash "$MANIFEST_RUNNER" --manifest "$MANIFEST" --phase contract)"
if ! printf '%s\n' "$lane_output" | grep -q "governance simulation contract lane tests passed."; then
  echo "expected governance simulation contract lane success marker" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$CONTRACT_WRAPPER" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected governance simulation contract lane wrapper to resolve governance manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q '"wrapper_name": "run_governance_simulation_contract_lane.sh"' "$MANIFEST"; then
  echo "expected governance simulation manifest wrapper_name metadata marker" >&2
  exit 1
fi
if ! grep -q '"phase": "contract"' "$MANIFEST"; then
  echo "expected governance simulation manifest phase metadata marker" >&2
  exit 1
fi
if ! grep -q "governance_simulation_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected governance simulation manifest to dispatch to shared module" >&2
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
if ! grep -q "from framework.contract_lane_helpers import" "$SHARED_CONTRACT"; then
  echo "expected governance simulation shared contract-lane module to use framework lane helpers" >&2
  exit 1
fi

if ! grep -Fq "governance_simulation_contract_lane.json" "$DEEP_LANE"; then
  echo "expected deep lane script to invoke governance contract manifest first" >&2
  exit 1
fi

if ! grep -q "governance-simulation-report.json" "$DEEP_LANE"; then
  echo "expected deep lane script to emit governance simulation report artifact" >&2
  exit 1
fi

echo "governance simulation contract lane script tests passed."

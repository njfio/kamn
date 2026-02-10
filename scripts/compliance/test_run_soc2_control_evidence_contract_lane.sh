#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/compliance/run_soc2_control_evidence_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/compliance/soc2_control_evidence_contract_lane_contract.py"
DEEP_LANE="$ROOT_DIR/scripts/compliance/run_soc2_control_evidence_deep_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected SOC2 control evidence contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q 'soc2_control_evidence_contract_lane_contract.py' "$CONTRACT_LANE"; then
  echo "expected SOC2 contract-lane wrapper to delegate to shared implementation" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected shared SOC2 contract-lane implementation to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected SOC2 control evidence deep lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "soc2 control evidence contract lane tests passed."; then
  echo "expected SOC2 control evidence contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_soc2_control_evidence_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected deep lane script to invoke SOC2 contract lane checks first" >&2
  exit 1
fi

if ! grep -q "soc2-control-evidence-report.json" "$DEEP_LANE"; then
  echo "expected deep lane script to emit SOC2 deep report artifact" >&2
  exit 1
fi

if ! grep -q "generate_soc2_control_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected shared SOC2 contract-lane implementation to execute bundle generator" >&2
  exit 1
fi

if ! grep -q "check_soc2_control_evidence_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected shared SOC2 contract-lane implementation to execute policy checker" >&2
  exit 1
fi

echo "soc2 control evidence contract lane script tests passed."

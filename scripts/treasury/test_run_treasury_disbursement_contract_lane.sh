#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/treasury/run_treasury_disbursement_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/treasury/treasury_disbursement_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/treasury_disbursement_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected treasury disbursement contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected treasury disbursement shared contract-lane module to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected treasury disbursement contract-lane manifest to exist" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "treasury disbursement contract lane tests passed."; then
  echo "expected treasury disbursement contract lane success output" >&2
  exit 1
fi
if [ ! -L "$CONTRACT_LANE" ]; then
  echo "expected treasury disbursement contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$CONTRACT_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected treasury disbursement contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected treasury disbursement contract lane wrapper to resolve treasury manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q "treasury_disbursement_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected treasury disbursement manifest to dispatch to shared module" >&2
  exit 1
fi
if ! grep -q "generate_treasury_disbursement_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected treasury disbursement shared contract-lane module to run evidence generator" >&2
  exit 1
fi
if ! grep -q "check_treasury_disbursement_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected treasury disbursement shared contract-lane module to run policy checker" >&2
  exit 1
fi

echo "treasury disbursement contract lane script tests passed."

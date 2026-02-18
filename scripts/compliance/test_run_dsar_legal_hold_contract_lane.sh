#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/compliance/run_dsar_legal_hold_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/compliance/dsar_legal_hold_contract_lane_contract.py"
DEEP_LANE="$ROOT_DIR/scripts/compliance/run_dsar_legal_hold_deep_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/compliance_dsar_legal_hold_contract_lane.json"

test_harness_require_executable "$CONTRACT_LANE" "expected DSAR legal-hold contract lane script to be executable"

if ! grep -q 'run_manifest_lane.sh' "$CONTRACT_LANE"; then
  echo "expected DSAR legal-hold contract lane wrapper to delegate via manifest runner" >&2
  exit 1
fi

test_harness_require_executable "$DISPATCHER" "expected shared non-Kolme dispatcher to be executable"

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected DSAR legal-hold contract lane wrapper to resolve DSAR manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q '"wrapper_name": "run_dsar_legal_hold_contract_lane.sh"' "$MANIFEST"; then
  echo "expected DSAR manifest wrapper_name metadata marker" >&2
  exit 1
fi
if ! grep -q '"phase": "contract"' "$MANIFEST"; then
  echo "expected DSAR manifest phase metadata marker" >&2
  exit 1
fi

test_harness_require_executable "$SHARED_CONTRACT" "expected shared DSAR legal-hold contract lane implementation to be executable"

test_harness_require_file "$MANIFEST" "expected DSAR legal-hold contract lane manifest to exist"

test_harness_require_executable "$DEEP_LANE" "expected DSAR legal-hold deep lane script to be executable"

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "dsar legal-hold contract lane tests passed."; then
  echo "expected DSAR legal-hold contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_dsar_legal_hold_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected deep lane script to invoke DSAR contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "dsar-legal-hold-report.json" "$DEEP_LANE"; then
  echo "expected deep lane script to emit DSAR legal-hold report artifact" >&2
  exit 1
fi

if ! grep -q "generate_dsar_legal_hold_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected shared DSAR contract lane implementation to execute DSAR bundle generator" >&2
  exit 1
fi

if ! grep -q "dsar_legal_hold_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected DSAR contract-lane manifest to dispatch to shared implementation" >&2
  exit 1
fi

if ! grep -q "check_dsar_legal_hold_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected shared DSAR contract lane implementation to execute DSAR policy checker" >&2
  exit 1
fi

if ! grep -q "from framework.contract_lane_helpers import" "$SHARED_CONTRACT"; then
  echo "expected shared DSAR contract-lane implementation to import framework lane helper utilities" >&2
  exit 1
fi

echo "dsar legal-hold contract lane script tests passed."

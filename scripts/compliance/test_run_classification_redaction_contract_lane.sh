#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_SCRIPT="$ROOT_DIR/scripts/compliance/run_classification_redaction_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/compliance/classification_redaction_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/compliance_classification_redaction_contract_lane.json"
LANE_SCRIPT="$ROOT_DIR/scripts/compliance/run_classification_redaction_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/compliance/check_classification_redaction_policy.sh"

if [ ! -x "$CONTRACT_SCRIPT" ]; then
  echo "expected classification/redaction contract lane script to be executable" >&2
  exit 1
fi
if ! grep -q 'run_manifest_lane.sh' "$CONTRACT_SCRIPT"; then
  echo "expected classification/redaction contract lane wrapper to delegate via manifest runner" >&2
  exit 1
fi
if ! grep -q 'compliance_classification_redaction_contract_lane.json' "$CONTRACT_SCRIPT"; then
  echo "expected classification/redaction contract lane wrapper to reference classification manifest" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected shared classification/redaction contract lane implementation to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected classification/redaction contract lane manifest to exist" >&2
  exit 1
fi
if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected classification/redaction lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected classification/redaction policy checker script to be executable" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$CONTRACT_SCRIPT" >"$tmp_out"
if ! grep -q "classification/redaction compliance contract lane tests passed." "$tmp_out"; then
  echo "expected classification/redaction contract lane success marker" >&2
  exit 1
fi

if ! grep -q "KAMN_CLASSIFICATION_REDACTION_CONTRACT_MAX_SECONDS" "$SHARED_CONTRACT"; then
  echo "expected classification/redaction contract lane implementation runtime guard env marker" >&2
  exit 1
fi
if ! grep -q "KAMN_CLASSIFICATION_REDACTION_FORCE_DOCS_CONTRACT_MISSING" "$SHARED_CONTRACT"; then
  echo "expected classification/redaction contract lane implementation forced docs-drift path" >&2
  exit 1
fi
if ! grep -q "reason_key mismatch" "$SHARED_CONTRACT"; then
  echo "expected classification/redaction contract lane implementation to enforce reason_key drift failures" >&2
  exit 1
fi
if ! grep -q "classification_redaction_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected classification/redaction manifest to dispatch to shared implementation" >&2
  exit 1
fi

echo "classification/redaction contract lane script tests passed."

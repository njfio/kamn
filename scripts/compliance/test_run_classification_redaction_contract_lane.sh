#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_SCRIPT="$ROOT_DIR/scripts/compliance/run_classification_redaction_contract_lane.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/compliance/run_classification_redaction_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/compliance/check_classification_redaction_policy.sh"

if [ ! -x "$CONTRACT_SCRIPT" ]; then
  echo "expected classification/redaction contract lane script to be executable" >&2
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

if ! grep -q "KAMN_CLASSIFICATION_REDACTION_CONTRACT_MAX_SECONDS" "$CONTRACT_SCRIPT"; then
  echo "expected classification/redaction contract lane runtime guard env marker" >&2
  exit 1
fi
if ! grep -q "KAMN_CLASSIFICATION_REDACTION_FORCE_DOCS_CONTRACT_MISSING" "$CONTRACT_SCRIPT"; then
  echo "expected classification/redaction contract lane forced docs-drift path" >&2
  exit 1
fi
if ! grep -q "reason_key mismatch" "$CONTRACT_SCRIPT"; then
  echo "expected classification/redaction contract lane to enforce reason_key drift failures" >&2
  exit 1
fi

echo "classification/redaction contract lane script tests passed."

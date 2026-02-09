#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/compliance/check_dsar_legal_hold_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BUNDLE_FILE="$TMP_DIR/dsar-contract.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --request-id "dsar-export-contract-001" \
    --subject-did "did:kamn:subject-contract" \
    --request-type EXPORT \
    --legal-hold-active false \
    --retention-expired true \
    --evidence-complete true \
    --approval-recorded true \
    --tamper-check PASS \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected DSAR contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected DSAR contract lane policy check decision to be GO" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  echo "expected DSAR contract lane to report no failed checks" >&2
  exit 1
fi

echo "dsar legal-hold contract lane tests passed."


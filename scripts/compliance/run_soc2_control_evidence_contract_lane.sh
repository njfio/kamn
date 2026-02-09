#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/compliance/generate_soc2_control_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/compliance/check_soc2_control_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BUNDLE_FILE="$TMP_DIR/soc2-control-contract.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --control-id "CC6.1" \
    --audit-period-start "2026-01-01" \
    --audit-period-end "2026-01-31" \
    --collector-did "did:kamn:auditor-contract" \
    --evidence-uri "s3://kamn-audit/soc2/cc6_1/contract/evidence.json" \
    --evidence-sha256 "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --tamper-check PASS \
    --completeness-check PASS \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected SOC2 contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected SOC2 contract lane policy check decision to be GO" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  echo "expected SOC2 contract lane to report no failed checks" >&2
  exit 1
fi

echo "soc2 control evidence contract lane tests passed."


#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_reputation_dispute_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/reputation/check_reputation_dispute_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BUNDLE_FILE="$TMP_DIR/reputation-dispute-contract.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --dispute-id "dispute-contract-001" \
    --subject-did "did:kamn:agent-contract-001" \
    --reviewer-did "did:kamn:reviewer-contract-001" \
    --dispute-reason-code "QUALITY" \
    --evidence-uri "s3://kamn-audit/reputation/dispute-contract-001.json" \
    --evidence-sha256 "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --evidence-hash-verified "PASS" \
    --original-trust-score 620 \
    --proposed-trust-score 570 \
    --max-adjustment-points 90 \
    --policy-window-open true \
    --approval-recorded true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected reputation dispute contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected reputation dispute contract lane policy decision to be GO" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  echo "expected reputation dispute contract lane to report no failed checks" >&2
  exit 1
fi

echo "reputation dispute contract lane tests passed."

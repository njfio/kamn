#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/governance/generate_stake_slash_risk_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/governance/check_stake_slash_risk_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BUNDLE_FILE="$TMP_DIR/stake-slash-contract.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --proposal-id "gov-risk-contract-001" \
    --simulation-hash "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --stake-at-risk-bps 120 \
    --max-stake-at-risk-bps 300 \
    --slash-probability-bps 40 \
    --max-slash-probability-bps 150 \
    --validator-churn-bps 60 \
    --max-validator-churn-bps 180 \
    --quorum-safety-margin-bps 220 \
    --min-quorum-safety-margin-bps 150 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected stake/slash contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected stake/slash contract lane policy decision to be GO" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  echo "expected stake/slash contract lane to report no failed checks" >&2
  exit 1
fi

echo "stake/slash risk contract lane tests passed."


#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/treasury/generate_treasury_disbursement_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/treasury/check_treasury_disbursement_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$ROOT_DIR"

BUNDLE_FILE="$TMP_DIR/treasury-disbursement-go.json"
start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --disbursement-id "disbursement-contract-2026-02-09" \
    --treasury-account-id "treasury-main-001" \
    --destination-account-id "ops-wallet-001" \
    --asset-symbol "KAMN" \
    --disbursement-amount 250000 \
    --daily-limit-amount 500000 \
    --required-approvals 2 \
    --received-approvals 2 \
    --approval-quorum-hash "sha256:approval-contract-2026-02-09" \
    --policy-window-open true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected treasury disbursement contract lane decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected treasury disbursement policy check decision to be GO" >&2
  exit 1
fi

cargo test -p kamn-core --test release_gonogo_checklist_docs >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 90 ]; then
  echo "treasury disbursement contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "treasury disbursement contract lane tests passed."

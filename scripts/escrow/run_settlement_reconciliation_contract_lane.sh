#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/escrow/check_settlement_reconciliation_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$ROOT_DIR"

BUNDLE_FILE="$TMP_DIR/settlement-evidence-go.json"
start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --escrow-id "escrow-contract-2026-02-09" \
    --settlement-outcome RELEASED \
    --receipt-id "receipt-contract-001" \
    --receipt-finality FINAL \
    --expected-release-amount 120 \
    --expected-refund-amount 0 \
    --observed-release-amount 120 \
    --observed-refund-amount 0 \
    --ledger-reference-id "ledger-entry-contract-001" \
    --timeout-elapsed false \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected settlement reconciliation contract lane decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected settlement reconciliation policy check decision to be GO" >&2
  exit 1
fi

cargo test -p kamn-core --test escrow_lifecycle >/dev/null
cargo test -p kamn-core --test escrow_lifecycle_docs >/dev/null
cargo test -p kamn-core --test release_gonogo_checklist_docs >/dev/null
cargo test -p kamn-core --test audit_export_interfaces_docs >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 90 ]; then
  echo "settlement reconciliation contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "settlement reconciliation contract lane tests passed."

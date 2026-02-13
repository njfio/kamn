#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/cutover/generate_cutover_rollback_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/cutover/check_cutover_rollback_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

bundle_file="$TMP_DIR/cutover-rollback-contract.json"

generator_output="$({
  bash "$GENERATOR" \
    --output-file "$bundle_file" \
    --cutover-manifest-id "cutover-mainnet-contract-2026-02-09" \
    --rollback-trigger-status CLEAR \
    --checkpoint-state READY \
    --failed-checkpoint-id "" \
    --rollback-target-hash "state-hash-contract" \
    --post-rollback-hash "state-hash-contract" \
    --evidence-complete true \
    --ci-fast-gate PASS
})"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected rollback contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected rollback contract lane policy decision to be GO" >&2
  exit 1
fi

echo "cutover rollback contract lane tests passed."

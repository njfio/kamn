#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_gonogo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_gonogo_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BUNDLE_FILE="$TMP_DIR/gonogo-contract.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --release-candidate "v1.0.0-contract" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:contract" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected contract lane policy check decision to be GO" >&2
  exit 1
fi

echo "go/no-go evidence contract lane tests passed."

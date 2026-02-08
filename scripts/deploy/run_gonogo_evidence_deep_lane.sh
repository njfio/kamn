#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_gonogo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_gonogo_evidence_policy.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/deploy/run_gonogo_evidence_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

bash "$CONTRACT_LANE"

NO_GO_BUNDLE="$TMP_DIR/gonogo-deep-no-go.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$NO_GO_BUNDLE" \
    --release-candidate "v1.0.0-deep" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:deep" \
    --ci-fast-gate PASS \
    --ci-deep-lane FAIL \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 1
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected deep-lane failure scenario decision to be NO-GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$NO_GO_BUNDLE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected deep-lane policy check decision to be NO-GO" >&2
  exit 1
fi

echo "go/no-go evidence deep lane tests passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/token/generate_token_launch_handoff_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/token/check_token_launch_handoff_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$ROOT_DIR"

BUNDLE_FILE="$TMP_DIR/token-launch-handoff-go.json"
start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --token-symbol "KAMN" \
    --configured-total-supply 1000000000 \
    --expected-total-supply 1000000000 \
    --configured-allocation-sum 1000000000 \
    --expected-allocation-sum 1000000000 \
    --allocation-bucket-count 5 \
    --expected-bucket-count 5 \
    --genesis-hash "sha256:token-launch-handoff-go-2026-02-09" \
    --required-approvals 2 \
    --received-approvals 2 \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected token launch handoff contract lane decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected token launch handoff policy check decision to be GO" >&2
  exit 1
fi

cargo test -p kamn-core --test token_config >/dev/null
cargo test -p kamn-core --test token_config_docs >/dev/null
cargo test -p kamn-core --test release_gonogo_checklist_docs >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 90 ]; then
  echo "token launch handoff contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "token launch handoff contract lane tests passed."

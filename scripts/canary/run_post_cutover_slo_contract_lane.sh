#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/canary/generate_post_cutover_slo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/canary/check_post_cutover_slo_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

bundle_file="$TMP_DIR/post-cutover-slo-contract.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$bundle_file" \
    --window-minutes 15 \
    --p95-latency-ms 140 \
    --max-p95-latency-ms 200 \
    --error-rate-bps 18 \
    --max-error-rate-bps 25 \
    --delivery-success-bps 9992 \
    --min-delivery-success-bps 9950 \
    --snapshot-age-seconds 30 \
    --max-snapshot-age-seconds 120 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected post-cutover SLO contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected post-cutover SLO contract lane policy decision to be GO" >&2
  exit 1
fi

echo "post-cutover SLO contract lane tests passed."

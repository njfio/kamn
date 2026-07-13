#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_dr_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_release_slo_gates.sh"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
CONTRACT_MANIFEST="$ROOT_DIR/scripts/framework/manifests/deploy_dr_evidence_contract_lane.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

DR_EVIDENCE_REPORT="$TMP_DIR/dr-evidence-report.json"

bash "$MANIFEST_RUNNER" --manifest "$CONTRACT_MANIFEST" --phase contract

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$DR_EVIDENCE_REPORT" \
    --drill-id "dr-deep-2026-02-08" \
    --recovery-rto-seconds 390 \
    --recovery-rpo-seconds 130 \
    --max-rto-seconds 300 \
    --max-rpo-seconds 120 \
    --rollback-restored true \
    --evidence-complete false \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected DR deep-lane failure scenario decision to be NO-GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$DR_EVIDENCE_REPORT")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected DR deep-lane policy check decision to be NO-GO" >&2
  exit 1
fi

echo "dr evidence deep lane tests passed."

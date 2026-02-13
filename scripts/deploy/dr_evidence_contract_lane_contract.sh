#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_dr_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_release_slo_gates.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

DR_EVIDENCE_REPORT="$TMP_DIR/dr-evidence-report.json"

generator_output="$({
  bash "$GENERATOR" \
    --output-file "$DR_EVIDENCE_REPORT" \
    --drill-id "dr-contract-2026-02-08" \
    --recovery-rto-seconds 210 \
    --recovery-rpo-seconds 75 \
    --max-rto-seconds 300 \
    --max-rpo-seconds 120 \
    --rollback-restored true \
    --evidence-complete true \
    --ci-fast-gate PASS
})"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected DR evidence contract lane decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$DR_EVIDENCE_REPORT")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected DR evidence policy check decision to be GO" >&2
  exit 1
fi

echo "dr evidence contract lane tests passed."

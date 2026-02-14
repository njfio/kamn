#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_staging_rehearsal_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_staging_rehearsal_policy.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/deploy/run_staging_rehearsal_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

STAGING_REHEARSAL_REPORT="$TMP_DIR/staging-rehearsal-report.json"

bash "$CONTRACT_LANE"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$STAGING_REHEARSAL_REPORT" \
    --release-candidate "v1.1.0-deep" \
    --deploy-status PASS \
    --rollback-status PASS \
    --rollback-target-hash "state-hash-stable" \
    --post-rollback-hash "state-hash-stable" \
    --recovery-time-seconds 1500 \
    --max-allowed-recovery-time-seconds 900 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected deep-lane MTTR-bound breach scenario decision to be NO-GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$STAGING_REHEARSAL_REPORT")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected deep-lane policy check decision to be NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^mttr_within_bound=false$"; then
  echo "expected deep-lane policy check to report out-of-bound MTTR evidence" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^staged_rehearsal_signoff_status=fail-closed$"; then
  echo "expected deep-lane policy check to report fail-closed staged rehearsal signoff status" >&2
  exit 1
fi

echo "staging rehearsal deep lane tests passed."

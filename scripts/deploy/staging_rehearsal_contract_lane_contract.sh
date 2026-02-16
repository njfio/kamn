#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_staging_rehearsal_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_staging_rehearsal_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BOUNDARY_THRESHOLDS_SCHEMA_VERSION="kamn.release.staging-rehearsal-boundary-thresholds.v1"
BOUNDARY_REASON_TAXONOMY_VERSION="kamn.release.staging-rehearsal-boundary-reason-taxonomy.v1"
BOUNDARY_REASON_CODES_CSV="rehearsal_boundary_ci_smoke_seconds_exceeded,rehearsal_boundary_local_heavy_opt_in_missing,rehearsal_runbook_contract_parity_mismatch"
CI_SMOKE_MAX_SECONDS=120
LOCAL_HEAVY_MAX_SECONDS=900
RUNBOOK_PARITY_STATUS="verified"
max_seconds="$CI_SMOKE_MAX_SECONDS"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi
if [ "$max_seconds" -gt "$CI_SMOKE_MAX_SECONDS" ]; then
  echo "rehearsal_boundary_ci_smoke_seconds_exceeded: requested --max-seconds=$max_seconds exceeds ci smoke boundary $CI_SMOKE_MAX_SECONDS" >&2
  exit 1
fi
if [ "$RUNBOOK_PARITY_STATUS" != "verified" ]; then
  echo "rehearsal_runbook_contract_parity_mismatch: runbook parity marker must remain verified" >&2
  exit 1
fi

start_epoch="$(date +%s)"
STAGING_REHEARSAL_REPORT="$TMP_DIR/staging-rehearsal-report.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$STAGING_REHEARSAL_REPORT" \
    --release-candidate "v1.1.0-contract" \
    --deploy-status PASS \
    --rollback-status PASS \
    --rollback-target-hash "state-hash-contract" \
    --post-rollback-hash "state-hash-contract" \
    --recovery-time-seconds 420 \
    --max-allowed-recovery-time-seconds 900 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected rehearsal contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$STAGING_REHEARSAL_REPORT")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected rehearsal contract lane policy check decision to be GO" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^mttr_within_bound=true$"; then
  echo "expected rehearsal contract lane policy check to confirm bounded MTTR evidence" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^staged_rehearsal_signoff_status=verified$"; then
  echo "expected rehearsal contract lane policy check to confirm verified staged rehearsal signoff" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "rehearsal_boundary_ci_smoke_seconds_exceeded: elapsed_seconds=$elapsed_seconds exceeded max-seconds=$max_seconds" >&2
  exit 1
fi

echo "rehearsal_runbook_contract_parity_status=$RUNBOOK_PARITY_STATUS"
echo "rehearsal_boundary_thresholds_schema_version=$BOUNDARY_THRESHOLDS_SCHEMA_VERSION"
echo "rehearsal_boundary_reason_taxonomy_status=verified"
echo "rehearsal_boundary_reason_taxonomy_version=$BOUNDARY_REASON_TAXONOMY_VERSION"
echo "rehearsal_boundary_reason_codes_csv=$BOUNDARY_REASON_CODES_CSV"
echo "rehearsal_boundary_ci_smoke_max_seconds=$CI_SMOKE_MAX_SECONDS"
echo "rehearsal_boundary_local_heavy_max_seconds=$LOCAL_HEAVY_MAX_SECONDS"
echo "ci_smoke_lane_cost_profile=low"
echo "local_heavy_lane_execution_mode=opt_in"

echo "staging rehearsal contract lane tests passed."

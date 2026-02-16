#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_staging_rehearsal_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_staging_rehearsal_policy.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/deploy/run_staging_rehearsal_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

LOCAL_HEAVY_OPT_IN_ENV="KAMN_STAGING_REHEARSAL_LOCAL_HEAVY_OPT_IN"
BOUNDARY_REASON_TAXONOMY_VERSION="kamn.release.staging-rehearsal-boundary-reason-taxonomy.v1"
BOUNDARY_REASON_CODES_CSV="rehearsal_boundary_ci_smoke_seconds_exceeded,rehearsal_boundary_local_heavy_opt_in_missing,rehearsal_runbook_contract_parity_mismatch"
LOCAL_HEAVY_MAX_SECONDS=900
max_seconds="$LOCAL_HEAVY_MAX_SECONDS"

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
if [ "$max_seconds" -gt "$LOCAL_HEAVY_MAX_SECONDS" ]; then
  echo "max-seconds exceeds local-heavy boundary: rehearsal_boundary_local_heavy_max_seconds=$LOCAL_HEAVY_MAX_SECONDS" >&2
  exit 1
fi
if [ "${!LOCAL_HEAVY_OPT_IN_ENV:-0}" != "1" ]; then
  echo "rehearsal_boundary_local_heavy_opt_in_missing: run mode requires explicit local-only opt-in via ${LOCAL_HEAVY_OPT_IN_ENV}=1" >&2
  exit 1
fi

STAGING_REHEARSAL_REPORT="$TMP_DIR/staging-rehearsal-report.json"

bash "$CONTRACT_LANE" --max-seconds 120

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

echo "rehearsal_boundary_reason_taxonomy_status=verified"
echo "rehearsal_boundary_reason_taxonomy_version=$BOUNDARY_REASON_TAXONOMY_VERSION"
echo "rehearsal_boundary_reason_codes_csv=$BOUNDARY_REASON_CODES_CSV"
echo "rehearsal_boundary_local_heavy_opt_in_status=verified"
echo "rehearsal_boundary_local_heavy_max_seconds=$LOCAL_HEAVY_MAX_SECONDS"

echo "staging rehearsal deep lane tests passed."

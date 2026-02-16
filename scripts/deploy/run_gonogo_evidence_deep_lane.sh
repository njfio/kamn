#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_gonogo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_gonogo_evidence_policy.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/deploy/run_gonogo_evidence_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

INCIDENT_GONOGO_BOUNDARY_REASON_TAXONOMY_VERSION="kamn.release.gonogo-incident-boundary-reason-taxonomy.v1"
INCIDENT_GONOGO_BOUNDARY_REASON_CODES_CSV="incident_gonogo_ci_smoke_seconds_exceeded,incident_gonogo_local_heavy_seconds_exceeded,incident_gonogo_local_heavy_opt_in_missing,incident_gonogo_evidence_convergence_mismatch"
INCIDENT_GONOGO_CI_SMOKE_MAX_SECONDS=120
INCIDENT_GONOGO_LOCAL_HEAVY_MAX_SECONDS=900
LOCAL_HEAVY_OPT_IN_ENV="KAMN_GONOGO_GATE_LOCAL_OPT_IN"

MAX_SECONDS="$INCIDENT_GONOGO_LOCAL_HEAVY_MAX_SECONDS"
while (($# > 0)); do
  case "$1" in
    --max-seconds)
      if (($# < 2)); then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi
if [ "$MAX_SECONDS" -lt 1 ]; then
  echo "max-seconds must be >= 1" >&2
  exit 1
fi
if [ "${!LOCAL_HEAVY_OPT_IN_ENV:-}" != "1" ]; then
  echo "incident_gonogo_local_heavy_opt_in_missing" >&2
  exit 1
fi
if [ "$MAX_SECONDS" -gt "$INCIDENT_GONOGO_LOCAL_HEAVY_MAX_SECONDS" ]; then
  echo "incident_gonogo_local_heavy_seconds_exceeded" >&2
  exit 1
fi

bash "$CONTRACT_LANE" --max-seconds "$INCIDENT_GONOGO_CI_SMOKE_MAX_SECONDS"

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

echo "incident_gonogo_boundary_reason_taxonomy_status=verified"
echo "incident_gonogo_boundary_reason_taxonomy_version=$INCIDENT_GONOGO_BOUNDARY_REASON_TAXONOMY_VERSION"
echo "incident_gonogo_boundary_reason_codes_csv=$INCIDENT_GONOGO_BOUNDARY_REASON_CODES_CSV"
echo "incident_gonogo_ci_smoke_max_seconds=$INCIDENT_GONOGO_CI_SMOKE_MAX_SECONDS"
echo "incident_gonogo_local_heavy_max_seconds=$INCIDENT_GONOGO_LOCAL_HEAVY_MAX_SECONDS"
echo "local_heavy_lane_execution_mode=opt_in"
echo "go/no-go evidence deep lane tests passed."

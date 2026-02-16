#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_gonogo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_gonogo_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

INCIDENT_GONOGO_BOUNDARY_REASON_TAXONOMY_VERSION="kamn.release.gonogo-incident-boundary-reason-taxonomy.v1"
INCIDENT_GONOGO_BOUNDARY_REASON_CODES_CSV="incident_gonogo_ci_smoke_seconds_exceeded,incident_gonogo_local_heavy_seconds_exceeded,incident_gonogo_local_heavy_opt_in_missing,incident_gonogo_evidence_convergence_mismatch"
INCIDENT_GONOGO_CI_SMOKE_MAX_SECONDS=120
INCIDENT_GONOGO_LOCAL_HEAVY_MAX_SECONDS=900

MAX_SECONDS="$INCIDENT_GONOGO_CI_SMOKE_MAX_SECONDS"
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
if [ "$MAX_SECONDS" -gt "$INCIDENT_GONOGO_CI_SMOKE_MAX_SECONDS" ]; then
  echo "incident_gonogo_ci_smoke_seconds_exceeded" >&2
  exit 1
fi

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

tls_evidence_report="$TMP_DIR/tls-evidence-report.json"
cat >"$tls_evidence_report" <<'JSON'
{
  "schema_version": "kamn.ci.kamn-core-live-https-dependency-posture-report.v1",
  "reason_taxonomy_version": "kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1",
  "status": "pass",
  "reason_codes": [
    "none"
  ],
  "reason_codes_csv": "none",
  "reason_codes_value": "none"
}
JSON

tls_bundle_file="$TMP_DIR/gonogo-contract-tls.json"
tls_generator_output="$(
  bash "$GENERATOR" \
    --output-file "$tls_bundle_file" \
    --release-candidate "v1.0.0-contract-tls" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:contract-tls" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --tls-evidence-report-file "$tls_evidence_report" \
    --tls-evidence-max-age-seconds 1800
)"
if ! printf '%s\n' "$tls_generator_output" | grep -q "^tls_evidence_gate_final_decision=GO$"; then
  echo "expected tls evidence gate decision marker from generator output" >&2
  exit 1
fi
if ! printf '%s\n' "$tls_generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected tls evidence contract lane bundle decision to be GO" >&2
  exit 1
fi

tls_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$tls_bundle_file")"
if ! printf '%s\n' "$tls_policy_output" | grep -q "^tls_evidence_gate_final_decision=GO$"; then
  echo "expected tls evidence gate decision marker from policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$tls_policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected tls evidence contract lane policy check decision to be GO" >&2
  exit 1
fi

milestone_preflight_summary="$TMP_DIR/milestone-preflight-summary.json"
milestone_preflight_policy="$TMP_DIR/milestone-preflight-policy.json"
milestone_live_bundle_summary="$TMP_DIR/milestone-live-bundle-summary.json"
milestone_live_bundle_policy="$TMP_DIR/milestone-live-bundle-policy.json"
milestone_gate_report="$TMP_DIR/milestone-go-no-go-gate-report.json"
milestone_bundle_file="$TMP_DIR/gonogo-milestone-contract.json"

cat >"$milestone_preflight_summary" <<'JSON'
{
  "schema_version": "kamn.kolme.local-live-deployment-preflight-summary.v1",
  "status": "ok",
  "contracts": {
    "ci_fast_gate_scope": "ci-fast-gate"
  }
}
JSON

cat >"$milestone_preflight_policy" <<'JSON'
{
  "schema_version": "kamn.kolme.local-live-deployment-preflight-policy-report.v1",
  "final_decision": "GO"
}
JSON

cat >"$milestone_live_bundle_summary" <<'JSON'
{
  "schema_version": "kamn.kolme.local-live-node-validation-bundle-summary.v1",
  "status": "ok",
  "rollback_evidence_file": "/tmp/rollback.json",
  "recovery_evidence_file": "/tmp/recovery.json",
  "artifact_paths": [
    "/tmp/rollback.json",
    "/tmp/recovery.json"
  ],
  "contracts": {
    "ci_fast_gate_scope": "local-only",
    "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
    "rollback_recovery_artifact_lineage_required": true
  }
}
JSON

cat >"$milestone_live_bundle_policy" <<'JSON'
{
  "schema_version": "kamn.kolme.local-live-node-validation-bundle-policy-report.v1",
  "final_decision": "GO"
}
JSON

cat >"$milestone_gate_report" <<'JSON'
{
  "schema_version": "kamn.runtime.go-no-go-gate-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "combined_reason_taxonomy_version": "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1",
  "combined_transport_reason_codes": [
    "fork_choice_stale_block_height"
  ],
  "combined_kolme_runtime_reason_code": "not_run",
  "kolme_runtime_commit_failure_taxonomy_version": "v1",
  "kolme_fixture_profile": "real-node-non-synthetic-v1",
  "kolme_fixture_profile_version": "v1",
  "kolme_fixture_profile_status": "planned",
  "combined_lane_marker_contract_status": "verified"
}
JSON

milestone_generator_output="$(
  bash "$GENERATOR" \
    --output-file "$milestone_bundle_file" \
    --release-candidate "v1.0.0-contract-milestone" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:contract-milestone" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --deployment-preflight-summary-file "$milestone_preflight_summary" \
    --deployment-preflight-policy-file "$milestone_preflight_policy" \
    --live-node-validation-summary-file "$milestone_live_bundle_summary" \
    --live-node-validation-policy-file "$milestone_live_bundle_policy" \
    --go-no-go-gate-report-file "$milestone_gate_report"
)"

if ! printf '%s\n' "$milestone_generator_output" | grep -q "^milestone_review_final_decision=GO$"; then
  echo "expected milestone aggregate decision marker from generator output" >&2
  exit 1
fi

milestone_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$milestone_bundle_file")"
if ! printf '%s\n' "$milestone_policy_output" | grep -q "^milestone_review_final_decision=GO$"; then
  echo "expected milestone aggregate decision marker from policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$milestone_policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected milestone aggregate policy check decision to be GO" >&2
  exit 1
fi

runbook_marker_missing_doc="$TMP_DIR/runbook-marker-missing.md"
cat >"$runbook_marker_missing_doc" <<'TXT'
# Upgrade Rollback Runbook
Marker intentionally incomplete for contract lane regression coverage.
TXT

milestone_missing_runbook_bundle_file="$TMP_DIR/gonogo-milestone-contract-missing-runbook.json"
milestone_missing_runbook_output="$(
  KAMN_GONOGO_RUNBOOK_DOC_FILE="$runbook_marker_missing_doc" \
    bash "$GENERATOR" \
      --output-file "$milestone_missing_runbook_bundle_file" \
      --release-candidate "v1.0.0-contract-missing-runbook" \
      --schema-target-version "1.0.0" \
      --runtime-image-digest "sha256:contract-missing-runbook" \
      --ci-fast-gate PASS \
      --ci-deep-lane PASS \
      --rollback-precheck PASS \
      --rollback-trigger-status CLEAR \
      --required-approvals 2 \
      --received-approvals 2 \
      --deployment-preflight-summary-file "$milestone_preflight_summary" \
      --deployment-preflight-policy-file "$milestone_preflight_policy" \
      --live-node-validation-summary-file "$milestone_live_bundle_summary" \
      --live-node-validation-policy-file "$milestone_live_bundle_policy" \
      --go-no-go-gate-report-file "$milestone_gate_report"
)"

if ! printf '%s\n' "$milestone_missing_runbook_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected milestone aggregate generator decision to fail closed when runbook markers are missing" >&2
  exit 1
fi

milestone_missing_runbook_policy_output="$(
  KAMN_GONOGO_RUNBOOK_DOC_FILE="$runbook_marker_missing_doc" \
    bash "$POLICY_CHECKER" \
      --bundle-file "$milestone_missing_runbook_bundle_file"
)"
if ! printf '%s\n' "$milestone_missing_runbook_policy_output" | grep -q "^milestone_review_final_decision=NO-GO$"; then
  echo "expected milestone aggregate policy decision marker to fail closed when runbook markers are missing" >&2
  exit 1
fi
if ! printf '%s\n' "$milestone_missing_runbook_policy_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected milestone aggregate policy check decision to fail closed when runbook markers are missing" >&2
  exit 1
fi

audit_integrity_report="$TMP_DIR/audit-integrity-policy-report.json"
cat >"$audit_integrity_report" <<'JSON'
{
  "schema_version": "kamn.runtime.sqlite-crash-recovery-live-policy-report.v1",
  "status": "ok",
  "final_decision": "GO",
  "sqlite_crash_recovery_policy_status": "verified",
  "durability_governance_reason_taxonomy_version": "kamn.runtime.durability-governance-reason-taxonomy.v1",
  "durability_governance_reason_codes_csv": "crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"
}
JSON

audit_bundle_file="$TMP_DIR/gonogo-audit-integrity-contract.json"
audit_generator_output="$(
  bash "$GENERATOR" \
    --output-file "$audit_bundle_file" \
    --release-candidate "v1.0.0-contract-audit" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:contract-audit" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --audit-integrity-report-file "$audit_integrity_report" \
    --audit-integrity-max-age-seconds 1800
)"
if ! printf '%s\n' "$audit_generator_output" | grep -q "^audit_integrity_gate_final_decision=GO$"; then
  echo "expected audit-integrity gate decision marker from generator output" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected audit-integrity contract lane bundle decision to be GO" >&2
  exit 1
fi

audit_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$audit_bundle_file")"
if ! printf '%s\n' "$audit_policy_output" | grep -q "^audit_integrity_gate_final_decision=GO$"; then
  echo "expected audit-integrity gate decision marker from policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected audit-integrity contract lane policy check decision to be GO" >&2
  exit 1
fi

slo_policy_report="$TMP_DIR/slo-policy-report.json"
cat >"$slo_policy_report" <<'JSON'
{
  "schema_version": "kamn.deploy.slo-rollback-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "reason_key": "deployment_slo_rollback_reason_codes:GO:v1",
  "reason_codes": []
}
JSON

slo_policy_bundle_file="$TMP_DIR/gonogo-slo-policy-contract.json"
slo_policy_generator_output="$(
  bash "$GENERATOR" \
    --output-file "$slo_policy_bundle_file" \
    --release-candidate "v1.0.0-contract-slo-policy" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:contract-slo-policy" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --slo-policy-report-file "$slo_policy_report" \
    --slo-policy-max-age-seconds 1800
)"
if ! printf '%s\n' "$slo_policy_generator_output" | grep -q "^slo_policy_gate_final_decision=GO$"; then
  echo "expected SLO policy gate decision marker from generator output" >&2
  exit 1
fi
if ! printf '%s\n' "$slo_policy_generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected SLO policy contract lane bundle decision to be GO" >&2
  exit 1
fi

slo_policy_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$slo_policy_bundle_file")"
if ! printf '%s\n' "$slo_policy_policy_output" | grep -q "^slo_policy_gate_final_decision=GO$"; then
  echo "expected SLO policy gate decision marker from policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$slo_policy_policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected SLO policy contract lane policy check decision to be GO" >&2
  exit 1
fi

incident_readiness_report="$TMP_DIR/incident-readiness-report.json"
bash "$ROOT_DIR/scripts/deploy/generate_staging_rehearsal_bundle.sh" \
  --output-file "$incident_readiness_report" \
  --release-candidate "v1.0.0-contract-incident-readiness" \
  --deploy-status PASS \
  --rollback-status PASS \
  --rollback-target-hash "state-hash-contract-incident-ready" \
  --post-rollback-hash "state-hash-contract-incident-ready" \
  --recovery-time-seconds 420 \
  --max-allowed-recovery-time-seconds 900 \
  --evidence-complete true \
  --ci-fast-gate PASS >/dev/null

incident_readiness_bundle_file="$TMP_DIR/gonogo-incident-readiness-contract.json"
incident_readiness_generator_output="$(
  bash "$GENERATOR" \
    --output-file "$incident_readiness_bundle_file" \
    --release-candidate "v1.0.0-contract-incident-readiness-gate" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:contract-incident-readiness" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --incident-readiness-report-file "$incident_readiness_report" \
    --incident-readiness-max-age-seconds 1800
)"
if ! printf '%s\n' "$incident_readiness_generator_output" | grep -q "^incident_readiness_gate_final_decision=GO$"; then
  echo "expected incident-readiness gate decision marker from generator output" >&2
  exit 1
fi
if ! printf '%s\n' "$incident_readiness_generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected incident-readiness contract lane bundle decision to be GO" >&2
  exit 1
fi

incident_readiness_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$incident_readiness_bundle_file")"
if ! printf '%s\n' "$incident_readiness_policy_output" | grep -q "^incident_readiness_gate_final_decision=GO$"; then
  echo "expected incident-readiness gate decision marker from policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$incident_readiness_policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected incident-readiness contract lane policy check decision to be GO" >&2
  exit 1
fi

echo "incident_gonogo_boundary_reason_taxonomy_status=verified"
echo "incident_gonogo_boundary_reason_taxonomy_version=$INCIDENT_GONOGO_BOUNDARY_REASON_TAXONOMY_VERSION"
echo "incident_gonogo_boundary_reason_codes_csv=$INCIDENT_GONOGO_BOUNDARY_REASON_CODES_CSV"
echo "incident_gonogo_ci_smoke_max_seconds=$INCIDENT_GONOGO_CI_SMOKE_MAX_SECONDS"
echo "incident_gonogo_local_heavy_max_seconds=$INCIDENT_GONOGO_LOCAL_HEAVY_MAX_SECONDS"
echo "ci_smoke_lane_cost_profile=low"
echo "go/no-go evidence contract lane tests passed."

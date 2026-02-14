#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_gonogo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_gonogo_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

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
  "final_decision": "GO"
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

echo "go/no-go evidence contract lane tests passed."

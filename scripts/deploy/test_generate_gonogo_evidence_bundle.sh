#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_gonogo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_gonogo_evidence_policy.sh"
UPGRADE_LINEAGE_CHECKER="$ROOT_DIR/scripts/deploy/check_upgrade_rehearsal_lineage_policy.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$GENERATOR" ]; then
  echo "expected go/no-go evidence bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected go/no-go evidence policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$UPGRADE_LINEAGE_CHECKER" ]; then
  echo "expected upgrade rehearsal lineage checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/gonogo-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --release-candidate "v1.0.0-rc.1" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:abc123" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2
)"

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected generator to derive GO decision"

python3 - "$go_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
required_markers = {
    "ci_fast_gate",
    "ci_deep_lane",
    "rollback_precheck",
    "rollback_trigger_status",
    "approval_quorum",
    "runtime_image_digest",
}
markers = payload.get("evidence_markers")
if not isinstance(markers, list):
    raise SystemExit("expected go/no-go bundle evidence_markers list")
if set(markers) != required_markers:
    raise SystemExit("expected go/no-go bundle evidence_markers to match required checklist markers")
PY

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected policy check to keep GO decision"

no_go_bundle="$TMP_DIR/gonogo-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --release-candidate "v1.0.0-rc.2" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:def456" \
    --ci-fast-gate PASS \
    --ci-deep-lane FAIL \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 1
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected generator to derive NO-GO decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected policy check to keep NO-GO decision"

tampered_bundle="$TMP_DIR/gonogo-tampered.json"
cp "$no_go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from policy checker" >&2
  exit 1
fi

# Regression: #623
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected regression guard to catch policy decision mismatch" >&2
  exit 1
fi

tampered_missing_evidence_bundle="$TMP_DIR/gonogo-missing-evidence-marker.json"
cp "$go_bundle" "$tampered_missing_evidence_bundle"
python3 - "$tampered_missing_evidence_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["evidence_markers"] = [marker for marker in payload.get("evidence_markers", []) if marker != "rollback_precheck"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_missing_evidence_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_missing_evidence_bundle" 2>&1)"
tampered_missing_evidence_code=$?
set -e

if [ "$tampered_missing_evidence_code" -eq 0 ]; then
  echo "expected missing-evidence-marker bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_missing_evidence_output" | grep -q "missing required evidence markers"; then
  echo "expected explicit missing-required-evidence-markers error from policy checker" >&2
  exit 1
fi

# Milestone-level aggregate lineage coverage for #3247.
milestone_preflight_summary="$TMP_DIR/milestone-preflight-summary.json"
milestone_preflight_policy="$TMP_DIR/milestone-preflight-policy.json"
milestone_live_bundle_summary="$TMP_DIR/milestone-live-bundle-summary.json"
milestone_live_bundle_policy="$TMP_DIR/milestone-live-bundle-policy.json"
milestone_gate_report="$TMP_DIR/milestone-go-no-go-gate-report.json"
milestone_bundle="$TMP_DIR/gonogo-milestone.json"

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
  "final_decision": "GO",
  "rotation_preflight_reason_taxonomy_version": "kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1",
  "rotation_preflight_reason_codes_csv": "signer_key_source_contract_version_mismatch,signer_key_source_invalid,signer_key_source_production_managed_external_required,signer_quorum_minimum_not_met,signer_rotation_epoch_stale,signer_rotation_rehearsal_drift_detected,signer_rotation_promotion_stalled,fallback_signer_secret_present_violation,fallback_signer_secret_checkpoint_reason_mismatch,fallback_signer_secret_remediation_missing,quorum_evidence_missing,quorum_evidence_rotation_metadata_missing,quorum_evidence_rotation_metadata_invalid,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_drift_telemetry_missing,runtime_signer_drift_telemetry_rotation_delta_invalid,runtime_signer_drift_matrix_inputs_invalid,runtime_signer_drift_rotation_fail_threshold_exceeded,runtime_signer_drift_quorum_fail_threshold_exceeded,custody_continuity_bypass_detected",
  "rotation_preflight_reason_codes_value": "none"
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
  "lane_mode": "dry-run",
  "ci_fast_gate_eligible": true,
  "ci_fast_gate_scope": "ci-fast-gate",
  "fast_gate_exclusion_status": "verified",
  "fast_gate_exclusion_reason_code": "go_no_go_gate_run_mode_excluded_from_fast_gate",
  "run_mode_command_status": "dry_run_no_commands_executed",
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

milestone_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$milestone_bundle" \
    --release-candidate "v1.0.0-rc.3" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:milestone" \
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

assert_eq "$(extract_value "$milestone_generate_output" "status")" "generated" "expected milestone bundle generation to succeed"
assert_eq "$(extract_value "$milestone_generate_output" "final_decision")" "GO" "expected milestone bundle decision to remain GO"
assert_eq "$(extract_value "$milestone_generate_output" "live_gonogo_reason_taxonomy_version")" "kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1" "expected deterministic live-go/no-go reason taxonomy marker for milestone aggregate evidence"
assert_eq "$(extract_value "$milestone_generate_output" "live_gonogo_reason_codes_csv")" "none" "expected deterministic live-go/no-go reason csv marker on pass path"

python3 - "$milestone_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
milestone = payload.get("milestone_review_bundle")
if not isinstance(milestone, dict):
    raise SystemExit("expected milestone_review_bundle object in go/no-go evidence bundle")
if milestone.get("schema_version") != "kamn.release.milestone-review-bundle.v1":
    raise SystemExit("expected milestone review bundle schema marker")
if milestone.get("reason_taxonomy_version") != "kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1":
    raise SystemExit("expected milestone review bundle reason taxonomy marker")
if milestone.get("reason_codes_csv") != "none":
    raise SystemExit("expected milestone review bundle reason_codes_csv=none on pass path")
if milestone.get("reason_codes_value") != "none":
    raise SystemExit("expected milestone review bundle reason_codes_value=none on pass path")
if milestone.get("final_decision") != "GO":
    raise SystemExit("expected milestone review bundle final_decision=GO")
if milestone.get("lineage_status") != "verified":
    raise SystemExit("expected milestone review bundle lineage_status=verified")
if milestone.get("reason_codes") != []:
    raise SystemExit("expected empty milestone review reason_codes for valid aggregate evidence")
contracts = milestone.get("contracts")
if not isinstance(contracts, dict):
    raise SystemExit("expected milestone review contracts object")
if contracts.get("linked_artifact_lineage_required") is not True:
    raise SystemExit("expected linked_artifact_lineage_required=true in milestone review contracts")
if contracts.get("live_bundle_runtime_provider_client_required") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected milestone review runtime provider contract marker")
if contracts.get("go_no_go_gate_combined_reason_taxonomy_version_required") != "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1":
    raise SystemExit("expected milestone review combined reason taxonomy contract marker")
if contracts.get("go_no_go_gate_combined_transport_reason_codes_required") != ["fork_choice_stale_block_height"]:
    raise SystemExit("expected milestone review combined transport reason-code contract marker")
if contracts.get("go_no_go_gate_combined_kolme_runtime_reason_codes_allowed") != ["live_runtime_integration_passed", "not_run"]:
    raise SystemExit("expected milestone review allowed combined Kolme reason-code contract marker")
if contracts.get("go_no_go_gate_combined_lane_marker_contract_status_required") != "verified":
    raise SystemExit("expected milestone review combined marker contract status contract marker")
if contracts.get("deployment_preflight_rotation_reason_taxonomy_version_required") != "kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1":
    raise SystemExit("expected milestone review deployment preflight rotation reason taxonomy contract marker")
if contracts.get("go_no_go_gate_ci_local_boundary_contract_required") is not True:
    raise SystemExit("expected milestone review ci/local boundary contract marker")
observed = milestone.get("observed")
if not isinstance(observed, dict):
    raise SystemExit("expected milestone review observed object")
if observed.get("go_no_go_gate_combined_reason_taxonomy_version") != "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1":
    raise SystemExit("expected observed combined reason taxonomy marker")
if observed.get("go_no_go_gate_combined_transport_reason_codes") != ["fork_choice_stale_block_height"]:
    raise SystemExit("expected observed combined transport reason codes marker")
if observed.get("go_no_go_gate_combined_kolme_runtime_reason_code") != "not_run":
    raise SystemExit("expected observed combined Kolme reason code marker")
if observed.get("go_no_go_gate_combined_lane_marker_contract_status") != "verified":
    raise SystemExit("expected observed combined lane marker contract status marker")
if observed.get("deployment_preflight_policy_rotation_reason_taxonomy_version") != "kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1":
    raise SystemExit("expected observed deployment preflight rotation reason taxonomy marker")
if observed.get("go_no_go_gate_lane_mode") != "dry-run":
    raise SystemExit("expected observed go/no-go lane mode boundary marker")
if observed.get("go_no_go_gate_ci_fast_gate_scope") != "ci-fast-gate":
    raise SystemExit("expected observed go/no-go ci fast-gate scope boundary marker")
if observed.get("go_no_go_gate_fast_gate_exclusion_reason_code") != "go_no_go_gate_run_mode_excluded_from_fast_gate":
    raise SystemExit("expected observed go/no-go fast-gate exclusion reason marker")
PY

milestone_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$milestone_bundle")"
assert_eq "$(extract_value "$milestone_policy_output" "status")" "ok" "expected milestone bundle policy check to pass"
assert_eq "$(extract_value "$milestone_policy_output" "final_decision")" "GO" "expected milestone bundle policy to keep GO decision"

milestone_lineage_output="$(
  python3 "$UPGRADE_LINEAGE_CHECKER" \
    --bundle-file "$milestone_bundle" \
    --expected-final-decision GO
)"
assert_eq "$(extract_value "$milestone_lineage_output" "status")" "ok" "expected upgrade lineage checker to pass for valid milestone bundle"
assert_eq "$(extract_value "$milestone_lineage_output" "upgrade_lineage_final_decision")" "GO" "expected upgrade lineage checker GO decision for valid milestone bundle"
assert_eq "$(extract_value "$milestone_lineage_output" "upgrade_lineage_reason_taxonomy_version")" "kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1" "expected deterministic upgrade lineage reason taxonomy marker"
assert_eq "$(extract_value "$milestone_lineage_output" "upgrade_lineage_reason_codes_csv")" "none" "expected deterministic upgrade lineage reason csv marker on pass path"
assert_eq "$(extract_value "$milestone_lineage_output" "promotion_gate_reason_taxonomy_version")" "kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1" "expected deterministic promotion-gate reason taxonomy marker"
assert_eq "$(extract_value "$milestone_lineage_output" "promotion_gate_reason_codes_csv")" "none" "expected deterministic promotion-gate reason csv marker on pass path"
assert_eq "$(extract_value "$milestone_lineage_output" "promotion_gate_reason_codes_value")" "none" "expected deterministic promotion-gate reason value marker on pass path"

integration_preflight_summary="$TMP_DIR/integration-preflight-summary.json"
integration_preflight_policy="$TMP_DIR/integration-preflight-policy.json"
integration_live_bundle_summary="$TMP_DIR/integration-live-bundle-summary.json"
integration_live_bundle_policy="$TMP_DIR/integration-live-bundle-policy.json"
integration_gate_report="$TMP_DIR/integration-go-no-go-gate-report.json"
integration_milestone_bundle="$TMP_DIR/gonogo-milestone-integration.json"

bash "$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh" \
  --mode dry-run \
  --output-json "$integration_preflight_summary" >/dev/null
python3 "$ROOT_DIR/scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py" \
  --report-file "$integration_preflight_summary" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$integration_preflight_policy" >/dev/null

bash "$ROOT_DIR/scripts/kolme/run_local_live_node_validation_bundle_lane.sh" \
  --mode dry-run \
  --output-json "$integration_live_bundle_summary" >/dev/null
python3 "$ROOT_DIR/scripts/kolme/check_local_live_node_validation_bundle_policy.py" \
  --report-file "$integration_live_bundle_summary" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$integration_live_bundle_policy" >/dev/null

bash "$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh" \
  --max-seconds 120 \
  --output-json "$integration_gate_report" >/dev/null

integration_milestone_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$integration_milestone_bundle" \
    --release-candidate "v1.0.0-rc.5" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:milestone-integration" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --deployment-preflight-summary-file "$integration_preflight_summary" \
    --deployment-preflight-policy-file "$integration_preflight_policy" \
    --live-node-validation-summary-file "$integration_live_bundle_summary" \
    --live-node-validation-policy-file "$integration_live_bundle_policy" \
    --go-no-go-gate-report-file "$integration_gate_report"
)"

assert_eq "$(extract_value "$integration_milestone_generate_output" "final_decision")" "GO" "expected integration milestone bundle decision to be GO for real generated evidence artifacts"
integration_milestone_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$integration_milestone_bundle")"
assert_eq "$(extract_value "$integration_milestone_policy_output" "status")" "ok" "expected policy checker to pass for integration milestone bundle"
assert_eq "$(extract_value "$integration_milestone_policy_output" "final_decision")" "GO" "expected policy checker to keep GO for integration milestone bundle"
integration_milestone_lineage_output="$(
  python3 "$UPGRADE_LINEAGE_CHECKER" \
    --bundle-file "$integration_milestone_bundle" \
    --expected-final-decision GO
)"
assert_eq "$(extract_value "$integration_milestone_lineage_output" "status")" "ok" "expected upgrade lineage checker to pass for integration milestone bundle"
assert_eq "$(extract_value "$integration_milestone_lineage_output" "upgrade_lineage_final_decision")" "GO" "expected upgrade lineage checker GO decision for integration milestone bundle"

milestone_missing_artifact_bundle="$TMP_DIR/gonogo-milestone-missing-artifact.json"
milestone_missing_artifact_output="$(
  bash "$GENERATOR" \
    --output-file "$milestone_missing_artifact_bundle" \
    --release-candidate "v1.0.0-rc.4" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:milestone-missing" \
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
    --go-no-go-gate-report-file "$TMP_DIR/missing-go-no-go-gate-report.json"
)"

assert_eq "$(extract_value "$milestone_missing_artifact_output" "final_decision")" "NO-GO" "expected milestone bundle to fail closed when linked artifact is missing"
assert_eq "$(extract_value "$milestone_missing_artifact_output" "live_gonogo_reason_taxonomy_version")" "kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1" "expected deterministic live-go/no-go reason taxonomy marker for missing artifact fail-closed path"

python3 - "$milestone_missing_artifact_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
milestone = payload.get("milestone_review_bundle", {})
reason_codes = milestone.get("reason_codes")
if not isinstance(reason_codes, list):
    raise SystemExit("expected milestone reason_codes list for missing artifact case")
if "milestone_review_go_no_go_gate_report_missing" not in reason_codes:
    raise SystemExit("expected missing linked artifact reason code in milestone review bundle")
if milestone.get("reason_codes_csv") == "none":
    raise SystemExit("expected milestone reason_codes_csv to include fail-closed markers for missing artifact case")
if milestone.get("lineage_status") != "fail-closed":
    raise SystemExit("expected fail-closed lineage status for missing linked artifact case")
PY

milestone_missing_artifact_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$milestone_missing_artifact_bundle")"
assert_eq "$(extract_value "$milestone_missing_artifact_policy_output" "status")" "ok" "expected policy checker to preserve deterministic NO-GO for missing artifact bundle"
assert_eq "$(extract_value "$milestone_missing_artifact_policy_output" "final_decision")" "NO-GO" "expected policy checker NO-GO decision for missing linked artifact bundle"
milestone_missing_artifact_lineage_output="$(
  python3 "$UPGRADE_LINEAGE_CHECKER" \
    --bundle-file "$milestone_missing_artifact_bundle" \
    --expected-final-decision NO-GO \
    --require-reason-code milestone_review_go_no_go_gate_report_missing
)"
assert_eq "$(extract_value "$milestone_missing_artifact_lineage_output" "status")" "ok" "expected upgrade lineage checker deterministic NO-GO for missing artifact bundle"
assert_eq "$(extract_value "$milestone_missing_artifact_lineage_output" "upgrade_lineage_final_decision")" "NO-GO" "expected upgrade lineage checker NO-GO decision for missing linked artifact bundle"

milestone_missing_runbook_marker_doc="$TMP_DIR/milestone-runbook-marker-missing.md"
cat >"$milestone_missing_runbook_marker_doc" <<'TXT'
# Upgrade Rollback Runbook

Marker intentionally incomplete for regression coverage.
TXT

milestone_missing_runbook_bundle="$TMP_DIR/gonogo-milestone-missing-runbook.json"
milestone_missing_runbook_output="$(
  KAMN_GONOGO_RUNBOOK_DOC_FILE="$milestone_missing_runbook_marker_doc" \
    bash "$GENERATOR" \
      --output-file "$milestone_missing_runbook_bundle" \
      --release-candidate "v1.0.0-rc.6" \
      --schema-target-version "1.0.0" \
      --runtime-image-digest "sha256:milestone-missing-runbook" \
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

assert_eq "$(extract_value "$milestone_missing_runbook_output" "final_decision")" "NO-GO" "expected milestone bundle to fail closed when operator runbook markers are missing"
assert_eq "$(extract_value "$milestone_missing_runbook_output" "live_gonogo_reason_taxonomy_version")" "kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1" "expected deterministic live-go/no-go reason taxonomy marker for runbook-marker fail-closed path"

python3 - "$milestone_missing_runbook_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
milestone = payload.get("milestone_review_bundle", {})
reason_codes = milestone.get("reason_codes")
if not isinstance(reason_codes, list):
    raise SystemExit("expected milestone reason_codes list for missing runbook marker case")
if "milestone_review_operator_runbook_markers_missing" not in reason_codes:
    raise SystemExit("expected missing operator runbook markers reason code in milestone review bundle")
if milestone.get("reason_codes_csv") == "none":
    raise SystemExit("expected milestone reason_codes_csv to include fail-closed markers for missing runbook marker case")
if milestone.get("lineage_status") != "fail-closed":
    raise SystemExit("expected fail-closed lineage status for missing runbook marker case")
PY

milestone_missing_runbook_policy_output="$(
  KAMN_GONOGO_RUNBOOK_DOC_FILE="$milestone_missing_runbook_marker_doc" \
    bash "$POLICY_CHECKER" \
      --bundle-file "$milestone_missing_runbook_bundle"
)"
assert_eq "$(extract_value "$milestone_missing_runbook_policy_output" "status")" "ok" "expected policy checker to preserve deterministic NO-GO for missing runbook marker bundle"
assert_eq "$(extract_value "$milestone_missing_runbook_policy_output" "final_decision")" "NO-GO" "expected policy checker NO-GO decision for missing runbook marker bundle"
milestone_missing_runbook_lineage_output="$(
  KAMN_GONOGO_RUNBOOK_DOC_FILE="$milestone_missing_runbook_marker_doc" \
    python3 "$UPGRADE_LINEAGE_CHECKER" \
      --bundle-file "$milestone_missing_runbook_bundle" \
      --expected-final-decision NO-GO \
      --require-reason-code milestone_review_operator_runbook_markers_missing
)"
assert_eq "$(extract_value "$milestone_missing_runbook_lineage_output" "status")" "ok" "expected upgrade lineage checker deterministic NO-GO for missing runbook marker bundle"
assert_eq "$(extract_value "$milestone_missing_runbook_lineage_output" "upgrade_lineage_final_decision")" "NO-GO" "expected upgrade lineage checker NO-GO decision for missing runbook marker bundle"

milestone_taxonomy_drift_gate_report="$TMP_DIR/milestone-go-no-go-gate-report.taxonomy-drift.json"
cp "$milestone_gate_report" "$milestone_taxonomy_drift_gate_report"
python3 - "$milestone_taxonomy_drift_gate_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["combined_reason_taxonomy_version"] = "kamn.runtime.local-full-stack-integration-reason-taxonomy.v0"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

milestone_taxonomy_drift_bundle="$TMP_DIR/gonogo-milestone-taxonomy-drift.json"
milestone_taxonomy_drift_output="$(
  bash "$GENERATOR" \
    --output-file "$milestone_taxonomy_drift_bundle" \
    --release-candidate "v1.0.0-rc.7" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:milestone-taxonomy-drift" \
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
    --go-no-go-gate-report-file "$milestone_taxonomy_drift_gate_report"
)"

assert_eq "$(extract_value "$milestone_taxonomy_drift_output" "final_decision")" "NO-GO" "expected milestone bundle to fail closed on combined reason taxonomy drift"
assert_eq "$(extract_value "$milestone_taxonomy_drift_output" "live_gonogo_reason_taxonomy_version")" "kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1" "expected deterministic live-go/no-go reason taxonomy marker for taxonomy-drift fail-closed path"

python3 - "$milestone_taxonomy_drift_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
milestone = payload.get("milestone_review_bundle", {})
reason_codes = milestone.get("reason_codes")
if not isinstance(reason_codes, list):
    raise SystemExit("expected milestone reason_codes list for taxonomy drift case")
if "milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch" not in reason_codes:
    raise SystemExit("expected combined reason taxonomy mismatch reason code in milestone review bundle")
if milestone.get("reason_codes_csv") == "none":
    raise SystemExit("expected milestone reason_codes_csv to include fail-closed markers for taxonomy drift case")
if milestone.get("lineage_status") != "fail-closed":
    raise SystemExit("expected fail-closed lineage status for combined reason taxonomy drift case")
PY

milestone_taxonomy_drift_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$milestone_taxonomy_drift_bundle")"
assert_eq "$(extract_value "$milestone_taxonomy_drift_policy_output" "status")" "ok" "expected policy checker to preserve deterministic NO-GO for taxonomy drift bundle"
assert_eq "$(extract_value "$milestone_taxonomy_drift_policy_output" "final_decision")" "NO-GO" "expected policy checker NO-GO decision for taxonomy drift bundle"
milestone_taxonomy_drift_lineage_output="$(
  python3 "$UPGRADE_LINEAGE_CHECKER" \
    --bundle-file "$milestone_taxonomy_drift_bundle" \
    --expected-final-decision NO-GO \
    --require-reason-code milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch
)"
assert_eq "$(extract_value "$milestone_taxonomy_drift_lineage_output" "status")" "ok" "expected upgrade lineage checker deterministic NO-GO for taxonomy drift bundle"
assert_eq "$(extract_value "$milestone_taxonomy_drift_lineage_output" "upgrade_lineage_final_decision")" "NO-GO" "expected upgrade lineage checker NO-GO decision for taxonomy drift bundle"

milestone_rotation_taxonomy_drift_preflight_policy="$TMP_DIR/milestone-preflight-policy.rotation-taxonomy-drift.json"
cp "$milestone_preflight_policy" "$milestone_rotation_taxonomy_drift_preflight_policy"
python3 - "$milestone_rotation_taxonomy_drift_preflight_policy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["rotation_preflight_reason_taxonomy_version"] = "kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v0"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

milestone_rotation_taxonomy_drift_bundle="$TMP_DIR/gonogo-milestone-rotation-taxonomy-drift.json"
milestone_rotation_taxonomy_drift_output="$(
  bash "$GENERATOR" \
    --output-file "$milestone_rotation_taxonomy_drift_bundle" \
    --release-candidate "v1.0.0-rc.8" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:milestone-rotation-taxonomy-drift" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --deployment-preflight-summary-file "$milestone_preflight_summary" \
    --deployment-preflight-policy-file "$milestone_rotation_taxonomy_drift_preflight_policy" \
    --live-node-validation-summary-file "$milestone_live_bundle_summary" \
    --live-node-validation-policy-file "$milestone_live_bundle_policy" \
    --go-no-go-gate-report-file "$milestone_gate_report"
)"
assert_eq "$(extract_value "$milestone_rotation_taxonomy_drift_output" "final_decision")" "NO-GO" "expected milestone bundle to fail closed on deployment preflight rotation taxonomy drift"

python3 - "$milestone_rotation_taxonomy_drift_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
milestone = payload.get("milestone_review_bundle", {})
reason_codes = milestone.get("reason_codes")
if not isinstance(reason_codes, list):
    raise SystemExit("expected milestone reason_codes list for deployment preflight rotation taxonomy drift case")
if "milestone_review_deployment_preflight_policy_rotation_reason_taxonomy_mismatch" not in reason_codes:
    raise SystemExit("expected deployment preflight rotation taxonomy mismatch reason code in milestone review bundle")
PY

milestone_boundary_drift_gate_report="$TMP_DIR/milestone-go-no-go-gate-report.boundary-drift.json"
cp "$milestone_gate_report" "$milestone_boundary_drift_gate_report"
python3 - "$milestone_boundary_drift_gate_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lane_mode"] = "run"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

milestone_boundary_drift_bundle="$TMP_DIR/gonogo-milestone-boundary-drift.json"
milestone_boundary_drift_output="$(
  bash "$GENERATOR" \
    --output-file "$milestone_boundary_drift_bundle" \
    --release-candidate "v1.0.0-rc.9" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:milestone-boundary-drift" \
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
    --go-no-go-gate-report-file "$milestone_boundary_drift_gate_report"
)"
assert_eq "$(extract_value "$milestone_boundary_drift_output" "final_decision")" "NO-GO" "expected milestone bundle to fail closed on go/no-go ci/local boundary drift"

python3 - "$milestone_boundary_drift_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
milestone = payload.get("milestone_review_bundle", {})
reason_codes = milestone.get("reason_codes")
if not isinstance(reason_codes, list):
    raise SystemExit("expected milestone reason_codes list for go/no-go boundary drift case")
if "milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch" not in reason_codes:
    raise SystemExit("expected go/no-go ci/local boundary mismatch reason code in milestone review bundle")
PY

milestone_boundary_drift_lineage_output="$(
  python3 "$UPGRADE_LINEAGE_CHECKER" \
    --bundle-file "$milestone_boundary_drift_bundle" \
    --expected-final-decision NO-GO \
    --require-reason-code milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch
)"
assert_eq "$(extract_value "$milestone_boundary_drift_lineage_output" "status")" "ok" "expected upgrade lineage checker deterministic NO-GO for go/no-go boundary drift bundle"
assert_eq "$(extract_value "$milestone_boundary_drift_lineage_output" "upgrade_lineage_final_decision")" "NO-GO" "expected upgrade lineage checker NO-GO decision for go/no-go boundary drift bundle"

milestone_lineage_tampered_bundle="$TMP_DIR/gonogo-milestone-lineage-tampered.json"
cp "$milestone_bundle" "$milestone_lineage_tampered_bundle"
python3 - "$milestone_lineage_tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["milestone_review_bundle"]["observed"]["go_no_go_gate_final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
milestone_lineage_tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$milestone_lineage_tampered_bundle" 2>&1)"
milestone_lineage_tampered_code=$?
set -e

if [ "$milestone_lineage_tampered_code" -eq 0 ]; then
  echo "expected policy checker to fail for tampered milestone lineage markers" >&2
  exit 1
fi

if ! printf '%s\n' "$milestone_lineage_tampered_output" | grep -q "milestone review bundle lineage mismatch"; then
  echo "expected deterministic milestone lineage mismatch error from policy checker" >&2
  exit 1
fi

set +e
milestone_lineage_tampered_checker_output="$(
  python3 "$UPGRADE_LINEAGE_CHECKER" \
    --bundle-file "$milestone_lineage_tampered_bundle" \
    --expected-final-decision GO 2>&1
)"
milestone_lineage_tampered_checker_code=$?
set -e

if [ "$milestone_lineage_tampered_checker_code" -eq 0 ]; then
  echo "expected upgrade lineage checker to fail for tampered milestone lineage markers" >&2
  exit 1
fi

if ! printf '%s\n' "$milestone_lineage_tampered_checker_output" | grep -q "milestone review bundle lineage mismatch"; then
  echo "expected deterministic milestone lineage mismatch error from upgrade lineage checker" >&2
  exit 1
fi

milestone_promotion_mapping_drift_bundle="$TMP_DIR/gonogo-milestone-promotion-mapping-drift.json"
cp "$milestone_bundle" "$milestone_promotion_mapping_drift_bundle"
python3 - "$milestone_promotion_mapping_drift_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["milestone_review_bundle"]["reason_codes_value"] = "milestone_review_go_no_go_gate_report_missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
milestone_promotion_mapping_drift_output="$(
  python3 "$UPGRADE_LINEAGE_CHECKER" \
    --bundle-file "$milestone_promotion_mapping_drift_bundle" \
    --expected-final-decision GO 2>&1
)"
milestone_promotion_mapping_drift_code=$?
set -e

if [ "$milestone_promotion_mapping_drift_code" -eq 0 ]; then
  echo "expected upgrade lineage checker to fail on promotion-gate reason mapping drift" >&2
  exit 1
fi

if ! printf '%s\n' "$milestone_promotion_mapping_drift_output" | grep -q "promotion gate reason mapping mismatch"; then
  echo "expected deterministic promotion-gate reason mapping mismatch error from upgrade lineage checker" >&2
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

tls_bundle="$TMP_DIR/gonogo-tls-go.json"
tls_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$tls_bundle" \
    --release-candidate "v1.0.0-rc.8" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:tls-go" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --tls-evidence-report-file "$tls_evidence_report" \
    --tls-evidence-max-age-seconds 1800
)"
assert_eq "$(extract_value "$tls_generate_output" "final_decision")" "GO" "expected GO bundle decision for converged tls evidence"
assert_eq "$(extract_value "$tls_generate_output" "tls_evidence_gate_final_decision")" "GO" "expected tls evidence gate decision to be GO"
assert_eq "$(extract_value "$tls_generate_output" "tls_evidence_reason_taxonomy_version")" "kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1" "expected deterministic tls evidence gate reason taxonomy marker"
assert_eq "$(extract_value "$tls_generate_output" "tls_evidence_reason_codes_csv")" "none" "expected deterministic tls evidence gate reason csv marker on pass path"
assert_eq "$(extract_value "$tls_generate_output" "tls_evidence_reason_codes_value")" "none" "expected deterministic tls evidence gate reason value marker on pass path"

python3 - "$tls_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
gate = payload.get("tls_evidence_gate")
if not isinstance(gate, dict):
    raise SystemExit("expected tls_evidence_gate object in go/no-go evidence bundle")
if gate.get("schema_version") != "kamn.release.gonogo-tls-evidence-gate.v1":
    raise SystemExit("expected tls_evidence_gate schema marker")
if gate.get("reason_taxonomy_version") != "kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1":
    raise SystemExit("expected tls_evidence_gate reason taxonomy marker")
if gate.get("final_decision") != "GO":
    raise SystemExit("expected tls_evidence_gate final_decision=GO")
PY

tls_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$tls_bundle")"
assert_eq "$(extract_value "$tls_policy_output" "status")" "ok" "expected tls converged bundle policy check to pass"
assert_eq "$(extract_value "$tls_policy_output" "tls_evidence_gate_final_decision")" "GO" "expected policy checker tls evidence decision to remain GO"
assert_eq "$(extract_value "$tls_policy_output" "tls_evidence_reason_taxonomy_version")" "kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1" "expected policy checker to emit deterministic tls reason taxonomy marker"
assert_eq "$(extract_value "$tls_policy_output" "tls_evidence_reason_codes_csv")" "none" "expected policy checker to emit deterministic tls reason csv marker on pass path"
assert_eq "$(extract_value "$tls_policy_output" "tls_evidence_reason_codes_value")" "none" "expected policy checker to emit deterministic tls reason value marker on pass path"
assert_eq "$(extract_value "$tls_policy_output" "final_decision")" "GO" "expected policy checker final decision to remain GO for converged tls evidence"

tls_stale_report="$TMP_DIR/tls-evidence-stale-report.json"
cp "$tls_evidence_report" "$tls_stale_report"
touch -d '1970-01-01T00:00:00Z' "$tls_stale_report"

tls_stale_bundle="$TMP_DIR/gonogo-tls-stale.json"
tls_stale_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$tls_stale_bundle" \
    --release-candidate "v1.0.0-rc.9" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:tls-stale" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --tls-evidence-report-file "$tls_stale_report" \
    --tls-evidence-max-age-seconds 1
)"
assert_eq "$(extract_value "$tls_stale_generate_output" "tls_evidence_gate_final_decision")" "NO-GO" "expected tls evidence gate to fail closed for stale evidence"
assert_eq "$(extract_value "$tls_stale_generate_output" "tls_evidence_reason_codes_csv")" "gonogo_tls_evidence_freshness_window_exceeded" "expected deterministic stale tls evidence reason code"
assert_eq "$(extract_value "$tls_stale_generate_output" "final_decision")" "NO-GO" "expected final decision to fail closed for stale tls evidence"

tls_stale_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$tls_stale_bundle")"
assert_eq "$(extract_value "$tls_stale_policy_output" "status")" "ok" "expected stale tls bundle policy check to pass deterministically"
assert_eq "$(extract_value "$tls_stale_policy_output" "tls_evidence_gate_final_decision")" "NO-GO" "expected stale tls evidence gate policy decision to remain NO-GO"
assert_eq "$(extract_value "$tls_stale_policy_output" "tls_evidence_reason_taxonomy_version")" "kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1" "expected stale tls policy output to include deterministic reason taxonomy marker"
assert_eq "$(extract_value "$tls_stale_policy_output" "tls_evidence_reason_codes_csv")" "gonogo_tls_evidence_freshness_window_exceeded" "expected stale tls policy output to include deterministic reason csv marker"
assert_eq "$(extract_value "$tls_stale_policy_output" "tls_evidence_reason_codes_value")" "gonogo_tls_evidence_freshness_window_exceeded" "expected stale tls policy output to include deterministic reason value marker"
assert_eq "$(extract_value "$tls_stale_policy_output" "final_decision")" "NO-GO" "expected stale tls bundle policy decision to remain NO-GO"

tls_missing_bundle="$TMP_DIR/gonogo-tls-missing.json"
tls_missing_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$tls_missing_bundle" \
    --release-candidate "v1.0.0-rc.10" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:tls-missing" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --tls-evidence-report-file "$TMP_DIR/missing-tls-evidence-report.json" \
    --tls-evidence-max-age-seconds 1800
)"
assert_eq "$(extract_value "$tls_missing_generate_output" "tls_evidence_gate_final_decision")" "NO-GO" "expected tls evidence gate to fail closed for missing evidence file"
assert_eq "$(extract_value "$tls_missing_generate_output" "tls_evidence_reason_codes_csv")" "gonogo_tls_evidence_file_missing" "expected deterministic missing tls evidence reason code"
assert_eq "$(extract_value "$tls_missing_generate_output" "tls_evidence_reason_codes_value")" "gonogo_tls_evidence_file_missing" "expected deterministic missing tls evidence reason value marker"
assert_eq "$(extract_value "$tls_missing_generate_output" "final_decision")" "NO-GO" "expected final decision to fail closed for missing tls evidence"

tls_missing_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$tls_missing_bundle")"
assert_eq "$(extract_value "$tls_missing_policy_output" "status")" "ok" "expected missing tls bundle policy check to pass deterministically"
assert_eq "$(extract_value "$tls_missing_policy_output" "tls_evidence_gate_final_decision")" "NO-GO" "expected missing tls evidence gate policy decision to remain NO-GO"
assert_eq "$(extract_value "$tls_missing_policy_output" "tls_evidence_reason_taxonomy_version")" "kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1" "expected missing tls policy output to include deterministic reason taxonomy marker"
assert_eq "$(extract_value "$tls_missing_policy_output" "tls_evidence_reason_codes_csv")" "gonogo_tls_evidence_file_missing" "expected missing tls policy output to include deterministic reason csv marker"
assert_eq "$(extract_value "$tls_missing_policy_output" "tls_evidence_reason_codes_value")" "gonogo_tls_evidence_file_missing" "expected missing tls policy output to include deterministic reason value marker"
assert_eq "$(extract_value "$tls_missing_policy_output" "final_decision")" "NO-GO" "expected missing tls bundle policy decision to remain NO-GO"

tls_invalid_json_report="$TMP_DIR/tls-evidence-invalid-json-report.json"
cat >"$tls_invalid_json_report" <<'JSON'
{ invalid json payload
JSON

tls_invalid_json_bundle="$TMP_DIR/gonogo-tls-invalid-json.json"
tls_invalid_json_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$tls_invalid_json_bundle" \
    --release-candidate "v1.0.0-rc.10.1" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:tls-invalid-json" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --tls-evidence-report-file "$tls_invalid_json_report" \
    --tls-evidence-max-age-seconds 1800
)"
assert_eq "$(extract_value "$tls_invalid_json_generate_output" "tls_evidence_gate_final_decision")" "NO-GO" "expected tls evidence gate to fail closed for invalid json evidence"
assert_eq "$(extract_value "$tls_invalid_json_generate_output" "tls_evidence_reason_codes_csv")" "gonogo_tls_evidence_invalid_json" "expected deterministic invalid-json tls evidence reason code"
assert_eq "$(extract_value "$tls_invalid_json_generate_output" "tls_evidence_reason_codes_value")" "gonogo_tls_evidence_invalid_json" "expected deterministic invalid-json tls evidence reason value marker"
assert_eq "$(extract_value "$tls_invalid_json_generate_output" "final_decision")" "NO-GO" "expected final decision to fail closed for invalid-json tls evidence"

tls_invalid_json_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$tls_invalid_json_bundle")"
assert_eq "$(extract_value "$tls_invalid_json_policy_output" "status")" "ok" "expected invalid-json tls bundle policy check to pass deterministically"
assert_eq "$(extract_value "$tls_invalid_json_policy_output" "tls_evidence_gate_final_decision")" "NO-GO" "expected invalid-json tls evidence gate policy decision to remain NO-GO"
assert_eq "$(extract_value "$tls_invalid_json_policy_output" "tls_evidence_reason_taxonomy_version")" "kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1" "expected invalid-json tls policy output to include deterministic reason taxonomy marker"
assert_eq "$(extract_value "$tls_invalid_json_policy_output" "tls_evidence_reason_codes_csv")" "gonogo_tls_evidence_invalid_json" "expected invalid-json tls policy output to include deterministic reason csv marker"
assert_eq "$(extract_value "$tls_invalid_json_policy_output" "tls_evidence_reason_codes_value")" "gonogo_tls_evidence_invalid_json" "expected invalid-json tls policy output to include deterministic reason value marker"
assert_eq "$(extract_value "$tls_invalid_json_policy_output" "final_decision")" "NO-GO" "expected invalid-json tls bundle policy decision to remain NO-GO"

tls_tampered_bundle="$TMP_DIR/gonogo-tls-tampered.json"
cp "$tls_bundle" "$tls_tampered_bundle"
python3 - "$tls_tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["tls_evidence_gate"]["observed"]["tls_evidence_report_status"] = "fail"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tls_tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tls_tampered_bundle" 2>&1)"
tls_tampered_code=$?
set -e

if [ "$tls_tampered_code" -eq 0 ]; then
  echo "expected policy checker to fail for tampered tls evidence gate markers" >&2
  exit 1
fi

if ! printf '%s\n' "$tls_tampered_output" | grep -q "tls evidence gate convergence mismatch"; then
  echo "expected deterministic tls evidence gate convergence mismatch error from policy checker" >&2
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

audit_bundle="$TMP_DIR/gonogo-audit-integrity-go.json"
audit_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$audit_bundle" \
    --release-candidate "v1.0.0-rc.11" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:audit-integrity-go" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --audit-integrity-report-file "$audit_integrity_report" \
    --audit-integrity-max-age-seconds 1800
)"
assert_eq "$(extract_value "$audit_generate_output" "audit_integrity_gate_final_decision")" "GO" "expected audit-integrity gate decision to be GO"
assert_eq "$(extract_value "$audit_generate_output" "audit_integrity_reason_taxonomy_version")" "kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1" "expected deterministic audit-integrity reason taxonomy marker"
assert_eq "$(extract_value "$audit_generate_output" "audit_integrity_reason_codes_csv")" "none" "expected deterministic audit-integrity reason codes csv marker on pass path"
assert_eq "$(extract_value "$audit_generate_output" "final_decision")" "GO" "expected final decision to remain GO for converged audit-integrity evidence"

python3 - "$audit_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
gate = payload.get("audit_integrity_gate")
if not isinstance(gate, dict):
    raise SystemExit("expected audit_integrity_gate object in go/no-go evidence bundle")
if gate.get("schema_version") != "kamn.release.gonogo-audit-integrity-gate.v1":
    raise SystemExit("expected audit-integrity gate schema marker")
if gate.get("reason_taxonomy_version") != "kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1":
    raise SystemExit("expected audit-integrity gate reason taxonomy marker")
if gate.get("final_decision") != "GO":
    raise SystemExit("expected audit-integrity gate final_decision=GO")
PY

audit_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$audit_bundle")"
assert_eq "$(extract_value "$audit_policy_output" "status")" "ok" "expected audit-integrity converged bundle policy check to pass"
assert_eq "$(extract_value "$audit_policy_output" "audit_integrity_gate_final_decision")" "GO" "expected audit-integrity gate policy decision to remain GO"
assert_eq "$(extract_value "$audit_policy_output" "final_decision")" "GO" "expected policy checker final decision to remain GO for converged audit-integrity evidence"

audit_unstable_report="$TMP_DIR/audit-integrity-unstable-report.json"
cp "$audit_integrity_report" "$audit_unstable_report"
python3 - "$audit_unstable_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["durability_governance_reason_taxonomy_version"] = "kamn.runtime.durability-governance-reason-taxonomy.v0"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

audit_unstable_bundle="$TMP_DIR/gonogo-audit-integrity-unstable.json"
audit_unstable_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$audit_unstable_bundle" \
    --release-candidate "v1.0.0-rc.12" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:audit-integrity-unstable" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --audit-integrity-report-file "$audit_unstable_report" \
    --audit-integrity-max-age-seconds 1800
)"
assert_eq "$(extract_value "$audit_unstable_generate_output" "audit_integrity_gate_final_decision")" "NO-GO" "expected audit-integrity gate to fail closed for unstable taxonomy outputs"
assert_eq "$(extract_value "$audit_unstable_generate_output" "audit_integrity_reason_codes_csv")" "gonogo_audit_integrity_reason_taxonomy_version_mismatch" "expected deterministic unstable audit-integrity taxonomy mismatch reason code"
assert_eq "$(extract_value "$audit_unstable_generate_output" "final_decision")" "NO-GO" "expected final decision to fail closed for unstable audit-integrity outputs"

audit_unstable_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$audit_unstable_bundle")"
assert_eq "$(extract_value "$audit_unstable_policy_output" "status")" "ok" "expected unstable audit-integrity bundle policy check to pass deterministically"
assert_eq "$(extract_value "$audit_unstable_policy_output" "audit_integrity_gate_final_decision")" "NO-GO" "expected unstable audit-integrity gate policy decision to remain NO-GO"
assert_eq "$(extract_value "$audit_unstable_policy_output" "final_decision")" "NO-GO" "expected unstable audit-integrity bundle policy decision to remain NO-GO"

audit_tampered_bundle="$TMP_DIR/gonogo-audit-integrity-tampered.json"
cp "$audit_bundle" "$audit_tampered_bundle"
python3 - "$audit_tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["audit_integrity_gate"]["observed"]["audit_integrity_report_status"] = "fail"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
audit_tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$audit_tampered_bundle" 2>&1)"
audit_tampered_code=$?
set -e

if [ "$audit_tampered_code" -eq 0 ]; then
  echo "expected policy checker to fail for tampered audit-integrity gate markers" >&2
  exit 1
fi

if ! printf '%s\n' "$audit_tampered_output" | grep -q "audit integrity gate convergence mismatch"; then
  echo "expected deterministic audit-integrity gate convergence mismatch error from policy checker" >&2
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

slo_policy_bundle="$TMP_DIR/gonogo-slo-policy-go.json"
slo_policy_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$slo_policy_bundle" \
    --release-candidate "v1.0.0-rc.13" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:slo-policy-go" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --slo-policy-report-file "$slo_policy_report" \
    --slo-policy-max-age-seconds 1800
)"
assert_eq "$(extract_value "$slo_policy_generate_output" "slo_policy_gate_final_decision")" "GO" "expected SLO policy gate decision to be GO"
assert_eq "$(extract_value "$slo_policy_generate_output" "slo_policy_reason_taxonomy_version")" "kamn.release.gonogo-slo-threshold-convergence-reason-taxonomy.v1" "expected deterministic SLO policy gate reason taxonomy marker"
assert_eq "$(extract_value "$slo_policy_generate_output" "slo_policy_reason_codes_csv")" "none" "expected deterministic SLO policy gate reason codes csv marker on pass path"
assert_eq "$(extract_value "$slo_policy_generate_output" "final_decision")" "GO" "expected final decision to remain GO for converged SLO policy evidence"

python3 - "$slo_policy_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
gate = payload.get("slo_policy_gate")
if not isinstance(gate, dict):
    raise SystemExit("expected slo_policy_gate object in go/no-go evidence bundle")
if gate.get("schema_version") != "kamn.release.gonogo-slo-policy-gate.v1":
    raise SystemExit("expected SLO policy gate schema marker")
if gate.get("reason_taxonomy_version") != "kamn.release.gonogo-slo-threshold-convergence-reason-taxonomy.v1":
    raise SystemExit("expected SLO policy gate reason taxonomy marker")
if gate.get("final_decision") != "GO":
    raise SystemExit("expected SLO policy gate final_decision=GO")
PY

slo_policy_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$slo_policy_bundle")"
assert_eq "$(extract_value "$slo_policy_policy_output" "status")" "ok" "expected SLO policy converged bundle policy check to pass"
assert_eq "$(extract_value "$slo_policy_policy_output" "slo_policy_gate_final_decision")" "GO" "expected SLO policy gate policy decision to remain GO"
assert_eq "$(extract_value "$slo_policy_policy_output" "final_decision")" "GO" "expected policy checker final decision to remain GO for converged SLO policy evidence"

slo_policy_drift_report="$TMP_DIR/slo-policy-threshold-drift-report.json"
cp "$slo_policy_report" "$slo_policy_drift_report"
python3 - "$slo_policy_drift_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_key"] = "deployment_slo_rollback_reason_codes:NO-GO:v1"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

slo_policy_drift_bundle="$TMP_DIR/gonogo-slo-policy-threshold-drift.json"
slo_policy_drift_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$slo_policy_drift_bundle" \
    --release-candidate "v1.0.0-rc.14" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:slo-policy-drift" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --slo-policy-report-file "$slo_policy_drift_report" \
    --slo-policy-max-age-seconds 1800
)"
assert_eq "$(extract_value "$slo_policy_drift_generate_output" "slo_policy_gate_final_decision")" "NO-GO" "expected SLO policy gate to fail closed for threshold drift evidence"
assert_eq "$(extract_value "$slo_policy_drift_generate_output" "slo_policy_reason_codes_csv")" "gonogo_slo_policy_reason_key_mismatch" "expected deterministic SLO reason-key mismatch reason code"
assert_eq "$(extract_value "$slo_policy_drift_generate_output" "final_decision")" "NO-GO" "expected final decision to fail closed for SLO threshold drift evidence"

slo_policy_drift_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$slo_policy_drift_bundle")"
assert_eq "$(extract_value "$slo_policy_drift_policy_output" "status")" "ok" "expected SLO threshold-drift bundle policy check to pass deterministically"
assert_eq "$(extract_value "$slo_policy_drift_policy_output" "slo_policy_gate_final_decision")" "NO-GO" "expected SLO threshold-drift gate policy decision to remain NO-GO"
assert_eq "$(extract_value "$slo_policy_drift_policy_output" "final_decision")" "NO-GO" "expected SLO threshold-drift bundle policy decision to remain NO-GO"

slo_policy_tampered_bundle="$TMP_DIR/gonogo-slo-policy-tampered.json"
cp "$slo_policy_bundle" "$slo_policy_tampered_bundle"
python3 - "$slo_policy_tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["slo_policy_gate"]["observed"]["slo_policy_report_status"] = "fail"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
slo_policy_tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$slo_policy_tampered_bundle" 2>&1)"
slo_policy_tampered_code=$?
set -e

if [ "$slo_policy_tampered_code" -eq 0 ]; then
  echo "expected policy checker to fail for tampered SLO policy gate markers" >&2
  exit 1
fi

if ! printf '%s\n' "$slo_policy_tampered_output" | grep -q "slo policy gate convergence mismatch"; then
  echo "expected deterministic SLO policy gate convergence mismatch error from policy checker" >&2
  exit 1
fi

incident_readiness_report="$TMP_DIR/incident-readiness-report.json"
bash "$ROOT_DIR/scripts/deploy/generate_staging_rehearsal_bundle.sh" \
  --output-file "$incident_readiness_report" \
  --release-candidate "v1.0.0-incident-readiness" \
  --deploy-status PASS \
  --rollback-status PASS \
  --rollback-target-hash "state-hash-incident-ready" \
  --post-rollback-hash "state-hash-incident-ready" \
  --recovery-time-seconds 420 \
  --max-allowed-recovery-time-seconds 900 \
  --evidence-complete true \
  --ci-fast-gate PASS >/dev/null

incident_readiness_bundle="$TMP_DIR/gonogo-incident-readiness-go.json"
incident_readiness_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$incident_readiness_bundle" \
    --release-candidate "v1.0.0-rc.15" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:incident-readiness-go" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --incident-readiness-report-file "$incident_readiness_report" \
    --incident-readiness-max-age-seconds 1800
)"
assert_eq "$(extract_value "$incident_readiness_generate_output" "incident_readiness_gate_final_decision")" "GO" "expected incident-readiness gate decision to be GO"
assert_eq "$(extract_value "$incident_readiness_generate_output" "incident_readiness_reason_taxonomy_version")" "kamn.release.gonogo-incident-readiness-convergence-reason-taxonomy.v1" "expected deterministic incident-readiness gate reason taxonomy marker"
assert_eq "$(extract_value "$incident_readiness_generate_output" "incident_readiness_reason_codes_csv")" "none" "expected deterministic incident-readiness gate reason codes csv marker on pass path"
assert_eq "$(extract_value "$incident_readiness_generate_output" "final_decision")" "GO" "expected final decision to remain GO for converged incident-readiness evidence"

python3 - "$incident_readiness_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
gate = payload.get("incident_readiness_gate")
if not isinstance(gate, dict):
    raise SystemExit("expected incident_readiness_gate object in go/no-go evidence bundle")
if gate.get("schema_version") != "kamn.release.gonogo-incident-readiness-gate.v1":
    raise SystemExit("expected incident-readiness gate schema marker")
if gate.get("reason_taxonomy_version") != "kamn.release.gonogo-incident-readiness-convergence-reason-taxonomy.v1":
    raise SystemExit("expected incident-readiness gate reason taxonomy marker")
if gate.get("final_decision") != "GO":
    raise SystemExit("expected incident-readiness gate final_decision=GO")
PY

incident_readiness_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$incident_readiness_bundle")"
assert_eq "$(extract_value "$incident_readiness_policy_output" "status")" "ok" "expected incident-readiness converged bundle policy check to pass"
assert_eq "$(extract_value "$incident_readiness_policy_output" "incident_readiness_gate_final_decision")" "GO" "expected incident-readiness gate policy decision to remain GO"
assert_eq "$(extract_value "$incident_readiness_policy_output" "final_decision")" "GO" "expected policy checker final decision to remain GO for converged incident-readiness evidence"

incident_readiness_drift_report="$TMP_DIR/incident-readiness-taxonomy-drift-report.json"
cp "$incident_readiness_report" "$incident_readiness_drift_report"
python3 - "$incident_readiness_drift_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_taxonomy"]["schema_version"] = "kamn.release.staging-rehearsal-reason-taxonomy.v0"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

incident_readiness_drift_bundle="$TMP_DIR/gonogo-incident-readiness-taxonomy-drift.json"
incident_readiness_drift_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$incident_readiness_drift_bundle" \
    --release-candidate "v1.0.0-rc.16" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:incident-readiness-drift" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --incident-readiness-report-file "$incident_readiness_drift_report" \
    --incident-readiness-max-age-seconds 1800
)"
assert_eq "$(extract_value "$incident_readiness_drift_generate_output" "incident_readiness_gate_final_decision")" "NO-GO" "expected incident-readiness gate to fail closed for taxonomy drift evidence"
assert_eq "$(extract_value "$incident_readiness_drift_generate_output" "incident_readiness_reason_codes_csv")" "gonogo_incident_readiness_reason_taxonomy_schema_mismatch" "expected deterministic incident-readiness taxonomy mismatch reason code"
assert_eq "$(extract_value "$incident_readiness_drift_generate_output" "final_decision")" "NO-GO" "expected final decision to fail closed for incident-readiness taxonomy drift evidence"

incident_readiness_stale_report="$TMP_DIR/incident-readiness-stale-report.json"
cp "$incident_readiness_report" "$incident_readiness_stale_report"
touch -d '1970-01-01T00:00:00Z' "$incident_readiness_stale_report"

incident_readiness_stale_bundle="$TMP_DIR/gonogo-incident-readiness-stale.json"
incident_readiness_stale_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$incident_readiness_stale_bundle" \
    --release-candidate "v1.0.0-rc.17" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:incident-readiness-stale" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --incident-readiness-report-file "$incident_readiness_stale_report" \
    --incident-readiness-max-age-seconds 1
)"
assert_eq "$(extract_value "$incident_readiness_stale_generate_output" "incident_readiness_gate_final_decision")" "NO-GO" "expected incident-readiness gate to fail closed for stale evidence"
assert_eq "$(extract_value "$incident_readiness_stale_generate_output" "incident_readiness_reason_codes_csv")" "gonogo_incident_readiness_freshness_window_exceeded" "expected deterministic stale incident-readiness reason code"
assert_eq "$(extract_value "$incident_readiness_stale_generate_output" "final_decision")" "NO-GO" "expected final decision to fail closed for stale incident-readiness evidence"

incident_readiness_partial_report="$TMP_DIR/incident-readiness-partial-evidence-report.json"
cp "$incident_readiness_report" "$incident_readiness_partial_report"
python3 - "$incident_readiness_partial_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("staged_rehearsal_signoff", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

incident_readiness_partial_bundle="$TMP_DIR/gonogo-incident-readiness-partial-evidence.json"
incident_readiness_partial_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$incident_readiness_partial_bundle" \
    --release-candidate "v1.0.0-rc.18" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:incident-readiness-partial" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2 \
    --incident-readiness-report-file "$incident_readiness_partial_report" \
    --incident-readiness-max-age-seconds 1800
)"
assert_eq "$(extract_value "$incident_readiness_partial_generate_output" "incident_readiness_gate_final_decision")" "NO-GO" "expected incident-readiness gate to fail closed for partial readiness evidence"
assert_eq "$(extract_value "$incident_readiness_partial_generate_output" "incident_readiness_reason_codes_csv")" "gonogo_incident_readiness_staged_signoff_schema_mismatch,gonogo_incident_readiness_staged_signoff_status_not_verified" "expected deterministic incident-readiness partial evidence convergence reason code markers"
assert_eq "$(extract_value "$incident_readiness_partial_generate_output" "final_decision")" "NO-GO" "expected final decision to fail closed for partial incident-readiness evidence"

incident_readiness_tampered_bundle="$TMP_DIR/gonogo-incident-readiness-tampered.json"
cp "$incident_readiness_bundle" "$incident_readiness_tampered_bundle"
python3 - "$incident_readiness_tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["incident_readiness_gate"]["observed"]["incident_readiness_report_final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
incident_readiness_tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$incident_readiness_tampered_bundle" 2>&1)"
incident_readiness_tampered_code=$?
set -e

if [ "$incident_readiness_tampered_code" -eq 0 ]; then
  echo "expected policy checker to fail for tampered incident-readiness gate markers" >&2
  exit 1
fi

if ! printf '%s\n' "$incident_readiness_tampered_output" | grep -q "incident readiness gate convergence mismatch"; then
  echo "expected deterministic incident-readiness gate convergence mismatch error from policy checker" >&2
  exit 1
fi

echo "go/no-go evidence bundle tests passed."

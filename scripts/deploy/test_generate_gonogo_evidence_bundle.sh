#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_gonogo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_gonogo_evidence_policy.sh"
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
PY

milestone_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$milestone_bundle")"
assert_eq "$(extract_value "$milestone_policy_output" "status")" "ok" "expected milestone bundle policy check to pass"
assert_eq "$(extract_value "$milestone_policy_output" "final_decision")" "GO" "expected milestone bundle policy to keep GO decision"

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
if milestone.get("lineage_status") != "fail-closed":
    raise SystemExit("expected fail-closed lineage status for missing linked artifact case")
PY

milestone_missing_artifact_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$milestone_missing_artifact_bundle")"
assert_eq "$(extract_value "$milestone_missing_artifact_policy_output" "status")" "ok" "expected policy checker to preserve deterministic NO-GO for missing artifact bundle"
assert_eq "$(extract_value "$milestone_missing_artifact_policy_output" "final_decision")" "NO-GO" "expected policy checker NO-GO decision for missing linked artifact bundle"

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

echo "go/no-go evidence bundle tests passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"
RUNNER_IMPL="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_lane_impl.sh"
RUN_MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_live_deployment_preflight_lane.json"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_SUMMARY="$(mktemp)"
TMP_ERR="$(mktemp)"
TMP_CUSTODY="$(mktemp)"
TMP_PROVENANCE="$(mktemp)"
TMP_QUORUM="$(mktemp)"
TMP_QUORUM_SINGLE="$(mktemp)"
trap 'rm -f "$TMP_SUMMARY" "$TMP_ERR" "$TMP_CUSTODY" "$TMP_PROVENANCE" "$TMP_QUORUM" "$TMP_QUORUM_SINGLE"' EXIT

printf '%s\n' "custody-attestation=ops-primary:epoch-1" >"$TMP_CUSTODY"
printf '%s\n' "signer-provenance=ops-primary:source-managed-external:epoch-1" >"$TMP_PROVENANCE"
TMP_CUSTODY_SHA="$(sha256sum "$TMP_CUSTODY" | awk '{print $1}')"
cat >"$TMP_QUORUM" <<JSON
{
  "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
  "required_approvals": 2,
  "received_approvals": 2,
  "approved_signers": [
    "ops-primary",
    "ops-secondary"
  ],
  "custody_evidence_sha256": "$TMP_CUSTODY_SHA",
  "signer_roles": {
    "ops-primary": "primary",
    "ops-secondary": "secondary"
  },
  "signer_rotation_epochs": {
    "ops-primary": 3,
    "ops-secondary": 2
  }
}
JSON

cat >"$TMP_QUORUM_SINGLE" <<JSON
{
  "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
  "required_approvals": 1,
  "received_approvals": 1,
  "approved_signers": [
    "ops-primary"
  ],
  "custody_evidence_sha256": "$TMP_CUSTODY_SHA",
  "signer_roles": {
    "ops-primary": "primary"
  },
  "signer_rotation_epochs": {
    "ops-primary": 3
  }
}
JSON

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

if [ ! -x "$RUNNER" ]; then
  echo "expected local Kolme live deployment preflight lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$RUNNER_IMPL" ]; then
  echo "expected local Kolme live deployment preflight implementation runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected local run lane dispatcher to be executable" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected local Kolme live deployment preflight lane runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local Kolme live deployment preflight lane runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if [ ! -f "$RUN_MANIFEST" ]; then
  echo "expected local Kolme live deployment preflight run manifest to exist" >&2
  exit 1
fi

python3 - "$RUN_MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected local Kolme live deployment preflight run manifest schema")
if payload.get("lane_id") != "kolme.local_kolme_live_deployment_preflight.run":
    raise SystemExit("unexpected local Kolme live deployment preflight run manifest lane_id")
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_local_kolme_live_deployment_preflight_lane_impl.sh",
]:
    raise SystemExit("unexpected local Kolme live deployment preflight run manifest command")
PY

resolved_run_manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
if [ "$resolved_run_manifest_path" != "$RUN_MANIFEST" ]; then
  echo "expected local Kolme live deployment preflight wrapper to resolve deterministic run manifest" >&2
  exit 1
fi

if bash "$DISPATCHER" --lane-wrapper run_missing_local_kolme_live_deployment_preflight_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected local run lane dispatcher to fail closed for unknown local Kolme live deployment preflight wrapper" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_deployment_preflight_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference deployment preflight lane runner" >&2
  exit 1
fi

if ! grep -q "run_lane_dispatch.sh --lane-wrapper run_local_kolme_live_deployment_preflight_lane.sh --resolve-manifest-path" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference deployment preflight run-wrapper dispatcher mapping" >&2
  exit 1
fi

if ! grep -q "kolme_local_kolme_live_deployment_preflight_lane.json" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference deployment preflight run manifest" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference deployment preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_deployment_preflight_lane.sh" "$CI_DOC_FILE"; then
  echo "expected CI strategy doc to reference deployment preflight lane runner" >&2
  exit 1
fi

if ! grep -q "run_lane_dispatch.sh --lane-wrapper run_local_kolme_live_deployment_preflight_lane.sh --resolve-manifest-path" "$CI_DOC_FILE"; then
  echo "expected CI strategy doc to reference deployment preflight run-wrapper dispatcher mapping" >&2
  exit 1
fi

if ! grep -q "kolme_local_kolme_live_deployment_preflight_lane.json" "$CI_DOC_FILE"; then
  echo "expected CI strategy doc to reference deployment preflight run manifest" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$CI_DOC_FILE"; then
  echo "expected CI strategy doc to reference deployment preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_deployment_preflight_lane.sh" "$README_FILE"; then
  echo "expected README to reference deployment preflight lane runner" >&2
  exit 1
fi

if ! grep -q "run_lane_dispatch.sh --lane-wrapper run_local_kolme_live_deployment_preflight_lane.sh --resolve-manifest-path" "$README_FILE"; then
  echo "expected README to reference deployment preflight run-wrapper dispatcher mapping" >&2
  exit 1
fi

if ! grep -q "kolme_local_kolme_live_deployment_preflight_lane.json" "$README_FILE"; then
  echo "expected README to reference deployment preflight run manifest" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$README_FILE"; then
  echo "expected README to reference deployment preflight policy checker" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_SUMMARY"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected deployment preflight dry-run status"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected deployment preflight dry-run lane mode"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected deployment preflight dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "ci_fast_gate_eligible")" "true" "expected deployment preflight lane to be fast-gate eligible"

python3 - "$TMP_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-summary.v1":
    raise SystemExit("unexpected deployment preflight summary schema")
if summary.get("runtime_mode") != "kolme-live":
    raise SystemExit("expected runtime_mode=kolme-live in deployment preflight summary")
if summary.get("signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
    raise SystemExit("expected signer profile selector env marker in deployment preflight summary")
if summary.get("signer_profile") != "ops-primary":
    raise SystemExit("expected signer profile marker in deployment preflight summary")
if summary.get("signer_profile_class") != "production":
    raise SystemExit("expected signer profile class marker in deployment preflight summary")
if summary.get("signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
    raise SystemExit("expected signer private key env marker in deployment preflight summary")
if summary.get("fallback_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK":
    raise SystemExit("expected fallback signer private key env marker in deployment preflight summary")
if summary.get("fallback_signer_secret_remediation") != "unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK":
    raise SystemExit("expected fallback signer remediation marker in deployment preflight summary")
if summary.get("fallback_signer_secret_present") is not False:
    raise SystemExit("expected fallback signer secret presence marker to be false in deployment preflight summary")
if summary.get("ci_fast_gate_eligible") is not True:
    raise SystemExit("expected deployment preflight summary to remain fast-gate eligible")
if summary.get("required_approvals") != 2:
    raise SystemExit("expected deployment preflight summary required_approvals=2")
if summary.get("received_approvals") != 0:
    raise SystemExit("expected deployment preflight summary received_approvals=0 for dry-run")
if summary.get("quorum_evidence_present") is not False:
    raise SystemExit("expected deployment preflight summary quorum evidence marker to be false in dry-run")
if summary.get("quorum_evidence_matches_threshold") is not False:
    raise SystemExit("expected deployment preflight summary quorum threshold marker to be false in dry-run")
if summary.get("quorum_evidence_signer_roles_present") is not False:
    raise SystemExit("expected deployment preflight summary signer-roles metadata marker to be false in dry-run")
if summary.get("quorum_evidence_signer_roles_valid") is not False:
    raise SystemExit("expected deployment preflight summary signer-roles metadata validity marker to be false in dry-run")
if summary.get("quorum_evidence_rotation_metadata_present") is not False:
    raise SystemExit("expected deployment preflight summary rotation metadata marker to be false in dry-run")
if summary.get("quorum_evidence_rotation_metadata_valid") is not False:
    raise SystemExit("expected deployment preflight summary rotation metadata validity marker to be false in dry-run")
if summary.get("custody_evidence_present") is not False:
    raise SystemExit("expected deployment preflight summary custody evidence marker to be false in dry-run")
if summary.get("signer_provenance_present") is not False:
    raise SystemExit("expected deployment preflight summary signer provenance marker to be false in dry-run")
if summary.get("signer_key_source_contract_version") != "v1":
    raise SystemExit("expected deployment preflight summary signer key-source contract version marker")
if summary.get("signer_key_source") != "managed-external":
    raise SystemExit("expected deployment preflight summary signer key-source marker")
if summary.get("signer_rotation_epoch") != 1:
    raise SystemExit("expected deployment preflight summary signer rotation epoch marker")
if summary.get("signer_previous_rotation_epoch") != 1:
    raise SystemExit("expected deployment preflight summary signer previous rotation epoch marker")
if summary.get("signer_rotation_freshness_max_delta") != 2:
    raise SystemExit("expected deployment preflight summary signer rotation freshness max delta marker")
contracts = summary.get("contracts", {})
if contracts.get("ci_fast_gate_scope") != "ci-fast-gate":
    raise SystemExit("expected deployment preflight contracts to set ci-fast-gate scope")
if contracts.get("fallback_private_key_path_allowed") is not False:
    raise SystemExit("expected deployment preflight contracts to prohibit fallback private key paths")
if contracts.get("fallback_signer_secret_rejected_profile_class") != "production":
    raise SystemExit("expected deployment preflight contracts to scope fallback signer secret rejection to production profiles")
if contracts.get("fallback_signer_secret_rejected_profiles") != ["ops-primary", "ops-secondary"]:
    raise SystemExit("expected deployment preflight contracts to define fallback signer secret rejected profiles")
if contracts.get("fallback_signer_secret_remediation") != "unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK":
    raise SystemExit("expected deployment preflight contracts fallback signer remediation marker")
if contracts.get("fallback_signer_secret_rejection_reason_code") != "fallback_signer_secret_present_violation":
    raise SystemExit("expected deployment preflight contracts fallback signer rejection reason marker")
if contracts.get("fallback_signer_secret_checkpoint_reason_code") != "checkpoint_failed_fallback_private_key_contract":
    raise SystemExit("expected deployment preflight contracts fallback signer checkpoint reason marker")
if contracts.get("custody_evidence_required") is not True:
    raise SystemExit("expected deployment preflight contracts to require signer custody evidence")
if contracts.get("approval_quorum_required") != 2:
    raise SystemExit("expected deployment preflight contracts approval quorum requirement marker")
if contracts.get("approval_quorum_minimum") != 2:
    raise SystemExit("expected deployment preflight contracts approval quorum minimum marker")
if contracts.get("quorum_evidence_required") is not True:
    raise SystemExit("expected deployment preflight contracts to require quorum evidence")
if contracts.get("quorum_evidence_sha256_required") is not True:
    raise SystemExit("expected deployment preflight contracts to require quorum evidence sha256")
if contracts.get("quorum_evidence_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected deployment preflight contracts quorum evidence schema marker")
if contracts.get("quorum_evidence_signer_uniqueness_required") is not True:
    raise SystemExit("expected deployment preflight contracts quorum signer uniqueness requirement marker")
if contracts.get("quorum_evidence_custody_sha256_match_required") is not True:
    raise SystemExit("expected deployment preflight contracts quorum custody sha256 match requirement marker")
if contracts.get("quorum_evidence_signer_roles_required") is not True:
    raise SystemExit("expected deployment preflight contracts quorum signer-role metadata requirement marker")
if contracts.get("quorum_evidence_signer_roles_allowed") != ["primary", "secondary"]:
    raise SystemExit("expected deployment preflight contracts quorum signer-role allowlist marker")
if contracts.get("quorum_evidence_rotation_metadata_required") is not True:
    raise SystemExit("expected deployment preflight contracts quorum rotation metadata requirement marker")
if contracts.get("quorum_evidence_rotation_metadata_positive_epochs_required") is not True:
    raise SystemExit("expected deployment preflight contracts quorum rotation positive epoch requirement marker")
if contracts.get("signer_provenance_required") is not True:
    raise SystemExit("expected deployment preflight contracts to require signer provenance evidence")
if contracts.get("signer_provenance_sha256_required") is not True:
    raise SystemExit("expected deployment preflight contracts to require signer provenance sha256 evidence")
if contracts.get("signer_key_source_contract_version") != "v1":
    raise SystemExit("expected deployment preflight contracts signer key-source contract version marker")
if contracts.get("signer_key_source") != "managed-external":
    raise SystemExit("expected deployment preflight contracts signer key-source marker")
if contracts.get("required_signer_key_source_for_production") != "managed-external":
    raise SystemExit("expected deployment preflight contracts required production signer key-source marker")
if contracts.get("signer_key_source_production_requirement_reason_code") != "signer_key_source_production_managed_external_required":
    raise SystemExit("expected deployment preflight contracts production signer key-source reason-code marker")
if contracts.get("signer_key_source_allowed_for_ops_primary") != ["managed-external"]:
    raise SystemExit("expected deployment preflight contracts ops-primary signer key-source allowlist marker")
if contracts.get("signer_key_source_allowed_for_ops_secondary") != ["managed-external"]:
    raise SystemExit("expected deployment preflight contracts ops-secondary signer key-source allowlist marker")
if contracts.get("signer_rotation_freshness_max_delta") != 2:
    raise SystemExit("expected deployment preflight contracts signer rotation freshness max delta marker")
if contracts.get("signer_rotation_stale_rejected") is not True:
    raise SystemExit("expected deployment preflight contracts signer rotation stale rejection marker")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
missing_secret_exit_code=$?
set -e

if [ "$missing_secret_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when signer secret is missing" >&2
  exit 1
fi

if ! grep -q "signer secret env is required for selected profile" "$TMP_ERR"; then
  echo "expected deterministic missing signer secret message from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --required-approvals 1 \
  --received-approvals 1 \
  --custody-evidence-file "$TMP_CUSTODY" \
  --quorum-evidence-file "$TMP_QUORUM_SINGLE" \
  --signer-provenance-file "$TMP_PROVENANCE" \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
minimum_quorum_exit_code=$?
set -e

if [ "$minimum_quorum_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when required approvals are below production multi-signer minimum" >&2
  exit 1
fi

if ! grep -q "required approvals must be at least 2 for production signer profiles" "$TMP_ERR"; then
  echo "expected deterministic production multi-signer minimum quorum message from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --required-approvals 2 \
  --received-approvals 1 \
  --custody-evidence-file "$TMP_CUSTODY" \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
quorum_shortfall_exit_code=$?
set -e

if [ "$quorum_shortfall_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when signer quorum is below threshold" >&2
  exit 1
fi

if ! grep -q "signer quorum approvals below required threshold" "$TMP_ERR"; then
  echo "expected deterministic signer quorum shortfall message from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK="2222222222222222222222222222222222222222222222222222222222222222" \
bash "$RUNNER" \
  --mode run \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
fallback_secret_exit_code=$?
set -e

if [ "$fallback_secret_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when fallback signer secret env is present" >&2
  exit 1
fi

if ! grep -q "fallback signer secret env must not be set" "$TMP_ERR"; then
  echo "expected deterministic fallback signer secret rejection message from deployment preflight lane" >&2
  exit 1
fi

if ! grep -q "remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK" "$TMP_ERR"; then
  echo "expected deterministic fallback signer remediation marker from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --required-approvals 2 \
  --received-approvals 2 \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
missing_custody_exit_code=$?
set -e

if [ "$missing_custody_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when custody evidence is missing" >&2
  exit 1
fi

if ! grep -q "signer custody evidence file is required for selected profile" "$TMP_ERR"; then
  echo "expected deterministic missing custody evidence message from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --required-approvals 2 \
  --received-approvals 2 \
  --custody-evidence-file "$TMP_CUSTODY" \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
missing_quorum_evidence_exit_code=$?
set -e

if [ "$missing_quorum_evidence_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when signer quorum evidence is missing" >&2
  exit 1
fi

if ! grep -q "signer quorum evidence file is required for selected profile" "$TMP_ERR"; then
  echo "expected deterministic missing signer quorum evidence message from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --required-approvals 2 \
  --received-approvals 2 \
  --custody-evidence-file "$TMP_CUSTODY" \
  --quorum-evidence-file "$TMP_QUORUM" \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
missing_provenance_exit_code=$?
set -e

if [ "$missing_provenance_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when signer provenance evidence is missing" >&2
  exit 1
fi

if ! grep -q "signer provenance evidence file is required for selected profile" "$TMP_ERR"; then
  echo "expected deterministic missing signer provenance evidence message from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --required-approvals 2 \
  --received-approvals 2 \
  --custody-evidence-file "$TMP_CUSTODY" \
  --quorum-evidence-file "$TMP_QUORUM" \
  --signer-provenance-file "$TMP_PROVENANCE" \
  --signer-key-source env-local \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
production_key_source_exit_code=$?
set -e

if [ "$production_key_source_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when production signer key-source is not managed-external" >&2
  exit 1
fi

if ! grep -q "production signer profiles require signer key source managed-external" "$TMP_ERR"; then
  echo "expected deterministic production signer key-source requirement message from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --required-approvals 2 \
  --received-approvals 2 \
  --custody-evidence-file "$TMP_CUSTODY" \
  --quorum-evidence-file "$TMP_QUORUM" \
  --signer-provenance-file "$TMP_PROVENANCE" \
  --signer-rotation-epoch 8 \
  --signer-previous-rotation-epoch 3 \
  --signer-rotation-freshness-max-delta 2 \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
stale_rotation_exit_code=$?
set -e

if [ "$stale_rotation_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when signer rotation metadata is stale" >&2
  exit 1
fi

if ! grep -q "signer rotation metadata exceeded freshness threshold" "$TMP_ERR"; then
  echo "expected deterministic signer rotation stale message from deployment preflight lane" >&2
  exit 1
fi

KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --required-approvals 2 \
  --received-approvals 2 \
  --custody-evidence-file "$TMP_CUSTODY" \
  --quorum-evidence-file "$TMP_QUORUM" \
  --signer-provenance-file "$TMP_PROVENANCE" \
  --output-json "$TMP_SUMMARY" >/dev/null

python3 - "$TMP_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if summary.get("status") != "ok":
    raise SystemExit("expected deployment preflight run summary status ok")
if summary.get("reason_code") != "deployment_preflight_passed":
    raise SystemExit("expected deployment preflight run summary pass reason code")
if summary.get("required_approvals") != 2 or summary.get("received_approvals") != 2:
    raise SystemExit("expected deployment preflight run summary to capture signer quorum counts")
if summary.get("quorum_evidence_present") is not True:
    raise SystemExit("expected deployment preflight run summary quorum evidence marker true")
if summary.get("quorum_evidence_sha256_valid") is not True:
    raise SystemExit("expected deployment preflight run summary quorum evidence sha256 marker true")
if summary.get("quorum_evidence_schema_valid") is not True:
    raise SystemExit("expected deployment preflight run summary quorum evidence schema marker true")
if summary.get("quorum_evidence_matches_threshold") is not True:
    raise SystemExit("expected deployment preflight run summary quorum threshold marker true")
if summary.get("quorum_evidence_custody_sha256_match") is not True:
    raise SystemExit("expected deployment preflight run summary quorum custody sha256 match marker true")
if summary.get("quorum_evidence_signer_roles_present") is not True:
    raise SystemExit("expected deployment preflight run summary signer-roles metadata marker true")
if summary.get("quorum_evidence_signer_roles_valid") is not True:
    raise SystemExit("expected deployment preflight run summary signer-roles metadata validity marker true")
if summary.get("quorum_evidence_rotation_metadata_present") is not True:
    raise SystemExit("expected deployment preflight run summary rotation metadata marker true")
if summary.get("quorum_evidence_rotation_metadata_valid") is not True:
    raise SystemExit("expected deployment preflight run summary rotation metadata validity marker true")
if summary.get("custody_evidence_present") is not True:
    raise SystemExit("expected deployment preflight run summary custody evidence marker true")
if summary.get("signer_provenance_present") is not True:
    raise SystemExit("expected deployment preflight run summary signer provenance marker true")
if summary.get("signer_provenance_sha256_valid") is not True:
    raise SystemExit("expected deployment preflight run summary signer provenance sha256 marker true")
if summary.get("signer_rotation_fresh") is not True:
    raise SystemExit("expected deployment preflight run summary signer rotation freshness marker true")
PY

echo "local Kolme live deployment preflight lane tests passed."

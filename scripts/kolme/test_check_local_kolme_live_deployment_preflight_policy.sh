#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_QUORUM_MINIMUM="$TMP_DIR/quorum-minimum-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_SUMMARY="$TMP_DIR/summary.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local Kolme live deployment preflight policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops docs to reference deployment preflight policy checker command" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$CI_DOC_FILE"; then
  echo "expected CI strategy docs to reference deployment preflight policy checker command" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$README_FILE"; then
  echo "expected README to reference deployment preflight policy checker command" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-live-deployment-preflight-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": false,
  "ci_fast_gate_eligible": true,
  "elapsed_seconds": 0,
  "max_seconds": 12,
  "budget_status": "not_run",
  "runtime_mode": "kolme-live",
  "signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
  "signer_profile": "ops-primary",
  "signer_profile_class": "production",
  "signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
  "fallback_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
  "fallback_signer_secret_remediation": "unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
  "signer_secret_present": false,
  "fallback_signer_secret_present": false,
  "signer_secret_hex_valid": false,
  "required_approvals": 2,
  "received_approvals": 0,
  "quorum_evidence_file": "",
  "quorum_evidence_present": false,
  "quorum_evidence_sha256": "",
  "quorum_evidence_sha256_valid": false,
  "quorum_evidence_schema_valid": false,
  "quorum_evidence_approval_count": 0,
  "quorum_evidence_signers_unique": false,
  "quorum_evidence_matches_threshold": false,
  "quorum_evidence_custody_sha256_match": false,
  "quorum_evidence_signer_roles_present": false,
  "quorum_evidence_signer_roles_valid": false,
  "quorum_evidence_rotation_metadata_present": false,
  "quorum_evidence_rotation_metadata_valid": false,
  "runtime_signer_attestation_schema_version": "kamn.kolme.runtime-signer-attestation.v1",
  "runtime_signer_attestation_bundle": {
    "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
    "required_approvals": 2,
    "approved_signers": [
      "ops-primary",
      "ops-secondary"
    ],
    "signer_profile": "ops-primary",
    "signer_key_source": "env-local"
  },
  "runtime_signer_attestation_profile_approved": true,
  "custody_evidence_file": "",
  "custody_evidence_present": false,
  "custody_evidence_sha256": "",
  "custody_evidence_sha256_valid": false,
  "signer_provenance_file": "",
  "signer_provenance_present": false,
  "signer_provenance_sha256": "",
  "signer_provenance_sha256_valid": false,
  "signer_key_source_contract_version": "v1",
  "signer_key_source": "env-local",
  "signer_rotation_epoch": 1,
  "signer_previous_rotation_epoch": 1,
  "signer_rotation_freshness_max_delta": 2,
  "signer_rotation_delta_epochs": 0,
  "signer_rotation_fresh": false,
  "contracts": {
    "ci_fast_gate_scope": "ci-fast-gate",
    "required_runtime_mode": "kolme-live",
    "signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
    "supported_signer_profiles": [
      "ops-primary",
      "ops-secondary"
    ],
    "primary_signer_secret_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    "secondary_signer_secret_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
    "fallback_signer_secret_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
    "fallback_signer_secret_rejected_profile_class": "production",
    "fallback_signer_secret_rejected_profiles": [
      "ops-primary",
      "ops-secondary"
    ],
    "fallback_signer_secret_remediation": "unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
    "fallback_signer_secret_rejection_reason_code": "fallback_signer_secret_present_violation",
    "fallback_signer_secret_checkpoint_reason_code": "checkpoint_failed_fallback_private_key_contract",
    "fallback_private_key_path_allowed": false,
    "required_secret_hex_length": 64,
    "secret_source": "env",
    "approval_quorum_minimum": 2,
    "approval_quorum_required": 2,
    "approval_quorum_source": "local-operator-attestations",
    "quorum_evidence_required": true,
    "quorum_evidence_sha256_required": true,
    "quorum_evidence_schema_version": "kamn.kolme.runtime-signer-attestation.v1",
    "quorum_evidence_signer_uniqueness_required": true,
    "quorum_evidence_custody_sha256_match_required": true,
    "quorum_evidence_signer_roles_required": true,
    "quorum_evidence_signer_roles_allowed": [
      "primary",
      "secondary"
    ],
    "quorum_evidence_rotation_metadata_required": true,
    "quorum_evidence_rotation_metadata_positive_epochs_required": true,
    "quorum_evidence_source": "operator-attestation-bundle",
    "runtime_signer_attestation_schema_version": "kamn.kolme.runtime-signer-attestation.v1",
    "runtime_signer_attestation_signer_uniqueness_required": true,
    "runtime_signer_attestation_threshold_required": true,
    "runtime_signer_attestation_profile_membership_required": true,
    "runtime_signer_attestation_required_approvals": 2,
    "custody_evidence_required": true,
    "custody_evidence_sha256_required": true,
    "signer_provenance_required": true,
    "signer_provenance_sha256_required": true,
    "signer_key_source_contract_version": "v1",
    "signer_key_source": "env-local",
    "signer_key_source_allowed_for_ops_primary": [
      "env-local",
      "managed-external"
    ],
    "signer_key_source_allowed_for_ops_secondary": [
      "env-local"
    ],
    "signer_rotation_freshness_max_delta": 2,
    "signer_rotation_stale_rejected": true
  },
  "checks": [
    {
      "id": "runtime_mode_contract",
      "command": "runtime-mode must equal kolme-live",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "signer_profile_contract",
      "command": "signer profile must be ops-primary or ops-secondary",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "signer_secret_contract",
      "command": "selected signer secret env must exist and be 64-char hex",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "fallback_private_key_contract",
      "command": "fallback signer secret env must remain unset",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "signer_quorum_contract",
      "command": "received approvals must satisfy required approvals threshold",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "quorum_evidence_contract",
      "command": "quorum evidence bundle must satisfy schema, signer uniqueness, threshold, and custody digest match",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "custody_evidence_contract",
      "command": "signer custody evidence file and sha256 marker must be present",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "signer_provenance_contract",
      "command": "signer provenance evidence file and sha256 marker must be present",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "signer_rotation_freshness_contract",
      "command": "signer rotation metadata must satisfy freshness threshold",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": []
}
JSON

python3 "$CHECKER" \
  --report-file "$TMP_REPORT_OK" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$TMP_POLICY_OUT" >/dev/null

python3 - "$TMP_POLICY_OUT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-policy-report.v1":
    raise SystemExit("unexpected deployment preflight policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid deployment preflight report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no reason codes for valid deployment preflight report")
PY

python3 - "$TMP_REPORT_OK" "$TMP_REPORT_QUORUM_MINIMUM" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
report["required_approvals"] = 1
report["received_approvals"] = 0
bundle = dict(report.get("runtime_signer_attestation_bundle", {}))
bundle["required_approvals"] = 1
report["runtime_signer_attestation_bundle"] = bundle
contracts = dict(report.get("contracts", {}))
contracts["approval_quorum_required"] = 1
contracts["runtime_signer_attestation_required_approvals"] = 1
report["contracts"] = contracts
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(report, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_QUORUM_MINIMUM" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
quorum_minimum_exit_code=$?
set -e

if [ "$quorum_minimum_exit_code" -eq 0 ]; then
  echo "expected deployment preflight policy checker to fail when production required approvals drop below multi-signer minimum" >&2
  exit 1
fi

if ! grep -q "signer_quorum_minimum_not_met" "$TMP_ERR"; then
  echo "expected signer_quorum_minimum_not_met reason for deployment preflight policy failure" >&2
  exit 1
fi

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-live-deployment-preflight-summary.v1",
  "mode": "run",
  "status": "ok",
  "reason_code": "deployment_preflight_passed",
  "local_only_enforced": false,
  "ci_fast_gate_eligible": false,
  "elapsed_seconds": 1,
  "max_seconds": 12,
  "budget_status": "within_budget",
  "runtime_mode": "kolme-standard",
  "signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
  "signer_profile": "legacy",
  "signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
  "fallback_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
  "signer_secret_present": true,
  "fallback_signer_secret_present": true,
  "signer_secret_hex_valid": true,
  "required_approvals": 2,
  "received_approvals": 1,
  "quorum_evidence_file": "",
  "quorum_evidence_present": false,
  "quorum_evidence_sha256": "",
  "quorum_evidence_sha256_valid": false,
  "quorum_evidence_schema_valid": false,
  "quorum_evidence_approval_count": 0,
  "quorum_evidence_signers_unique": false,
  "quorum_evidence_matches_threshold": false,
  "quorum_evidence_custody_sha256_match": false,
  "runtime_signer_attestation_schema_version": "",
  "runtime_signer_attestation_bundle": {
    "schema_version": "kamn.kolme.runtime-signer-attestation.v0",
    "required_approvals": 3,
    "approved_signers": [
      "ops-primary",
      "ops-primary"
    ],
    "signer_profile": "ops-primary",
    "signer_key_source": "legacy-local"
  },
  "runtime_signer_attestation_profile_approved": false,
  "custody_evidence_file": "",
  "custody_evidence_present": false,
  "custody_evidence_sha256": "",
  "custody_evidence_sha256_valid": false,
  "signer_provenance_file": "",
  "signer_provenance_present": false,
  "signer_provenance_sha256": "",
  "signer_provenance_sha256_valid": false,
  "signer_key_source_contract_version": "v0",
  "signer_key_source": "legacy-local",
  "signer_rotation_epoch": 7,
  "signer_previous_rotation_epoch": 1,
  "signer_rotation_freshness_max_delta": 2,
  "signer_rotation_delta_epochs": 6,
  "signer_rotation_fresh": false,
  "contracts": {
    "ci_fast_gate_scope": "local-only",
    "required_runtime_mode": "kolme-live",
    "signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
    "supported_signer_profiles": [
      "ops-primary",
      "ops-secondary"
    ],
    "primary_signer_secret_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    "secondary_signer_secret_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
    "fallback_signer_secret_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
    "fallback_private_key_path_allowed": true,
    "required_secret_hex_length": 64,
    "secret_source": "env",
    "approval_quorum_required": 2,
    "approval_quorum_source": "local-operator-attestations",
    "quorum_evidence_required": true,
    "quorum_evidence_sha256_required": true,
    "quorum_evidence_schema_version": "kamn.kolme.runtime-signer-attestation.v1",
    "quorum_evidence_signer_uniqueness_required": true,
    "quorum_evidence_custody_sha256_match_required": true,
    "quorum_evidence_source": "operator-attestation-bundle",
    "runtime_signer_attestation_schema_version": "kamn.kolme.runtime-signer-attestation.v0",
    "runtime_signer_attestation_signer_uniqueness_required": false,
    "runtime_signer_attestation_threshold_required": false,
    "runtime_signer_attestation_profile_membership_required": false,
    "runtime_signer_attestation_required_approvals": 1,
    "custody_evidence_required": true,
    "custody_evidence_sha256_required": true,
    "signer_provenance_required": false,
    "signer_provenance_sha256_required": false,
    "signer_key_source_contract_version": "v0",
    "signer_key_source": "legacy-local",
    "signer_key_source_allowed_for_ops_primary": [
      "legacy-local"
    ],
    "signer_key_source_allowed_for_ops_secondary": [
      "legacy-local"
    ],
    "signer_rotation_freshness_max_delta": 4,
    "signer_rotation_stale_rejected": false
  },
  "checks": [
    {
      "id": "runtime_mode_contract",
      "command": "runtime-mode must equal kolme-live",
      "status": "pass",
      "reason_code": "runtime_mode_validated"
    }
  ],
  "artifact_paths": []
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_BAD" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code deployment_preflight_passed \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
bad_exit_code=$?
set -e

if [ "$bad_exit_code" -eq 0 ]; then
  echo "expected deployment preflight policy checker to fail for invalid report markers" >&2
  exit 1
fi

if ! grep -q "runtime_mode_mismatch" "$TMP_ERR"; then
  echo "expected runtime mode mismatch reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "signer_profile_mismatch" "$TMP_ERR"; then
  echo "expected signer profile mismatch reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "ci_fast_gate_eligibility_violation" "$TMP_ERR"; then
  echo "expected fast-gate eligibility violation reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:signer_secret_contract" "$TMP_ERR"; then
  echo "expected missing signer_secret_contract check reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:signer_provenance_contract" "$TMP_ERR"; then
  echo "expected missing signer_provenance_contract check reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "fallback_signer_secret_present_violation" "$TMP_ERR"; then
  echo "expected fallback signer secret presence violation reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "fallback_signer_secret_checkpoint_reason_mismatch" "$TMP_ERR"; then
  echo "expected fallback signer secret checkpoint reason mismatch for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "fallback_signer_secret_remediation_missing" "$TMP_ERR"; then
  echo "expected fallback signer secret remediation missing reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:fallback_private_key_contract" "$TMP_ERR"; then
  echo "expected missing fallback_private_key_contract check reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "signer_quorum_shortfall" "$TMP_ERR"; then
  echo "expected signer quorum shortfall reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "custody_evidence_missing" "$TMP_ERR"; then
  echo "expected custody evidence missing reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:signer_quorum_contract" "$TMP_ERR"; then
  echo "expected missing signer_quorum_contract check reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:quorum_evidence_contract" "$TMP_ERR"; then
  echo "expected missing quorum_evidence_contract check reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:custody_evidence_contract" "$TMP_ERR"; then
  echo "expected missing custody_evidence_contract check reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:signer_rotation_freshness_contract" "$TMP_ERR"; then
  echo "expected missing signer_rotation_freshness_contract check reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "signer_key_source_contract_version_mismatch" "$TMP_ERR"; then
  echo "expected signer key-source contract version mismatch reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "signer_key_source_invalid" "$TMP_ERR"; then
  echo "expected signer key-source invalid reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "signer_provenance_missing" "$TMP_ERR"; then
  echo "expected signer provenance missing reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "signer_rotation_epoch_stale" "$TMP_ERR"; then
  echo "expected signer rotation stale reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "quorum_evidence_missing" "$TMP_ERR"; then
  echo "expected quorum evidence missing reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "quorum_evidence_signer_roles_missing" "$TMP_ERR"; then
  echo "expected quorum evidence signer-roles missing reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "quorum_evidence_rotation_metadata_missing" "$TMP_ERR"; then
  echo "expected quorum evidence rotation metadata missing reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_signer_attestation_schema_version_missing" "$TMP_ERR"; then
  echo "expected runtime signer attestation schema version missing reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_signer_attestation_schema_invalid" "$TMP_ERR"; then
  echo "expected runtime signer attestation schema invalid reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_signer_attestation_approved_signers_not_unique" "$TMP_ERR"; then
  echo "expected runtime signer attestation duplicate signer reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_signer_attestation_quorum_shortfall" "$TMP_ERR"; then
  echo "expected runtime signer attestation quorum shortfall reason for deployment preflight policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_signer_attestation_profile_not_approved" "$TMP_ERR"; then
  echo "expected runtime signer attestation profile membership reason for deployment preflight policy failure" >&2
  exit 1
fi

if [ ! -x "$RUNNER" ]; then
  echo "expected local Kolme live deployment preflight lane runner to be executable" >&2
  exit 1
fi

bash "$RUNNER" \
  --mode dry-run \
  --output-json "$TMP_SUMMARY" >/dev/null

python3 "$CHECKER" \
  --report-file "$TMP_SUMMARY" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$TMP_POLICY_OUT" >/dev/null

echo "local Kolme live deployment preflight policy checker tests passed."

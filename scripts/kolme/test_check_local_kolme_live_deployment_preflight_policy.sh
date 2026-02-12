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
  "signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
  "fallback_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
  "signer_secret_present": false,
  "fallback_signer_secret_present": false,
  "signer_secret_hex_valid": false,
  "required_approvals": 2,
  "received_approvals": 0,
  "custody_evidence_file": "",
  "custody_evidence_present": false,
  "custody_evidence_sha256": "",
  "custody_evidence_sha256_valid": false,
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
    "fallback_private_key_path_allowed": false,
    "required_secret_hex_length": 64,
    "secret_source": "env",
    "approval_quorum_required": 2,
    "approval_quorum_source": "local-operator-attestations",
    "custody_evidence_required": true,
    "custody_evidence_sha256_required": true
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
      "id": "custody_evidence_contract",
      "command": "signer custody evidence file and sha256 marker must be present",
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
  "custody_evidence_file": "",
  "custody_evidence_present": false,
  "custody_evidence_sha256": "",
  "custody_evidence_sha256_valid": false,
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
    "custody_evidence_required": true,
    "custody_evidence_sha256_required": true
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

if ! grep -q "fallback_signer_secret_present_violation" "$TMP_ERR"; then
  echo "expected fallback signer secret presence violation reason for deployment preflight policy failure" >&2
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

if ! grep -q "check_missing:custody_evidence_contract" "$TMP_ERR"; then
  echo "expected missing custody_evidence_contract check reason for deployment preflight policy failure" >&2
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

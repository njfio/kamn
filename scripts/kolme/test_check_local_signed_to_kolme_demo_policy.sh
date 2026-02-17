#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_signed_to_kolme_demo_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
SECURITY_DOC="$ROOT_DIR/docs/security/key-management.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/signed_to_kolme_ok.json"
TMP_REPORT_MISSING_SUBMIT="$TMP_DIR/signed_to_kolme_missing_submit.json"
TMP_REPORT_MISSING_FINALITY="$TMP_DIR/signed_to_kolme_missing_finality.json"
TMP_REPORT_TAXONOMY_DRIFT="$TMP_DIR/signed_to_kolme_taxonomy_drift.json"
TMP_REPORT_NORMALIZED_DRIFT="$TMP_DIR/signed_to_kolme_normalized_drift.json"
TMP_REPORT_SIMULATED_SIGNING="$TMP_DIR/signed_to_kolme_simulated_signing.json"
TMP_REPORT_MISSING_NATIVE_SIGNING_MARKER="$TMP_DIR/signed_to_kolme_missing_native_signing_marker.json"
TMP_POLICY_OK="$TMP_DIR/signed_to_kolme_policy_ok.json"
TMP_POLICY_BAD="$TMP_DIR/signed_to_kolme_policy_bad.json"
TMP_ERR="$TMP_DIR/policy_error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local signed-to-Kolme demo policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "check_local_signed_to_kolme_demo_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops docs to reference signed-to-Kolme policy checker command" >&2
  exit 1
fi

if ! grep -q "check_local_signed_to_kolme_demo_policy.py" "$README_FILE"; then
  echo "expected README to reference signed-to-Kolme policy checker command" >&2
  exit 1
fi

if [ ! -f "$SECURITY_DOC" ]; then
  echo "expected key management security doc to exist" >&2
  exit 1
fi

if ! grep -q "native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1" "$SECURITY_DOC"; then
  echo "expected key management security doc to include native signer taxonomy marker" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-signed-to-kolme-demo-summary.v1",
  "mode": "run",
  "status": "ok",
  "reason_code": "signed_to_kolme_demo_passed",
  "local_only_enforced": true,
  "elapsed_seconds": 12,
  "max_seconds": 240,
  "budget_status": "within_budget",
  "runtime_commit_submit_evidence_marker": "status=submitted",
  "runtime_commit_submit_evidence_marker_present": true,
  "runtime_commit_finality_evidence_marker": "finality=final",
  "runtime_commit_finality_evidence_marker_present": true,
  "runtime_commit_submit_finality_contract_version": "v1",
  "runtime_commit_submit_finality_linked": true,
  "runtime_commit_live_status": "ok",
  "runtime_commit_live_reason_code": "live_runtime_commit_and_finality_commands_passed",
  "runtime_commit_live_summary_path": "/tmp/kolme-local-runtime-commit-live-summary.json",
  "runtime_commit_live_policy_report_path": "/tmp/kolme-local-runtime-commit-live-policy.json",
  "runtime_signing_profile_contract_version": "v1",
  "runtime_signing_profile": "kolme-fork-secp256k1-v1",
  "reason_taxonomy": {
    "schema_version": "kamn.kolme.local-signed-to-kolme-demo.reason-taxonomy.v1",
    "overall": "demo.success",
    "signed_demo_checkpoint": "checkpoint.pass",
    "signed_integration_checkpoint": "checkpoint.pass",
    "runtime_integration_checkpoint": "checkpoint.pass",
    "runtime_commit_live": "runtime_commit.success"
  },
  "normalized_evidence": {
    "schema_version": "kamn.kolme.local-signed-to-kolme-demo.evidence-normalization.v1",
    "primary_check_order": [
      "localhost_signed_demo_contract",
      "localhost_signed_integration_contract",
      "local_kamn_runtime_integration_run"
    ],
    "checks_by_id": {
      "localhost_signed_demo_contract": {
        "status": "pass",
        "reason_code": "localhost_signed_demo_contract_passed",
        "command": "bash scripts/sdk/run_localhost_signed_demo_contract_lane.sh"
      },
      "localhost_signed_integration_contract": {
        "status": "pass",
        "reason_code": "localhost_signed_integration_contract_passed",
        "command": "bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
      },
      "local_kamn_runtime_integration_run": {
        "status": "pass",
        "reason_code": "local_kamn_runtime_integration_run_passed",
        "command": "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run"
      }
    }
  },
  "checks": [
    {
      "id": "localhost_signed_demo_contract",
      "command": "bash scripts/sdk/run_localhost_signed_demo_contract_lane.sh",
      "status": "pass",
      "reason_code": "localhost_signed_demo_contract_passed"
    },
    {
      "id": "localhost_signed_integration_contract",
      "command": "bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh",
      "status": "pass",
      "reason_code": "localhost_signed_integration_contract_passed"
    },
    {
      "id": "local_kamn_runtime_integration_run",
      "command": "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run",
      "status": "pass",
      "reason_code": "local_kamn_runtime_integration_run_passed"
    }
  ],
  "artifact_paths": [
    "/tmp/localhost-signed-demo-contract-report.json",
    "/tmp/localhost-signed-integration-contract-report.json",
    "/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
    "/tmp/kolme-local-kamn-live-runtime-integration-policy.json",
    "/tmp/kolme-local-runtime-commit-live-summary.json",
    "/tmp/kolme-local-runtime-commit-live-policy.json",
    "/tmp/kolme-local-runtime-commit-endpoint-output.txt",
    "/tmp/kolme-local-runtime-commit-live-finality-output.txt"
  ]
}
JSON

python3 "$CHECKER" \
  --report-file "$TMP_REPORT_OK" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code signed_to_kolme_demo_passed \
  --output-json "$TMP_POLICY_OK" >/dev/null

python3 - "$TMP_POLICY_OK" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-signed-to-kolme-demo-policy-report.v1":
    raise SystemExit("unexpected signed-to-Kolme policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid signed-to-Kolme report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no reason codes for valid signed-to-Kolme report")
if report.get("native_signer_reason_taxonomy_version") != "kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1":
    raise SystemExit("expected deterministic native_signer_reason_taxonomy_version in policy report")
if report.get("native_signer_reason_codes_csv") != "runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch":
    raise SystemExit("expected deterministic native_signer_reason_codes_csv in policy report")
if report.get("native_signer_reason_codes_value") != "none":
    raise SystemExit("expected native_signer_reason_codes_value=none in GO policy report")
PY

python3 - "$TMP_REPORT_OK" "$TMP_REPORT_SIMULATED_SIGNING" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
report["runtime_signing_profile"] = "simulated"
for check in report.get("checks", []):
    if isinstance(check, dict) and check.get("id") == "local_kamn_runtime_integration_run":
        check["command"] = (
            "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated "
            "bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run"
        )
normalized = report.get("normalized_evidence", {}).get("checks_by_id", {}).get("local_kamn_runtime_integration_run")
if isinstance(normalized, dict):
    normalized["command"] = (
        "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated "
        "bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run"
    )
pathlib.Path(sys.argv[2]).write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_SIMULATED_SIGNING" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_BAD" >"$TMP_ERR" 2>&1
simulated_signing_code=$?
set -e

if [ "$simulated_signing_code" -eq 0 ]; then
  echo "expected checker to fail when simulated signing profile is accepted in run-mode evidence" >&2
  exit 1
fi
if ! grep -q "runtime_commit_simulated_signing_profile_detected" "$TMP_ERR"; then
  echo "expected runtime_commit_simulated_signing_profile_detected reason code" >&2
  exit 1
fi
if ! grep -q "runtime_signing_profile_mismatch" "$TMP_ERR"; then
  echo "expected runtime_signing_profile_mismatch reason code" >&2
  exit 1
fi

python3 - "$TMP_REPORT_OK" "$TMP_REPORT_MISSING_NATIVE_SIGNING_MARKER" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
for check in report.get("checks", []):
    if isinstance(check, dict) and check.get("id") == "local_kamn_runtime_integration_run":
        check["command"] = "bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run"
normalized = report.get("normalized_evidence", {}).get("checks_by_id", {}).get("local_kamn_runtime_integration_run")
if isinstance(normalized, dict):
    normalized["command"] = "bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run"
pathlib.Path(sys.argv[2]).write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_MISSING_NATIVE_SIGNING_MARKER" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_BAD" >"$TMP_ERR" 2>&1
missing_native_signing_marker_code=$?
set -e

if [ "$missing_native_signing_marker_code" -eq 0 ]; then
  echo "expected checker to fail when native signing profile marker is missing from run-mode runtime evidence command" >&2
  exit 1
fi
if ! grep -q "runtime_commit_native_signing_profile_marker_missing" "$TMP_ERR"; then
  echo "expected runtime_commit_native_signing_profile_marker_missing reason code" >&2
  exit 1
fi

python3 - "$TMP_REPORT_OK" "$TMP_REPORT_MISSING_SUBMIT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
report["runtime_commit_submit_evidence_marker_present"] = False
pathlib.Path(sys.argv[2]).write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_MISSING_SUBMIT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_BAD" >"$TMP_ERR" 2>&1
missing_submit_code=$?
set -e

if [ "$missing_submit_code" -eq 0 ]; then
  echo "expected checker to fail when runtime commit submit evidence marker is missing" >&2
  exit 1
fi
if ! grep -q "runtime_commit_submit_evidence_marker_missing" "$TMP_ERR"; then
  echo "expected runtime_commit_submit_evidence_marker_missing reason code" >&2
  exit 1
fi

# Regression: #2388
python3 - "$TMP_REPORT_OK" "$TMP_REPORT_MISSING_FINALITY" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
report["runtime_commit_finality_evidence_marker_present"] = False
report["runtime_commit_submit_finality_linked"] = False
pathlib.Path(sys.argv[2]).write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_MISSING_FINALITY" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_BAD" >"$TMP_ERR" 2>&1
missing_finality_code=$?
set -e

if [ "$missing_finality_code" -eq 0 ]; then
  echo "expected checker to fail when runtime commit finality evidence marker is missing" >&2
  exit 1
fi
if ! grep -q "runtime_commit_finality_evidence_marker_missing" "$TMP_ERR"; then
  echo "expected runtime_commit_finality_evidence_marker_missing reason code" >&2
  exit 1
fi
if ! grep -q "runtime_commit_submit_finality_linkage_missing" "$TMP_ERR"; then
  echo "expected runtime_commit_submit_finality_linkage_missing reason code" >&2
  exit 1
fi

# Regression: #4498
python3 - "$TMP_REPORT_OK" "$TMP_REPORT_TAXONOMY_DRIFT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
report["reason_taxonomy"]["overall"] = "demo.not_run"
pathlib.Path(sys.argv[2]).write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_TAXONOMY_DRIFT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_BAD" >"$TMP_ERR" 2>&1
taxonomy_drift_code=$?
set -e

if [ "$taxonomy_drift_code" -eq 0 ]; then
  echo "expected checker to fail when reason taxonomy output drifts" >&2
  exit 1
fi
if ! grep -q "reason_taxonomy_overall_mismatch" "$TMP_ERR"; then
  echo "expected reason_taxonomy_overall_mismatch reason code" >&2
  exit 1
fi

python3 - "$TMP_REPORT_OK" "$TMP_REPORT_NORMALIZED_DRIFT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
report["normalized_evidence"]["checks_by_id"]["local_kamn_runtime_integration_run"]["status"] = "fail"
pathlib.Path(sys.argv[2]).write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_NORMALIZED_DRIFT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_BAD" >"$TMP_ERR" 2>&1
normalized_drift_code=$?
set -e

if [ "$normalized_drift_code" -eq 0 ]; then
  echo "expected checker to fail when normalized evidence output drifts" >&2
  exit 1
fi
if ! grep -q "normalized_evidence_status_mismatch:local_kamn_runtime_integration_run" "$TMP_ERR"; then
  echo "expected normalized_evidence_status_mismatch reason code" >&2
  exit 1
fi

echo "local signed-to-Kolme demo policy checker tests passed."

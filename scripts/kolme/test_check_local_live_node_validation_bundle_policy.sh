#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_live_node_validation_bundle_policy.py"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_live_node_validation_bundle_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_INTEGRATION_POLICY="$TMP_DIR/integration-policy-report.json"
TMP_SUMMARY="$TMP_DIR/bundle-summary.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local live-node validation bundle policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "check_local_live_node_validation_bundle_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops docs to reference local live-node validation bundle policy checker" >&2
  exit 1
fi

if ! grep -q "check_local_live_node_validation_bundle_policy.py" "$CI_DOC_FILE"; then
  echo "expected CI strategy docs to reference local live-node validation bundle policy checker" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-live-node-validation-bundle-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "ci_fast_gate_eligible": false,
  "elapsed_seconds": 0,
  "max_seconds": 480,
  "budget_status": "not_run",
  "checkout_path": "/tmp/kolme_fork",
  "expected_remote_url": "https://github.com/njfio/kolme_fork.git",
  "expected_ref": "refs/heads/main",
  "base_url": "http://127.0.0.1:3000",
  "fork_chain_version": "v0.15.2",
  "integration_command": "KAMN_KOLME_LOCAL_HEAVY=1 KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json",
  "integration_policy_command": "python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json",
  "process_lifecycle_command": "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --integration-runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json",
  "process_lifecycle_policy_command": "python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json",
  "integration_report": "/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
  "integration_policy_report": "/tmp/kolme-local-kamn-live-runtime-integration-policy.json",
  "integration_runtime_policy_report": "/tmp/kolme-local-runtime-commit-live-policy.json",
  "integration_runtime_commit_live_summary": "/tmp/kolme-local-runtime-commit-live-summary.json",
  "process_lifecycle_report": "/tmp/kolme-local-fork-process-lifecycle-summary.json",
  "process_lifecycle_policy_report": "/tmp/kolme-local-fork-process-lifecycle-policy.json",
  "rollback_evidence_file": "/tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json",
  "recovery_evidence_file": "/tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json",
  "checks": [
    {
      "id": "integration_bundle",
      "command": "KAMN_KOLME_LOCAL_HEAVY=1 KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "integration_policy",
      "command": "python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "process_lifecycle_bundle",
      "command": "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --integration-runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "process_lifecycle_policy",
      "command": "python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": [
    "/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
    "/tmp/kolme-local-kamn-live-runtime-integration-policy.json",
    "/tmp/kolme-local-runtime-commit-live-policy.json",
    "/tmp/kolme-local-runtime-commit-live-summary.json",
    "/tmp/kolme-local-fork-process-lifecycle-summary.json",
    "/tmp/kolme-local-fork-process-lifecycle-policy.json",
    "/tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json",
    "/tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json"
  ],
  "contracts": {
    "ci_fast_gate_scope": "local-only",
    "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
    "bundle_contract": "live_node_release_bundle_v1",
    "rollback_recovery_artifact_lineage_required": true,
    "process_lifecycle_rollback_evidence_option": "--rollback-evidence-file",
    "process_lifecycle_recovery_evidence_option": "--recovery-evidence-file"
  }
}
JSON

start_epoch="$(date +%s)"
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_OK" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$TMP_POLICY_OUT" >/dev/null
elapsed_seconds="$(( $(date +%s) - start_epoch ))"

if [ "$elapsed_seconds" -gt 2 ]; then
  echo "expected local live-node validation bundle policy checker to complete in <=2 seconds for fixture input" >&2
  exit 1
fi

python3 - "$TMP_POLICY_OUT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-live-node-validation-bundle-policy-report.v1":
    raise SystemExit("unexpected local live-node validation bundle policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid local live-node validation bundle report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no reason codes for valid local live-node validation bundle report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-live-node-validation-bundle-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "ci_fast_gate_eligible": false,
  "elapsed_seconds": 0,
  "max_seconds": 480,
  "budget_status": "not_run",
  "integration_command": "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated-v1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run",
  "integration_policy_command": "python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json",
  "process_lifecycle_command": "bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run",
  "process_lifecycle_policy_command": "python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json",
  "integration_report": "/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
  "integration_policy_report": "/tmp/kolme-local-kamn-live-runtime-integration-policy.json",
  "integration_runtime_policy_report": "/tmp/kolme-local-runtime-commit-live-policy.json",
  "integration_runtime_commit_live_summary": "/tmp/kolme-local-runtime-commit-live-summary.json",
  "process_lifecycle_report": "/tmp/kolme-local-fork-process-lifecycle-summary.json",
  "process_lifecycle_policy_report": "/tmp/kolme-local-fork-process-lifecycle-policy.json",
  "checks": [
    {
      "id": "integration_bundle",
      "command": "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated-v1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "integration_policy",
      "command": "python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": [
    "/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
    "/tmp/kolme-local-kamn-live-runtime-integration-policy.json"
  ],
  "contracts": {
    "ci_fast_gate_scope": "ci-fast-gate",
    "runtime_provider_client_contract": "InMemoryKolmeRuntimeCommitClient",
    "bundle_contract": "live_node_release_bundle_v1"
  }
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_BAD" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
bad_exit_code=$?
set -e

if [ "$bad_exit_code" -eq 0 ]; then
  echo "expected policy checker to fail for local live-node validation bundle marker drift" >&2
  exit 1
fi

if ! grep -q "ci_fast_gate_scope_mismatch" "$TMP_ERR"; then
  echo "expected ci_fast_gate_scope mismatch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:process_lifecycle_bundle" "$TMP_ERR"; then
  echo "expected missing process lifecycle check reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_provider_client_contract_contract_mismatch" "$TMP_ERR"; then
  echo "expected runtime provider contract mismatch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "integration_simulated_signing_profile_detected" "$TMP_ERR"; then
  echo "expected simulated signing profile reason for policy failure" >&2
  exit 1
fi

if ! grep -q "rollback_evidence_file_missing" "$TMP_ERR"; then
  echo "expected rollback evidence lineage reason for policy failure" >&2
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
  --output-json "$TMP_INTEGRATION_POLICY" >/dev/null

python3 - "$TMP_INTEGRATION_POLICY" <<'PY'
import json
import pathlib
import sys

policy = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if policy.get("final_decision") != "GO":
    raise SystemExit("expected GO from checker when evaluating runner-generated dry-run summary")
if policy.get("reason_codes") != []:
    raise SystemExit("expected no policy reason codes for runner-generated dry-run summary")
PY

echo "local live-node validation bundle policy checker tests passed."

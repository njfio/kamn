#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_real_process_policy.py"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected real-fork local process wrapper policy checker to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-fork-real-process-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "elapsed_seconds": 0,
  "max_seconds": 360,
  "budget_status": "not_run",
  "selected_serve_command": "cd /tmp/kolme_fork && cargo run --bin example-six-sigma -- serve api-server",
  "allow_non_fork_serve_command": false,
  "contracts": {
    "default_profile": "example-six-sigma:serve-api-server",
    "expected_cargo_bin": "example-six-sigma",
    "expected_component": "api-server",
    "checkout_bootstrap_runner": "run_local_kolme_fork_checkout_bootstrap_lane.sh",
    "checkout_bootstrap_checker": "check_local_kolme_fork_checkout_bootstrap_policy.py",
    "profile_preflight_runner": "run_local_kolme_fork_profile_preflight_lane.sh",
    "profile_preflight_checker": "check_local_kolme_fork_profile_preflight_policy.py",
    "self_test_runner": "run_local_kolme_fork_self_test_lane.sh",
    "self_test_checker": "check_local_kolme_fork_self_test_policy.py",
    "lifecycle_runner": "run_local_kolme_fork_process_lifecycle_lane.sh",
    "lifecycle_checker": "check_local_kolme_fork_process_lifecycle_policy.py",
    "wrapper_lifecycle_rollback_evidence_option": "--lifecycle-rollback-evidence-file",
    "wrapper_lifecycle_recovery_evidence_option": "--lifecycle-recovery-evidence-file",
    "lifecycle_rollback_evidence_option": "--rollback-evidence-file",
    "lifecycle_recovery_evidence_option": "--recovery-evidence-file"
  },
  "lifecycle_rollback_evidence_file": "/tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json",
  "lifecycle_recovery_evidence_file": "/tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json",
  "checks": [
    {
      "id": "real_fork_command_profile",
      "command": "default profile: cargo run --bin example-six-sigma -- serve api-server",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "checkout_bootstrap_lane",
      "command": "bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh ...",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "checkout_bootstrap_policy",
      "command": "python3 scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py ...",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "profile_preflight_lane",
      "command": "bash scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh ...",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "profile_preflight_policy",
      "command": "python3 scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py ...",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "self_test_lane",
      "command": "bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh ...",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "self_test_policy",
      "command": "python3 scripts/kolme/check_local_kolme_fork_self_test_policy.py ...",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "process_lifecycle_lane",
      "command": "bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh ... --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "process_lifecycle_policy",
      "command": "python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py ...",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": [
    "/tmp/kolme-local-fork-checkout-bootstrap-summary.json",
    "/tmp/kolme-local-fork-checkout-bootstrap-policy.json",
    "/tmp/kolme-local-fork-profile-preflight-summary.json",
    "/tmp/kolme-local-fork-profile-preflight-policy.json",
    "/tmp/kolme-local-fork-self-test-summary.json",
    "/tmp/kolme-local-fork-self-test-policy.json",
    "/tmp/kolme-local-fork-process-lifecycle-summary.json",
    "/tmp/kolme-local-fork-process-lifecycle-policy.json",
    "/tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json",
    "/tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json"
  ]
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
if report.get("schema_version") != "kamn.kolme.local-fork-real-process-policy-report.v1":
    raise SystemExit("unexpected wrapper policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid wrapper report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no failed checks for valid wrapper report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-fork-real-process-summary.v1",
  "mode": "run",
  "status": "ok",
  "reason_code": "real_fork_process_wrapper_passed",
  "local_only_enforced": true,
  "elapsed_seconds": 3,
  "max_seconds": 360,
  "budget_status": "within_budget",
  "selected_serve_command": "cd /tmp/kolme_fork && cargo run --bin example-six-sigma -- serve api-server",
  "allow_non_fork_serve_command": false,
  "contracts": {
    "default_profile": "example-six-sigma:serve-api-server",
    "expected_cargo_bin": "example-six-sigma",
    "expected_component": "api-server",
    "checkout_bootstrap_runner": "run_local_kolme_fork_checkout_bootstrap_lane.sh",
    "checkout_bootstrap_checker": "missing-checker.py",
    "profile_preflight_runner": "run_local_kolme_fork_profile_preflight_lane.sh",
    "profile_preflight_checker": "check_local_kolme_fork_profile_preflight_policy.py",
    "self_test_runner": "run_local_kolme_fork_self_test_lane.sh",
    "self_test_checker": "check_local_kolme_fork_self_test_policy.py",
    "lifecycle_runner": "run_local_kolme_fork_process_lifecycle_lane.sh",
    "lifecycle_checker": "check_local_kolme_fork_process_lifecycle_policy.py"
  },
  "checks": [
    {
      "id": "real_fork_command_profile",
      "command": "default profile",
      "status": "pass",
      "reason_code": "command_profile_validated"
    },
    {
      "id": "checkout_bootstrap_lane",
      "command": "checkout bootstrap",
      "status": "pass",
      "reason_code": "checkout_bootstrap_lane_passed"
    }
  ],
  "artifact_paths": [
    "/tmp/only-one-artifact.json"
  ]
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_BAD" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
bad_exit_code=$?
set -e

if [ "$bad_exit_code" -eq 0 ]; then
  echo "expected wrapper policy checker to fail for invalid report" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$TMP_ERR"; then
  echo "expected checker output to include final_decision=NO-GO for invalid report" >&2
  exit 1
fi

if ! grep -q "checkout_bootstrap_checker_mismatch" "$TMP_ERR"; then
  echo "expected checkout_bootstrap_checker_mismatch reason marker for invalid report" >&2
  exit 1
fi

echo "real-fork local process wrapper policy checker tests passed."

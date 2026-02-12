#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_rust_test_matrix_policy.py"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork rust test matrix policy checker to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-fork-rust-test-matrix-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "elapsed_seconds": 0,
  "max_seconds_per_command": 120,
  "command_count": 2,
  "cargo_profile": "strict",
  "budget_status": "not_run",
  "evidence_bundle_schema_version": "kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v1",
  "evidence_bundle": {
    "schema_version": "kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v1",
    "summary_schema_version": "kamn.kolme.local-fork-rust-test-matrix-summary.v1",
    "status": "ok",
    "reason_code": "dry_run_no_commands_executed",
    "budget_status": "not_run",
    "command_count": 2,
    "artifact_paths": [
      "/tmp/meta.json",
      "/tmp/matrix-logs"
    ]
  },
  "checkpoints": [
    {
      "id": "fork_sync_metadata",
      "command": "echo planned",
      "status": "planned",
      "output_file": "/tmp/meta.json"
    },
    {
      "id": "matrix_command_1",
      "command": "echo c1",
      "status": "planned",
      "output_file": "/tmp/c1.log"
    },
    {
      "id": "matrix_command_2",
      "command": "echo c2",
      "status": "planned",
      "output_file": "/tmp/c2.log"
    }
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
if report.get("schema_version") != "kamn.kolme.local-fork-rust-test-matrix-policy-report.v1":
    raise SystemExit("unexpected matrix policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid dry-run report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no failed checks for valid dry-run report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-fork-rust-test-matrix-summary.v1",
  "mode": "run",
  "status": "fail",
  "reason_code": "fork_rust_test_command_timeout",
  "local_only_enforced": true,
  "elapsed_seconds": 6,
  "max_seconds_per_command": 5,
  "command_count": 1,
  "cargo_profile": "strict",
  "budget_status": "exceeded_budget",
  "evidence_bundle_schema_version": "kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v0",
  "evidence_bundle": {
    "schema_version": "kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v0",
    "summary_schema_version": "kamn.kolme.local-fork-rust-test-matrix-summary.v1",
    "status": "fail",
    "reason_code": "fork_rust_test_command_timeout",
    "budget_status": "exceeded_budget",
    "command_count": 1,
    "artifact_paths": [
      "/tmp/meta.json"
    ]
  },
  "checkpoints": [
    {
      "id": "fork_sync_metadata",
      "command": "echo pass",
      "status": "pass",
      "output_file": "/tmp/meta.json"
    },
    {
      "id": "matrix_command_1",
      "command": "sleep 2",
      "status": "fail",
      "output_file": "/tmp/c1.log"
    }
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
  echo "expected policy checker to fail for run report when expected decision is GO" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$TMP_ERR"; then
  echo "expected checker output to include final_decision=NO-GO for failing report" >&2
  exit 1
fi

if ! grep -q "observed_final_decision_mismatch" "$TMP_ERR"; then
  echo "expected mismatch reason code for failing policy decision" >&2
  exit 1
fi

if ! grep -q "evidence_bundle_schema_invalid" "$TMP_ERR"; then
  echo "expected evidence_bundle_schema_invalid reason marker for invalid evidence bundle schema" >&2
  exit 1
fi

echo "local fork rust test matrix policy checker tests passed."

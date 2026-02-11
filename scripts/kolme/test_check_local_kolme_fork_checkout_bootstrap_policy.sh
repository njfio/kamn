#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork checkout bootstrap policy checker to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-fork-checkout-bootstrap-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "elapsed_seconds": 0,
  "max_seconds": 90,
  "budget_status": "not_run",
  "checkout_path": "/tmp/kolme_fork",
  "fork_remote_url": "https://github.com/njfio/kolme_fork.git",
  "expected_remote_url": "https://github.com/njfio/kolme_fork.git",
  "expected_ref": "refs/heads/main",
  "bootstrap_action": "planned",
  "sync_metadata_report": "/tmp/kolme-sync-summary.json",
  "diagnostics": {
    "git_version": "git version 2.45.0",
    "cargo_version": "cargo 1.86.0",
    "rustc_version": "rustc 1.86.0"
  },
  "checks": [
    {
      "id": "checkout_prepare",
      "command": "git clone --depth 1 --branch main ...",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "sync_metadata",
      "command": "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run ...",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": [
    "/tmp/kolme-sync-summary.json"
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
if report.get("schema_version") != "kamn.kolme.local-fork-checkout-bootstrap-policy-report.v1":
    raise SystemExit("unexpected bootstrap policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid dry-run bootstrap report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no failed checks for valid bootstrap report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-fork-checkout-bootstrap-summary.v1",
  "mode": "run",
  "status": "fail",
  "reason_code": "checkpoint_failed_cargo_version",
  "local_only_enforced": true,
  "elapsed_seconds": 4,
  "max_seconds": 90,
  "budget_status": "within_budget",
  "checkout_path": "/tmp/kolme_fork",
  "fork_remote_url": "https://github.com/njfio/kolme_fork.git",
  "expected_remote_url": "https://github.com/njfio/kolme_fork.git",
  "expected_ref": "refs/heads/main",
  "bootstrap_action": "updated",
  "sync_metadata_report": "/tmp/kolme-sync-summary.json",
  "diagnostics": {
    "git_version": "git version 2.45.0",
    "cargo_version": "",
    "rustc_version": "rustc 1.86.0"
  },
  "checks": [
    {
      "id": "checkout_prepare",
      "command": "git fetch origin main",
      "status": "pass",
      "reason_code": "checkout_updated"
    },
    {
      "id": "diagnostics_cargo_version",
      "command": "__missing_cargo__ --version",
      "status": "fail",
      "reason_code": "cargo_version_failed"
    }
  ],
  "artifact_paths": [
    "/tmp/kolme-sync-summary.json"
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
  echo "expected policy checker to fail for invalid bootstrap report" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$TMP_ERR"; then
  echo "expected checker output to include final_decision=NO-GO for failing report" >&2
  exit 1
fi

if ! grep -q "diagnostics_cargo_version_missing" "$TMP_ERR"; then
  echo "expected diagnostics_cargo_version_missing reason marker for failing report" >&2
  exit 1
fi

echo "local fork checkout bootstrap policy checker tests passed."

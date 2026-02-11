#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_heavy_validation_matrix_policy.py"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local heavy validation matrix policy checker to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-heavy-validation-summary.v1",
  "summary_type": "commands",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "commands": [
    "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json",
    "bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json /tmp/kolme-version-compatibility-report.json",
    "bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json --policy-output-json /tmp/kolme-local-fork-rust-test-matrix-policy.json",
    "bash scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json"
  ],
  "artifact_paths": [
    "/tmp/kolme-local-bootstrap-summary.json",
    "/tmp/kolme-version-compatibility-report.json",
    "/tmp/kolme-local-fork-rust-test-matrix-summary.json",
    "/tmp/kolme-local-fork-rust-test-matrix-policy.json",
    "/tmp/kolme-local-live-api-conformance-summary.json",
    "/tmp/kolme-local-live-api-conformance-policy.json"
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
if report.get("schema_version") != "kamn.kolme.local-heavy-validation-policy-report.v1":
    raise SystemExit("unexpected local heavy validation policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid local heavy validation report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no failed checks for valid local heavy validation report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-heavy-validation-summary.v1",
  "summary_type": "commands",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "commands": [
    "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json"
  ],
  "artifact_paths": [
    "/tmp/kolme-local-bootstrap-summary.json"
  ]
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
  echo "expected policy checker to fail for missing local heavy commands" >&2
  exit 1
fi

if ! grep -q "command_missing:run_version_compatibility_replay_deep_lane.sh" "$TMP_ERR"; then
  echo "expected missing deep replay command marker for policy failure" >&2
  exit 1
fi

echo "local heavy validation matrix policy checker tests passed."

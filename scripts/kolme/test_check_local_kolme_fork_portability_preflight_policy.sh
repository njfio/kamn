#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_portability_preflight_policy.py"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork portability preflight policy checker to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-fork-portability-preflight-summary.v1",
  "summary_type": "checkpoints",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "elapsed_seconds": 0,
  "max_seconds": 300,
  "budget_status": "pass",
  "checkpoints": [
    {
      "id": "local_opt_in_guard",
      "command": "bash scripts/framework/assert_local_heavy_opt_in.sh",
      "status": "planned"
    },
    {
      "id": "mold_linker_probe",
      "command": "bash -lc 'command -v mold >/dev/null 2>&1 || command -v ld.mold >/dev/null 2>&1'",
      "status": "planned"
    },
    {
      "id": "kolme_compile_probe",
      "command": "bash -lc 'cd /tmp/kolme_fork && RUSTFLAGS='' cargo test -p kolme --locked --no-run'",
      "status": "planned"
    },
    {
      "id": "libudev_probe",
      "command": "bash -lc 'pkg-config --libs --cflags libudev'",
      "status": "planned"
    },
    {
      "id": "integration_compile_probe",
      "command": "bash -lc 'cd /tmp/kolme_fork && RUSTFLAGS='' cargo test -p integration-tests --test six-sigma --locked --no-run'",
      "status": "planned"
    }
  ],
  "artifact_paths": [
    "/tmp/linker.log",
    "/tmp/kolme.log",
    "/tmp/libudev.log",
    "/tmp/integration.log"
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
if report.get("schema_version") != "kamn.kolme.local-fork-portability-preflight-policy-report.v1":
    raise SystemExit("unexpected portability preflight policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid dry-run report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no failed checks for valid dry-run report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-fork-portability-preflight-summary.v1",
  "summary_type": "checkpoints",
  "mode": "run",
  "status": "fail",
  "reason_code": "checkpoint_failed_libudev_probe",
  "local_only_enforced": true,
  "elapsed_seconds": 15,
  "max_seconds": 300,
  "budget_status": "pass",
  "checkpoints": [
    {
      "id": "local_opt_in_guard",
      "command": "bash scripts/framework/assert_local_heavy_opt_in.sh",
      "status": "pass"
    },
    {
      "id": "mold_linker_probe",
      "command": "bash -lc 'command -v mold >/dev/null 2>&1 || command -v ld.mold >/dev/null 2>&1'",
      "status": "pass"
    },
    {
      "id": "kolme_compile_probe",
      "command": "bash -lc 'cd /tmp/kolme_fork && RUSTFLAGS='' cargo test -p kolme --locked --no-run'",
      "status": "pass"
    },
    {
      "id": "libudev_probe",
      "command": "bash -lc 'pkg-config --libs --cflags libudev'",
      "status": "fail"
    },
    {
      "id": "integration_compile_probe",
      "command": "bash -lc 'cd /tmp/kolme_fork && RUSTFLAGS='' cargo test -p integration-tests --test six-sigma --locked --no-run'",
      "status": "skipped"
    }
  ],
  "artifact_paths": [
    "/tmp/linker.log",
    "/tmp/kolme.log",
    "/tmp/libudev.log",
    "/tmp/integration.log"
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

echo "local fork portability preflight policy checker tests passed."

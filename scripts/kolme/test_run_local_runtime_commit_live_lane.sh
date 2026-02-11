#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_runtime_commit_live_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_REPORT="$(mktemp)"
TMP_OUTPUT="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_OUTPUT" "$TMP_ERR"' EXIT

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
  echo "expected local runtime commit live lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/assert_local_heavy_opt_in.sh" "$RUNNER"; then
  echo "expected runtime commit live runner to invoke shared local-heavy opt-in guard helper" >&2
  exit 1
fi

if ! grep -q "run_local_runtime_commit_live_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local runtime commit live lane runner" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run live lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "budget_status")" "not_run" "expected dry-run budget status"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-runtime-commit-live-summary.v1":
    raise SystemExit("unexpected live lane summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true")
checks = report.get("checks")
if not isinstance(checks, list) or not checks:
    raise SystemExit("expected deterministic checks in summary")
if not any(
    check.get("id") == "runtime_commit_live_preflight" and check.get("status") == "planned"
    for check in checks
):
    raise SystemExit("expected planned runtime commit live preflight check")
PY

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --live-command "printf 'status=submitted\n'" \
    --max-seconds 5 \
    --base-url "http://127.0.0.1:1" \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT" >"$TMP_ERR" 2>&1
preflight_failure_code=$?
set -e

if [ "$preflight_failure_code" -eq 0 ]; then
  echo "expected run mode preflight failure to fail closed" >&2
  exit 1
fi

if ! grep -q "reason_code=live_preflight_failed" "$TMP_ERR"; then
  echo "expected preflight failure reason marker" >&2
  exit 1
fi

set +e
bash "$RUNNER" \
  --mode run \
  --output-json "$TMP_REPORT" \
  --live-output-file "$TMP_OUTPUT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected explicit local opt-in failure message" >&2
  exit 1
fi

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --skip-preflight \
      --live-command "printf 'status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:1\nfinality=final\n'" \
      --max-seconds 5 \
      --output-json "$TMP_REPORT" \
      --live-output-file "$TMP_OUTPUT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected run mode to pass"
assert_eq "$(extract_value "$run_output" "lane_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "live_runtime_commit_command_passed" "expected pass reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget status"

python3 - "$TMP_REPORT" "$TMP_OUTPUT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
live_output = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
if report.get("mode") != "run":
    raise SystemExit("expected run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status in summary")
if report.get("reason_code") != "live_runtime_commit_command_passed":
    raise SystemExit("expected pass reason code in summary")
if report.get("max_seconds") != 5:
    raise SystemExit("expected max_seconds=5")
if "status=submitted" not in live_output:
    raise SystemExit("expected live command output marker")
PY

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --skip-preflight \
    --live-command "sleep 2" \
    --max-seconds 1 \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT" >"$TMP_ERR" 2>&1
timeout_code=$?
set -e

if [ "$timeout_code" -eq 0 ]; then
  echo "expected run mode timeout to fail closed" >&2
  exit 1
fi

if ! grep -q "reason_code=live_runtime_commit_command_timeout" "$TMP_ERR"; then
  echo "expected timeout reason marker" >&2
  exit 1
fi

echo "local runtime commit live lane tests passed."

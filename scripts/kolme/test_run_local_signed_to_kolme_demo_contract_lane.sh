#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_signed_to_kolme_demo_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_signed_to_kolme_demo_policy.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_ERR"' EXIT

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
  echo "expected unified local signed-to-Kolme demo runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected unified local signed-to-Kolme demo policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_signed_to_kolme_demo_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference unified local signed-to-Kolme demo runner" >&2
  exit 1
fi

if ! grep -q "check_local_signed_to_kolme_demo_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference unified local signed-to-Kolme demo policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_signed_to_kolme_demo_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference unified local signed-to-Kolme demo runner" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run unified demo lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run lane mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "local_only_enforced")" "true" "expected local-only marker"

checker_dry_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code dry_run_no_commands_executed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_dry_run_output" "status")" "ok" "expected checker GO decision for dry-run report"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-signed-to-kolme-demo-summary.v1":
    raise SystemExit("unexpected unified demo summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok dry-run status")
checks = report.get("checks")
if not isinstance(checks, list):
    raise SystemExit("expected checks list in summary")
for expected_id in (
    "localhost_signed_demo_contract",
    "localhost_signed_integration_contract",
    "local_kamn_runtime_integration_contract",
):
    entries = [entry for entry in checks if isinstance(entry, dict) and entry.get("id") == expected_id]
    if not entries:
      raise SystemExit(f"missing check id: {expected_id}")
    if entries[0].get("status") != "planned":
      raise SystemExit(f"expected planned status for check id: {expected_id}")
PY

set +e
bash "$RUNNER" --mode run --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without local opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for unified demo lane" >&2
  exit 1
fi

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --max-seconds 420 \
      --localhost-signed-demo-max-seconds 60 \
      --localhost-signed-integration-max-seconds 120 \
      --kolme-runtime-integration-max-seconds 300 \
      --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected run mode unified demo lane to pass"
assert_eq "$(extract_value "$run_output" "lane_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "signed_to_kolme_demo_passed" "expected run-mode success reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget marker"

checker_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code signed_to_kolme_demo_passed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_run_output" "status")" "ok" "expected checker GO decision for run report"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("mode") != "run":
    raise SystemExit("expected run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok run status")
checks = report.get("checks")
if not isinstance(checks, list):
    raise SystemExit("expected check list in run summary")
for expected_id in (
    "localhost_signed_demo_contract",
    "localhost_signed_integration_contract",
    "local_kamn_runtime_integration_contract",
):
    entries = [entry for entry in checks if isinstance(entry, dict) and entry.get("id") == expected_id]
    if not entries:
      raise SystemExit(f"missing check id: {expected_id}")
    if entries[0].get("status") != "pass":
      raise SystemExit(f"expected pass status for check id: {expected_id}")
PY

echo "unified local signed-to-Kolme demo contract lane tests passed."

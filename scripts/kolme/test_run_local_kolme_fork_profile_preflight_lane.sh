#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -rf "$TMP_DIR"; rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_ERR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork profile preflight runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork profile preflight policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_profile_preflight_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork profile preflight runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_profile_preflight_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork profile preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_profile_preflight_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork profile preflight runner" >&2
  exit 1
fi

CHECKOUT_PATH="$TMP_DIR/kolme_fork"
mkdir -p "$CHECKOUT_PATH"
git -C "$CHECKOUT_PATH" init -q
git -C "$CHECKOUT_PATH" checkout -q -b main
git -C "$CHECKOUT_PATH" config user.email "ci@example.com"
git -C "$CHECKOUT_PATH" config user.name "CI Runner"
cat >"$CHECKOUT_PATH/README.md" <<'EOF'
local fork profile preflight fixture checkout
EOF
git -C "$CHECKOUT_PATH" add README.md
git -C "$CHECKOUT_PATH" commit -q -m "init preflight fixture"
git -C "$CHECKOUT_PATH" remote add origin "https://github.com/njfio/kolme_fork.git"

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --checkout-path "$CHECKOUT_PATH" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run preflight lane to pass"
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
if report.get("schema_version") != "kamn.kolme.local-fork-profile-preflight-summary.v1":
    raise SystemExit("unexpected preflight summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok dry-run status")
checks = report.get("checks")
if not isinstance(checks, list):
    raise SystemExit("expected checks list in summary")
for expected_id in (
    "profile_contract",
    "probe_command",
):
    entries = [entry for entry in checks if isinstance(entry, dict) and entry.get("id") == expected_id]
    if not entries:
        raise SystemExit(f"missing check id: {expected_id}")
    if entries[0].get("status") != "planned":
        raise SystemExit(f"expected planned status for check id: {expected_id}")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --checkout-path "$CHECKOUT_PATH" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without local opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --checkout-path "$CHECKOUT_PATH" \
    --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_allow_non_default_code=$?
set -e

if [ "$run_without_allow_non_default_code" -eq 0 ]; then
  echo "expected preflight run mode without explicit probe override allowance to fail closed in fixture context" >&2
  exit 1
fi

if ! grep -q "must use default cargo profile probe command" "$TMP_ERR"; then
  echo "expected deterministic default probe command policy failure message" >&2
  exit 1
fi

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --probe-command "bash -lc 'exit 0'" \
      --allow-non-default-probe-command \
      --max-seconds 30 \
      --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected run mode preflight lane to pass"
assert_eq "$(extract_value "$run_output" "lane_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "profile_preflight_passed" "expected run-mode success reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget marker"

checker_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code profile_preflight_passed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_run_output" "status")" "ok" "expected checker GO decision for run report"

echo "local fork profile preflight lane tests passed."

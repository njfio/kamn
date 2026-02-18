#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_checkout_bootstrap_contract_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -rf "$TMP_DIR"; rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_ERR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork checkout bootstrap lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork checkout bootstrap policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local fork checkout bootstrap contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_checkout_bootstrap_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork checkout bootstrap runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_checkout_bootstrap_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork checkout bootstrap policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork checkout bootstrap contract lane" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_checkout_bootstrap_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork checkout bootstrap runner" >&2
  exit 1
fi

SOURCE_REPO="$TMP_DIR/source_fork"
CHECKOUT_PATH="$TMP_DIR/checkout_fork"

mkdir -p "$SOURCE_REPO"
git -C "$SOURCE_REPO" init -q
git -C "$SOURCE_REPO" checkout -q -b main
git -C "$SOURCE_REPO" config user.email "ci@example.com"
git -C "$SOURCE_REPO" config user.name "CI Runner"
cat >"$SOURCE_REPO/README.md" <<'EOF'
local fork bootstrap source fixture
EOF
git -C "$SOURCE_REPO" add README.md
git -C "$SOURCE_REPO" commit -q -m "init source fixture"

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --checkout-path "$CHECKOUT_PATH" \
    --fork-remote-url "$SOURCE_REPO" \
    --expected-remote-url "$SOURCE_REPO" \
    --expected-ref "refs/heads/main" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run bootstrap lane to pass"
assert_eq "$(extract_value "$dry_run_output" "bootstrap_mode")" "dry-run" "expected dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "budget_status")" "not_run" "expected dry-run budget marker"

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-fork-checkout-bootstrap-summary.v1":
    raise SystemExit("unexpected local fork checkout bootstrap summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in bootstrap summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in bootstrap summary")
checks = report.get("checks")
if not isinstance(checks, list) or not checks:
    raise SystemExit("expected deterministic check entries in bootstrap summary")
if not any(isinstance(entry, dict) and entry.get("id") == "checkout_prepare" for entry in checks):
    raise SystemExit("expected checkout_prepare check entry in bootstrap summary")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --checkout-path "$CHECKOUT_PATH" \
  --fork-remote-url "$SOURCE_REPO" \
  --expected-remote-url "$SOURCE_REPO" \
  --expected-ref "refs/heads/main" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without local opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for bootstrap lane" >&2
  exit 1
fi

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --fork-remote-url "$SOURCE_REPO" \
      --expected-remote-url "$SOURCE_REPO" \
      --expected-ref "refs/heads/main" \
      --max-seconds 60 \
      --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected bootstrap run to pass"
assert_eq "$(extract_value "$run_output" "bootstrap_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "fork_checkout_bootstrap_passed" "expected run success reason"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget marker"

checker_run_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code fork_checkout_bootstrap_passed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_run_output" "status")" "ok" "expected checker GO decision for bootstrap run report"

python3 - "$TMP_REPORT" "$CHECKOUT_PATH" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
checkout_path = pathlib.Path(sys.argv[2])
if report.get("mode") != "run":
    raise SystemExit("expected run mode in bootstrap summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status in bootstrap summary")
if report.get("bootstrap_action") not in ("cloned", "updated", "validated"):
    raise SystemExit("expected bootstrap_action to be cloned/updated/validated")
if not checkout_path.exists():
    raise SystemExit("expected checkout path to exist after bootstrap run")
diagnostics = report.get("diagnostics")
if not isinstance(diagnostics, dict):
    raise SystemExit("expected diagnostics object in bootstrap summary")
for required_key in ("git_version", "cargo_version", "rustc_version"):
    value = diagnostics.get(required_key)
    if not isinstance(value, str) or not value.strip():
        raise SystemExit(f"expected non-empty diagnostics.{required_key}")
PY

cat >"$SOURCE_REPO/CHANGELOG.md" <<'EOF'
update from source fixture
EOF
git -C "$SOURCE_REPO" add CHANGELOG.md
git -C "$SOURCE_REPO" commit -q -m "update source fixture"

run_update_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --fork-remote-url "$SOURCE_REPO" \
      --expected-remote-url "$SOURCE_REPO" \
      --expected-ref "refs/heads/main" \
      --max-seconds 60 \
      --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_update_output" "status")" "ok" "expected bootstrap update run to pass"
assert_eq "$(extract_value "$run_update_output" "reason_code")" "fork_checkout_bootstrap_passed" "expected update success reason"

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --checkout-path "$CHECKOUT_PATH" \
    --fork-remote-url "$SOURCE_REPO" \
    --expected-remote-url "$TMP_DIR/not-source" \
    --expected-ref "refs/heads/main" \
    --max-seconds 60 \
    --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
remote_mismatch_code=$?
set -e

if [ "$remote_mismatch_code" -eq 0 ]; then
  echo "expected bootstrap run to fail on expected remote mismatch" >&2
  exit 1
fi

if ! grep -q "reason_code=checkpoint_failed_sync_metadata" "$TMP_ERR"; then
  echo "expected checkpoint_failed_sync_metadata marker for expected remote mismatch" >&2
  exit 1
fi

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --checkout-path "$CHECKOUT_PATH" \
    --fork-remote-url "$SOURCE_REPO" \
    --expected-remote-url "$SOURCE_REPO" \
    --expected-ref "refs/heads/main" \
    --cargo-version-command "__missing_cargo__ --version" \
    --allow-non-default-diagnostic-commands \
    --max-seconds 60 \
    --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
cargo_diag_fail_code=$?
set -e

if [ "$cargo_diag_fail_code" -eq 0 ]; then
  echo "expected bootstrap run to fail when cargo diagnostics command fails" >&2
  exit 1
fi

if ! grep -q "reason_code=checkpoint_failed_cargo_version" "$TMP_ERR"; then
  echo "expected checkpoint_failed_cargo_version marker for cargo diagnostics failure" >&2
  exit 1
fi

echo "local fork checkout bootstrap lane tests passed."

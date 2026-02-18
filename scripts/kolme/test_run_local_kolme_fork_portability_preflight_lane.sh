#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_portability_preflight_lane.sh"
SUMMARY_HELPER="$ROOT_DIR/scripts/framework/generate_local_lane_summary.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_REPORT="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_ERR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork portability preflight lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$SUMMARY_HELPER" ]; then
  echo "expected shared local-lane summary helper to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_portability_preflight_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork portability preflight lane" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --checkout-path /tmp/kolme_fork \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run portability preflight lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run lane mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "local_only_enforced")" "true" "expected local-only marker"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-fork-portability-preflight-summary.v1":
    raise SystemExit("unexpected portability preflight summary schema")
if report.get("summary_type") != "checkpoints":
    raise SystemExit("expected summary_type=checkpoints")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok dry-run status")
if report.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code marker")
checkpoint_ids = [
    entry.get("id")
    for entry in report.get("checkpoints", [])
    if isinstance(entry, dict)
]
for expected_id in (
    "local_opt_in_guard",
    "mold_linker_probe",
    "kolme_compile_probe",
    "libudev_probe",
    "integration_compile_probe",
):
    if expected_id not in checkpoint_ids:
        raise SystemExit(f"missing checkpoint id: {expected_id}")
PY

set +e
bash "$RUNNER" --mode run --checkout-path /tmp/kolme_fork --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without local opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for portability preflight lane" >&2
  exit 1
fi

echo "local fork portability preflight lane tests passed."

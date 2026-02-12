#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TREND_CHECKER="$ROOT_DIR/scripts/ci/check_kolme_wrapper_budget_trend.sh"
PYTHON_CHECKER="$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py"
THRESHOLD_FILE="$ROOT_DIR/.ci/kolme-wrapper-budget-trend-thresholds.json"
BASELINE_FIXTURE="$ROOT_DIR/fixtures/kolme_compatibility/wrapper_inventory_baseline.json"
MATRIX_FIXTURE="$ROOT_DIR/fixtures/kolme_compatibility/lane_migration_matrix.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$TREND_CHECKER" ]; then
  echo "expected trend checker wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$PYTHON_CHECKER" ]; then
  echo "expected python baseline checker script to be executable" >&2
  exit 1
fi

if [ ! -f "$THRESHOLD_FILE" ]; then
  echo "expected trend threshold file to exist" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FIXTURE" ]; then
  echo "expected baseline fixture to exist" >&2
  exit 1
fi

if [ ! -f "$MATRIX_FIXTURE" ]; then
  echo "expected matrix fixture to exist" >&2
  exit 1
fi

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$BASELINE_FIXTURE" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"

grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^mode=trend$' "$TMP_DIR/pass.out"
grep -q '^wrapper_count_delta=0$' "$TMP_DIR/pass.out"
grep -q '^total_shell_loc_delta=0$' "$TMP_DIR/pass.out"
grep -q '^violation_count=0$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"

MUTATED_BASELINE="$TMP_DIR/mutated-baseline.json"
cp "$BASELINE_FIXTURE" "$MUTATED_BASELINE"

python3 - "$MUTATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload["total_shell_loc"] = 0
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$MUTATED_BASELINE" >"$TMP_DIR/fail.out" 2>&1; then
  echo "expected trend checker to fail when total shell LOC delta exceeds configured threshold" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail.out"
grep -q '^mode=trend$' "$TMP_DIR/fail.out"
grep -q '^reason_codes=total_shell_loc_delta_threshold_exceeded$' "$TMP_DIR/fail.out"
grep -q 'total_shell_loc_delta exceeded trend threshold' "$TMP_DIR/fail.out"

RELAXED_THRESHOLD="$TMP_DIR/relaxed-threshold.json"
cat >"$RELAXED_THRESHOLD" <<'JSON'
{
  "schema_version": "kamn.kolme.wrapper-budget-trend-thresholds.v1",
  "max_wrapper_count_increase": 1,
  "max_total_shell_loc_increase": 10,
  "enforce_lane_shell_loc_nonincreasing": false
}
JSON

python3 "$PYTHON_CHECKER" check \
  --trend-mode \
  --threshold-file "$RELAXED_THRESHOLD" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$MUTATED_BASELINE" >"$TMP_DIR/relaxed.out"

grep -q '^status=pass$' "$TMP_DIR/relaxed.out"
grep -q '^mode=trend$' "$TMP_DIR/relaxed.out"
grep -q '^reason_codes=none$' "$TMP_DIR/relaxed.out"

echo "Kolme wrapper budget trend checker tests passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_non_kolme_wave_trend_test_loc_soft_budget.sh"
PYTHON_CHECKER="$ROOT_DIR/scripts/ci/check_non_kolme_wave_trend_test_loc_soft_budget.py"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_baseline.json"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_thresholds.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected non-Kolme wave trend-test LOC soft-budget checker wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$PYTHON_CHECKER" ]; then
  echo "expected non-Kolme wave trend-test LOC soft-budget python checker to be executable" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FILE" ]; then
  echo "expected non-Kolme wave trend-test LOC baseline fixture to exist" >&2
  exit 1
fi

if [ ! -f "$THRESHOLD_FILE" ]; then
  echo "expected non-Kolme wave trend-test LOC threshold fixture to exist" >&2
  exit 1
fi

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$CHECKER" \
  --baseline-file "$BASELINE_FILE" \
  --threshold-file "$THRESHOLD_FILE" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"

grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^script_count_delta=0$' "$TMP_DIR/pass.out"
grep -q '^total_shell_loc_delta=0$' "$TMP_DIR/pass.out"
grep -q '^violation_count=0$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"

MUTATED_TOTAL_BASELINE="$TMP_DIR/mutated-total-baseline.json"
cp "$BASELINE_FILE" "$MUTATED_TOTAL_BASELINE"
python3 - "$MUTATED_TOTAL_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload["total_shell_loc"] = 1
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECKER" \
  --baseline-file "$MUTATED_TOTAL_BASELINE" \
  --threshold-file "$THRESHOLD_FILE" >"$TMP_DIR/fail-total.out" 2>&1; then
  echo "expected checker to fail when total shell LOC delta exceeds threshold" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-total.out"
grep -q 'total_shell_loc_delta_threshold_exceeded' "$TMP_DIR/fail-total.out"

MUTATED_STALE_BASELINE="$TMP_DIR/mutated-stale-baseline.json"
cp "$BASELINE_FILE" "$MUTATED_STALE_BASELINE"
python3 - "$MUTATED_STALE_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload["script_files"][0] = "scripts/ci/test_check_non_kolme_wave404_wrapper_family_budget_trend.sh"
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECKER" \
  --baseline-file "$MUTATED_STALE_BASELINE" \
  --threshold-file "$THRESHOLD_FILE" >"$TMP_DIR/fail-stale.out" 2>&1; then
  echo "expected checker to fail on stale baseline script inventory" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-stale.out"
grep -q 'missing_baseline_scripts' "$TMP_DIR/fail-stale.out"

RELAXED_THRESHOLD="$TMP_DIR/relaxed-threshold.json"
cat >"$RELAXED_THRESHOLD" <<'JSON'
{
  "schema_version": "kamn.ci.non-kolme-wave-trend-test-loc-thresholds.v1",
  "max_script_count_increase": 1,
  "max_total_shell_loc_increase": 200
}
JSON

MUTATED_UNDOCUMENTED_GROWTH_BASELINE="$TMP_DIR/mutated-undocumented-growth-baseline.json"
cp "$BASELINE_FILE" "$MUTATED_UNDOCUMENTED_GROWTH_BASELINE"
python3 - "$MUTATED_UNDOCUMENTED_GROWTH_BASELINE" "$ROOT_DIR" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
root_dir = Path(sys.argv[2])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))

removed_script = payload["script_files"].pop()
removed_path = root_dir / removed_script
with removed_path.open("r", encoding="utf-8") as handle:
    removed_loc = sum(1 for _ in handle)

payload["script_count"] = len(payload["script_files"])
payload["total_shell_loc"] = int(payload["total_shell_loc"]) - removed_loc
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECKER" \
  --baseline-file "$MUTATED_UNDOCUMENTED_GROWTH_BASELINE" \
  --threshold-file "$RELAXED_THRESHOLD" >"$TMP_DIR/fail-undocumented-growth.out" 2>&1; then
  echo "expected checker to fail on undocumented current-script growth even when threshold deltas are relaxed" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-undocumented-growth.out"
grep -q 'unexpected_current_scripts' "$TMP_DIR/fail-undocumented-growth.out"

echo "non-Kolme wave trend-test LOC soft-budget checker tests passed."

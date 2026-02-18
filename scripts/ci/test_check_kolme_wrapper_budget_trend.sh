#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
TREND_CHECKER="$ROOT_DIR/scripts/ci/check_kolme_wrapper_budget_trend.sh"
WAVE10_TREND_CHECKER="$ROOT_DIR/scripts/ci/check_kolme_wave10_wrapper_family_budget_trend.sh"
PYTHON_CHECKER="$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py"
THRESHOLD_FILE="$ROOT_DIR/.ci/kolme-wrapper-budget-trend-thresholds.json"
BASELINE_FIXTURE="$ROOT_DIR/fixtures/kolme_compatibility/wrapper_inventory_baseline.json"
MATRIX_FIXTURE="$ROOT_DIR/fixtures/kolme_compatibility/lane_migration_matrix.json"
WAVE10_THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/kolme_wave10_wrapper_family_trend_thresholds.json"
WAVE10_BASELINE_FIXTURE="$ROOT_DIR/fixtures/ci/kolme_wave10_wrapper_family_baseline.json"
WAVE10_MATRIX_FIXTURE="$ROOT_DIR/fixtures/ci/kolme_wave10_wrapper_family_matrix.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$TREND_CHECKER" "expected trend checker wrapper to be executable"

test_harness_require_executable "$PYTHON_CHECKER" "expected python baseline checker script to be executable"

test_harness_require_executable "$WAVE10_TREND_CHECKER" "expected wave-10 trend checker wrapper to be executable"

test_harness_require_file "$THRESHOLD_FILE" "expected trend threshold file to exist"

test_harness_require_file "$BASELINE_FIXTURE" "expected baseline fixture to exist"

test_harness_require_file "$MATRIX_FIXTURE" "expected matrix fixture to exist"

test_harness_require_file "$WAVE10_THRESHOLD_FILE" "expected wave-10 trend threshold fixture to exist"

test_harness_require_file "$WAVE10_BASELINE_FIXTURE" "expected wave-10 baseline fixture to exist"

test_harness_require_file "$WAVE10_MATRIX_FIXTURE" "expected wave-10 matrix fixture to exist"

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

MUTATED_WRAPPER_COUNT_BASELINE="$TMP_DIR/mutated-wrapper-count-baseline.json"
cp "$BASELINE_FIXTURE" "$MUTATED_WRAPPER_COUNT_BASELINE"
python3 - "$MUTATED_WRAPPER_COUNT_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload["wrapper_count"] = 0
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$MUTATED_WRAPPER_COUNT_BASELINE" >"$TMP_DIR/fail-wrapper-count.out" 2>&1; then
  echo "expected trend checker to fail when wrapper_count delta exceeds configured threshold" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-wrapper-count.out"
grep -q '^mode=trend$' "$TMP_DIR/fail-wrapper-count.out"
grep -q '^reason_codes=wrapper_count_delta_threshold_exceeded$' "$TMP_DIR/fail-wrapper-count.out"
grep -q 'wrapper_count_delta exceeded trend threshold' "$TMP_DIR/fail-wrapper-count.out"

MUTATED_MISSING_BASELINE_METADATA="$TMP_DIR/mutated-missing-baseline-metadata.json"
cp "$BASELINE_FIXTURE" "$MUTATED_MISSING_BASELINE_METADATA"
python3 - "$MUTATED_MISSING_BASELINE_METADATA" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload.pop("wrapper_count", None)
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$TREND_CHECKER" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$MUTATED_MISSING_BASELINE_METADATA" >"$TMP_DIR/fail-missing-baseline-metadata.out" 2>&1; then
  echo "expected trend checker to fail with deterministic reason code when baseline metadata is missing" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-missing-baseline-metadata.out"
grep -q '^mode=trend$' "$TMP_DIR/fail-missing-baseline-metadata.out"
grep -q '^reason_codes=baseline_wrapper_count_invalid$' "$TMP_DIR/fail-missing-baseline-metadata.out"
grep -q 'baseline wrapper_count must be an integer' "$TMP_DIR/fail-missing-baseline-metadata.out"

MUTATED_MISSING_THRESHOLD_METADATA="$TMP_DIR/mutated-missing-threshold-metadata.json"
cp "$THRESHOLD_FILE" "$MUTATED_MISSING_THRESHOLD_METADATA"
python3 - "$MUTATED_MISSING_THRESHOLD_METADATA" <<'PY'
import json
import sys
from pathlib import Path

threshold_path = Path(sys.argv[1])
payload = json.loads(threshold_path.read_text(encoding="utf-8"))
payload.pop("max_total_shell_loc_increase", None)
threshold_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if python3 "$PYTHON_CHECKER" check \
  --trend-mode \
  --threshold-file "$MUTATED_MISSING_THRESHOLD_METADATA" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$BASELINE_FIXTURE" >"$TMP_DIR/fail-missing-threshold-metadata.out" 2>&1; then
  echo "expected trend checker to fail with deterministic reason code when threshold metadata is missing" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-missing-threshold-metadata.out"
grep -q '^mode=trend$' "$TMP_DIR/fail-missing-threshold-metadata.out"
grep -q '^reason_codes=trend_threshold_total_shell_loc_invalid$' "$TMP_DIR/fail-missing-threshold-metadata.out"
grep -q 'max_total_shell_loc_increase' "$TMP_DIR/fail-missing-threshold-metadata.out"

RELAXED_THRESHOLD="$TMP_DIR/relaxed-threshold.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$RELAXED_THRESHOLD" <<'JSON'
{
  "schema_version": "kamn.kolme.wrapper-budget-trend-thresholds.v1",
  "max_wrapper_count_increase": 1,
  "max_total_shell_loc_increase": 11,
  "enforce_lane_shell_loc_nonincreasing": false,
  "min_wrapper_count_reduction": 0,
  "min_total_shell_loc_reduction": 0
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

REDUCTION_TARGET_THRESHOLD="$TMP_DIR/reduction-target-threshold.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$REDUCTION_TARGET_THRESHOLD" <<'JSON'
{
  "schema_version": "kamn.kolme.wrapper-budget-trend-thresholds.v1",
  "max_wrapper_count_increase": 0,
  "max_total_shell_loc_increase": 0,
  "enforce_lane_shell_loc_nonincreasing": true,
  "min_wrapper_count_reduction": 1,
  "min_total_shell_loc_reduction": 1
}
JSON

if python3 "$PYTHON_CHECKER" check \
  --trend-mode \
  --threshold-file "$REDUCTION_TARGET_THRESHOLD" \
  --matrix-file "$MATRIX_FIXTURE" \
  --baseline-file "$BASELINE_FIXTURE" >"$TMP_DIR/reduction-target-fail.out" 2>&1; then
  echo "expected trend checker to fail when configured reduction targets are unmet" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/reduction-target-fail.out"
grep -q '^mode=trend$' "$TMP_DIR/reduction-target-fail.out"
grep -q 'wrapper_count_reduction_target_unmet' "$TMP_DIR/reduction-target-fail.out"
grep -q 'total_shell_loc_reduction_target_unmet' "$TMP_DIR/reduction-target-fail.out"

WAVE10_PASS_REPORT="$TMP_DIR/wave10-pass-report.json"
bash "$WAVE10_TREND_CHECKER" \
  --matrix-file "$WAVE10_MATRIX_FIXTURE" \
  --baseline-file "$WAVE10_BASELINE_FIXTURE" \
  --output-json "$WAVE10_PASS_REPORT" >"$TMP_DIR/wave10-pass.out"

grep -q '^status=pass$' "$TMP_DIR/wave10-pass.out"
grep -q '^mode=trend$' "$TMP_DIR/wave10-pass.out"
grep -q '^wrapper_count_delta=0$' "$TMP_DIR/wave10-pass.out"
grep -q '^total_shell_loc_delta=0$' "$TMP_DIR/wave10-pass.out"
grep -q '^violation_count=0$' "$TMP_DIR/wave10-pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/wave10-pass.out"

WAVE10_MUTATED_BASELINE="$TMP_DIR/wave10-mutated-baseline.json"
cp "$WAVE10_BASELINE_FIXTURE" "$WAVE10_MUTATED_BASELINE"
python3 - "$WAVE10_MUTATED_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload["total_shell_loc"] = 0
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$WAVE10_TREND_CHECKER" \
  --matrix-file "$WAVE10_MATRIX_FIXTURE" \
  --baseline-file "$WAVE10_MUTATED_BASELINE" >"$TMP_DIR/wave10-fail-total.out" 2>&1; then
  echo "expected wave-10 trend checker to fail when total shell LOC delta exceeds threshold" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/wave10-fail-total.out"
grep -q '^mode=trend$' "$TMP_DIR/wave10-fail-total.out"
grep -q 'total_shell_loc_delta_threshold_exceeded' "$TMP_DIR/wave10-fail-total.out"

WAVE10_MUTATED_STALE_BASELINE="$TMP_DIR/wave10-mutated-stale-baseline.json"
cp "$WAVE10_BASELINE_FIXTURE" "$WAVE10_MUTATED_STALE_BASELINE"
python3 - "$WAVE10_MUTATED_STALE_BASELINE" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
payload = json.loads(baseline_path.read_text(encoding="utf-8"))
payload["lanes"] = payload["lanes"][:-1]
payload["wrapper_count"] = len(payload["lanes"])
payload["symlink_wrapper_count"] = len(payload["lanes"])
payload["total_shell_loc"] = sum(int(lane["shell_loc"]) for lane in payload["lanes"])
baseline_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$WAVE10_TREND_CHECKER" \
  --matrix-file "$WAVE10_MATRIX_FIXTURE" \
  --baseline-file "$WAVE10_MUTATED_STALE_BASELINE" >"$TMP_DIR/wave10-fail-stale.out" 2>&1; then
  echo "expected wave-10 trend checker to fail on stale baseline lane inventory" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/wave10-fail-stale.out"
grep -q '^mode=trend$' "$TMP_DIR/wave10-fail-stale.out"
grep -q 'unexpected_new_lanes_in_current_inventory' "$TMP_DIR/wave10-fail-stale.out"

echo "Kolme wrapper budget trend checker tests passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_non_kolme_wave_trend_test_loc_soft_budget.sh"
PYTHON_CHECKER="$ROOT_DIR/scripts/ci/check_non_kolme_wave_trend_test_loc_soft_budget.py"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_baseline.json"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_thresholds.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected non-Kolme wave trend-test LOC soft-budget checker wrapper to be executable"

test_harness_require_executable "$PYTHON_CHECKER" "expected non-Kolme wave trend-test LOC soft-budget python checker to be executable"

test_harness_require_file "$BASELINE_FILE" "expected non-Kolme wave trend-test LOC baseline fixture to exist"

test_harness_require_file "$THRESHOLD_FILE" "expected non-Kolme wave trend-test LOC threshold fixture to exist"

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$CHECKER" \
  --baseline-file "$BASELINE_FILE" \
  --threshold-file "$THRESHOLD_FILE" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"

grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^soft_overrun_status=within$' "$TMP_DIR/pass.out"
grep -q '^waiver_status=none$' "$TMP_DIR/pass.out"
grep -q '^review_required=false$' "$TMP_DIR/pass.out"
grep -q '^waived_reason_codes=none$' "$TMP_DIR/pass.out"
grep -q '^script_count_delta=0$' "$TMP_DIR/pass.out"
grep -q '^total_shell_loc_delta=0$' "$TMP_DIR/pass.out"
grep -q '^violation_count=0$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"

CORRUPT_THRESHOLD_FILE="$TMP_DIR/corrupt-threshold.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$CORRUPT_THRESHOLD_FILE" <<'JSON'
{ "schema_version":
JSON

if bash "$CHECKER" \
  --baseline-file "$BASELINE_FILE" \
  --threshold-file "$CORRUPT_THRESHOLD_FILE" >"$TMP_DIR/fail-threshold-corrupt.out" 2>&1; then
  echo "expected checker to fail when threshold JSON is corrupt" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-threshold-corrupt.out"
grep -q '^reason_codes=threshold_json_invalid$' "$TMP_DIR/fail-threshold-corrupt.out"

MALFORMED_THRESHOLD_FILE="$TMP_DIR/malformed-threshold.json"
cp "$THRESHOLD_FILE" "$MALFORMED_THRESHOLD_FILE"
python3 - "$MALFORMED_THRESHOLD_FILE" <<'PY'
import json
import sys
from pathlib import Path

threshold_path = Path(sys.argv[1])
payload = json.loads(threshold_path.read_text(encoding="utf-8"))
payload.pop("threshold_refreshed_on", None)
payload["threshold_max_age_days"] = -1
threshold_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECKER" \
  --baseline-file "$BASELINE_FILE" \
  --threshold-file "$MALFORMED_THRESHOLD_FILE" >"$TMP_DIR/fail-threshold-malformed.out" 2>&1; then
  echo "expected checker to fail when threshold metadata is malformed" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-threshold-malformed.out"
grep -q '^reason_codes=threshold_refresh_metadata_invalid$' "$TMP_DIR/fail-threshold-malformed.out"

INVALID_DATE_THRESHOLD_FILE="$TMP_DIR/invalid-date-threshold.json"
cp "$THRESHOLD_FILE" "$INVALID_DATE_THRESHOLD_FILE"
python3 - "$INVALID_DATE_THRESHOLD_FILE" <<'PY'
import json
import sys
from pathlib import Path

threshold_path = Path(sys.argv[1])
payload = json.loads(threshold_path.read_text(encoding="utf-8"))
payload["threshold_refreshed_on"] = "not-a-date"
payload["threshold_max_age_days"] = 30
threshold_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECKER" \
  --baseline-file "$BASELINE_FILE" \
  --threshold-file "$INVALID_DATE_THRESHOLD_FILE" >"$TMP_DIR/fail-threshold-invalid-date.out" 2>&1; then
  echo "expected checker to fail when threshold_refreshed_on is invalid" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-threshold-invalid-date.out"
grep -q '^reason_codes=threshold_refreshed_on_invalid$' "$TMP_DIR/fail-threshold-invalid-date.out"

STALE_THRESHOLD_FILE="$TMP_DIR/stale-threshold.json"
cp "$THRESHOLD_FILE" "$STALE_THRESHOLD_FILE"
python3 - "$STALE_THRESHOLD_FILE" <<'PY'
import json
import sys
from pathlib import Path

threshold_path = Path(sys.argv[1])
payload = json.loads(threshold_path.read_text(encoding="utf-8"))
payload["threshold_refreshed_on"] = "2000-01-01"
payload["threshold_max_age_days"] = 30
threshold_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECKER" \
  --baseline-file "$BASELINE_FILE" \
  --threshold-file "$STALE_THRESHOLD_FILE" >"$TMP_DIR/fail-threshold-stale.out" 2>&1; then
  echo "expected checker to fail when threshold metadata is stale" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail-threshold-stale.out"
grep -q '^reason_codes=threshold_file_stale$' "$TMP_DIR/fail-threshold-stale.out"

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
grep -q '^soft_overrun_status=exceeded$' "$TMP_DIR/fail-total.out"
grep -q '^waiver_status=none$' "$TMP_DIR/fail-total.out"
grep -q '^review_required=true$' "$TMP_DIR/fail-total.out"
grep -q 'total_shell_loc_delta_threshold_exceeded' "$TMP_DIR/fail-total.out"
grep -q 'delta_threshold_violation_unwaived' "$TMP_DIR/fail-total.out"

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
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$RELAXED_THRESHOLD" <<'JSON'
{
  "schema_version": "kamn.ci.non-kolme-wave-trend-test-loc-thresholds.v1",
  "threshold_refreshed_on": "2026-02-15",
  "threshold_max_age_days": 365,
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
if removed_path.is_symlink():
    removed_loc = 1
else:
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

VALID_WAIVER_FILE="$TMP_DIR/valid-waiver.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$VALID_WAIVER_FILE" <<'JSON'
{
  "schema_version": "kamn.ci.non-kolme-wave-trend-test-loc-soft-budget-waiver.v1",
  "scope": "non_kolme_wave_trend_test_loc_soft_budget",
  "expires_on": "2099-12-31",
  "approved_by": "ops",
  "justification": "temporary shell surface drift while wrapper migration wave is in flight",
  "allowed_reason_codes": [
    "total_shell_loc_delta_threshold_exceeded",
    "script_count_delta_threshold_exceeded"
  ]
}
JSON

WAIVER_PASS_REPORT="$TMP_DIR/waiver-pass-report.json"
bash "$CHECKER" \
  --baseline-file "$MUTATED_TOTAL_BASELINE" \
  --threshold-file "$THRESHOLD_FILE" \
  --waiver-file "$VALID_WAIVER_FILE" \
  --output-json "$WAIVER_PASS_REPORT" >"$TMP_DIR/waiver-pass.out"

grep -q '^status=pass$' "$TMP_DIR/waiver-pass.out"
grep -q '^soft_overrun_status=exceeded$' "$TMP_DIR/waiver-pass.out"
grep -q '^waiver_status=applied$' "$TMP_DIR/waiver-pass.out"
grep -q '^review_required=true$' "$TMP_DIR/waiver-pass.out"
grep -q 'waived_reason_codes=total_shell_loc_delta_threshold_exceeded' "$TMP_DIR/waiver-pass.out"
grep -q 'delta_threshold_waiver_applied' "$TMP_DIR/waiver-pass.out"

EXPIRED_WAIVER_FILE="$TMP_DIR/expired-waiver.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$EXPIRED_WAIVER_FILE" <<'JSON'
{
  "schema_version": "kamn.ci.non-kolme-wave-trend-test-loc-soft-budget-waiver.v1",
  "scope": "non_kolme_wave_trend_test_loc_soft_budget",
  "expires_on": "2000-01-01",
  "approved_by": "ops",
  "justification": "expired waiver regression check",
  "allowed_reason_codes": [
    "total_shell_loc_delta_threshold_exceeded"
  ]
}
JSON

if bash "$CHECKER" \
  --baseline-file "$MUTATED_TOTAL_BASELINE" \
  --threshold-file "$THRESHOLD_FILE" \
  --waiver-file "$EXPIRED_WAIVER_FILE" >"$TMP_DIR/waiver-expired.out" 2>&1; then
  echo "expected checker to fail when waiver file is expired" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/waiver-expired.out"
grep -q '^reason_codes=waiver_expired$' "$TMP_DIR/waiver-expired.out"

SCOPE_MISMATCH_WAIVER_FILE="$TMP_DIR/scope-mismatch-waiver.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$SCOPE_MISMATCH_WAIVER_FILE" <<'JSON'
{
  "schema_version": "kamn.ci.non-kolme-wave-trend-test-loc-soft-budget-waiver.v1",
  "scope": "non_kolme_wave_other_scope",
  "expires_on": "2099-12-31",
  "approved_by": "ops",
  "justification": "scope mismatch regression check",
  "allowed_reason_codes": [
    "total_shell_loc_delta_threshold_exceeded"
  ]
}
JSON

if bash "$CHECKER" \
  --baseline-file "$MUTATED_TOTAL_BASELINE" \
  --threshold-file "$THRESHOLD_FILE" \
  --waiver-file "$SCOPE_MISMATCH_WAIVER_FILE" >"$TMP_DIR/waiver-scope-mismatch.out" 2>&1; then
  echo "expected checker to fail when waiver scope does not match checker scope" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/waiver-scope-mismatch.out"
grep -q '^reason_codes=waiver_scope_mismatch$' "$TMP_DIR/waiver-scope-mismatch.out"

echo "non-Kolme wave trend-test LOC soft-budget checker tests passed."

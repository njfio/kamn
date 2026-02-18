#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_test_harness_loc_soft_budget_contract_lane.sh \
  --output-json <path> \
  [--max-runtime-seconds <int>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_test_harness_loc_soft_budget.sh"
TREND_THRESHOLD_FILE="$ROOT_DIR/.ci/test-harness-loc-trend-thresholds.env"

OUTPUT_JSON=""
MAX_RUNTIME_SECONDS="${KAMN_TEST_HARNESS_LOC_SOFT_BUDGET_CONTRACT_MAX_SECONDS:-120}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      OUTPUT_JSON="${2:-}"
      shift 2
      ;;
    --max-runtime-seconds)
      MAX_RUNTIME_SECONDS="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$OUTPUT_JSON" ]; then
  usage >&2
  exit 2
fi

case "$MAX_RUNTIME_SECONDS" in
  ''|*[!0-9]*)
    echo "--max-runtime-seconds must be a non-negative integer" >&2
    exit 2
    ;;
esac

test_harness_require_executable "$CHECK_SCRIPT" "expected test-harness soft-budget checker to be executable: $CHECK_SCRIPT"

test_harness_require_file "$TREND_THRESHOLD_FILE" "expected trend threshold file to exist: $TREND_THRESHOLD_FILE"

mkdir -p "$(dirname "$OUTPUT_JSON")"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

start_epoch="$(date +%s)"

BUDGET_FILE="$tmp_dir/soft-budget.env"
BASELINE_FILE_TREND="$tmp_dir/baseline-trend.env"
BASELINE_FILE_SOFT_EXCEEDED="$tmp_dir/baseline-soft-exceeded.env"
WITHIN_REPORT="$tmp_dir/harness-within.json"
EXCEEDED_REPORT="$tmp_dir/harness-exceeded.json"
WARN_REPORT="$tmp_dir/harness-warn.json"
FAIL_REPORT="$tmp_dir/harness-fail.json"
WITHIN_POLICY="$tmp_dir/harness-within-policy.json"
EXCEEDED_POLICY="$tmp_dir/harness-exceeded-policy.json"
WARN_POLICY="$tmp_dir/harness-warn-policy.json"
FAIL_POLICY="$tmp_dir/harness-fail-policy.json"

cat >"$BUDGET_FILE" <<'EOF_BUDGET'
TEST_HARNESS_SCRIPT_COUNT_SOFT_MAX=20
TEST_HARNESS_SHELL_LINE_TOTAL_SOFT_MAX=200
EOF_BUDGET

cat >"$BASELINE_FILE_TREND" <<'EOF_BASELINE'
TEST_HARNESS_SCRIPT_COUNT_BASELINE=8
TEST_HARNESS_SHELL_LINE_TOTAL_BASELINE=80
EOF_BASELINE

cat >"$BASELINE_FILE_SOFT_EXCEEDED" <<'EOF_BASELINE'
TEST_HARNESS_SCRIPT_COUNT_BASELINE=20
TEST_HARNESS_SHELL_LINE_TOTAL_BASELINE=200
EOF_BASELINE

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WITHIN_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 10,
  "harness_shell_line_total": 100
}
EOF_REPORT

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$EXCEEDED_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 21,
  "harness_shell_line_total": 205
}
EOF_REPORT

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WARN_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 13,
  "harness_shell_line_total": 130
}
EOF_REPORT

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$FAIL_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 19,
  "harness_shell_line_total": 195
}
EOF_REPORT

bash "$CHECK_SCRIPT" \
  --report-file "$WITHIN_REPORT" \
  --budget-file "$BUDGET_FILE" \
  --baseline-file "$BASELINE_FILE_TREND" \
  --trend-threshold-file "$TREND_THRESHOLD_FILE" \
  --output-json "$WITHIN_POLICY" >/dev/null

bash "$CHECK_SCRIPT" \
  --report-file "$EXCEEDED_REPORT" \
  --budget-file "$BUDGET_FILE" \
  --baseline-file "$BASELINE_FILE_SOFT_EXCEEDED" \
  --trend-threshold-file "$TREND_THRESHOLD_FILE" \
  --output-json "$EXCEEDED_POLICY" >/dev/null

bash "$CHECK_SCRIPT" \
  --report-file "$WARN_REPORT" \
  --budget-file "$BUDGET_FILE" \
  --baseline-file "$BASELINE_FILE_TREND" \
  --trend-threshold-file "$TREND_THRESHOLD_FILE" \
  --output-json "$WARN_POLICY" >/dev/null

bash "$CHECK_SCRIPT" \
  --report-file "$FAIL_REPORT" \
  --budget-file "$BUDGET_FILE" \
  --baseline-file "$BASELINE_FILE_TREND" \
  --trend-threshold-file "$TREND_THRESHOLD_FILE" \
  --output-json "$FAIL_POLICY" >/dev/null

python3 - "$WITHIN_POLICY" "$EXCEEDED_POLICY" "$WARN_POLICY" "$FAIL_POLICY" "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path

within_path = Path(sys.argv[1])
exceeded_path = Path(sys.argv[2])
warn_path = Path(sys.argv[3])
fail_path = Path(sys.argv[4])
output_path = Path(sys.argv[5])

within = json.loads(within_path.read_text(encoding="utf-8"))
exceeded = json.loads(exceeded_path.read_text(encoding="utf-8"))
warn = json.loads(warn_path.read_text(encoding="utf-8"))
fail = json.loads(fail_path.read_text(encoding="utf-8"))


def require_equal(payload: dict, key: str, expected: str, label: str) -> None:
    value = payload.get(key)
    if value != expected:
        raise SystemExit(f"{label}: expected {key}={expected}, got {value}")


def require_reason(payload: dict, reason_code: str, label: str) -> None:
    reason_codes = payload.get("reason_codes")
    if not isinstance(reason_codes, list):
        raise SystemExit(f"{label}: reason_codes must be a list")
    if reason_code not in reason_codes:
        raise SystemExit(f"{label}: missing reason code {reason_code}")


require_equal(within, "status", "ok", "within policy")
require_equal(within, "soft_budget_status", "within", "within policy")
require_equal(within, "trend_status", "within", "within policy")
require_equal(within, "policy_decision", "GO", "within policy")
if within.get("reason_codes") not in ([], None):
    raise SystemExit("within policy: expected no reason_codes")

require_equal(exceeded, "status", "ok", "soft-exceeded policy")
require_equal(exceeded, "soft_budget_status", "exceeded", "soft-exceeded policy")
require_equal(exceeded, "trend_status", "within", "soft-exceeded policy")
require_equal(exceeded, "policy_decision", "WARN", "soft-exceeded policy")
require_reason(
    exceeded,
    "harness_script_count_soft_max_exceeded",
    "soft-exceeded policy",
)
require_reason(
    exceeded,
    "harness_shell_line_total_soft_max_exceeded",
    "soft-exceeded policy",
)

require_equal(warn, "status", "ok", "trend-warn policy")
require_equal(warn, "soft_budget_status", "within", "trend-warn policy")
require_equal(warn, "trend_status", "warn", "trend-warn policy")
require_equal(warn, "policy_decision", "WARN", "trend-warn policy")
require_reason(
    warn,
    "harness_script_count_trend_warn_delta_exceeded",
    "trend-warn policy",
)
require_reason(
    warn,
    "harness_shell_line_total_trend_warn_delta_exceeded",
    "trend-warn policy",
)

require_equal(fail, "status", "ok", "trend-fail policy")
require_equal(fail, "soft_budget_status", "within", "trend-fail policy")
require_equal(fail, "trend_status", "fail", "trend-fail policy")
require_equal(fail, "policy_decision", "NO-GO", "trend-fail policy")
require_reason(
    fail,
    "harness_script_count_trend_fail_delta_exceeded",
    "trend-fail policy",
)
require_reason(
    fail,
    "harness_shell_line_total_trend_fail_delta_exceeded",
    "trend-fail policy",
)

summary = {
    "schema_version": "kamn.ci.test-harness-loc-soft-budget-contract-report.v1",
    "status": "pass",
    "within_decision": within["policy_decision"],
    "exceeded_decision": exceeded["policy_decision"],
    "warn_decision": warn["policy_decision"],
    "fail_decision": fail["policy_decision"],
    "ci_smoke_lane_cost_profile": "low",
    "ci_smoke_runtime_budget_status": "within",
    "combined_reason_code_contract": "pass",
    "trend_reason_code_contract": "pass",
    "within_reason_codes": within.get("reason_codes", []),
    "exceeded_reason_codes": exceeded.get("reason_codes", []),
    "warn_reason_codes": warn.get("reason_codes", []),
    "fail_reason_codes": fail.get("reason_codes", []),
}
output_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
status="pass"
reason_key="test_harness_loc_soft_budget_contract_ok"
if [ "$elapsed_seconds" -gt "$MAX_RUNTIME_SECONDS" ]; then
  status="fail"
  reason_key="test_harness_loc_soft_budget_contract_runtime_budget_exceeded"
fi

python3 - "$OUTPUT_JSON" "$status" "$elapsed_seconds" "$MAX_RUNTIME_SECONDS" "$reason_key" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
status = sys.argv[2]
elapsed_seconds = int(sys.argv[3])
max_runtime_seconds = int(sys.argv[4])
reason_key = sys.argv[5]

payload = json.loads(path.read_text(encoding="utf-8"))
payload["status"] = status
payload["runtime_seconds"] = elapsed_seconds
payload["max_runtime_seconds"] = max_runtime_seconds
payload["reason_key"] = reason_key
payload["ci_smoke_lane_cost_profile"] = "low"
payload["ci_smoke_runtime_budget_status"] = "within" if status == "pass" else "exceeded"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

within_decision="$(python3 - "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("within_decision", ""))
PY
)"

exceeded_decision="$(python3 - "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("exceeded_decision", ""))
PY
)"

warn_decision="$(python3 - "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("warn_decision", ""))
PY
)"

fail_decision="$(python3 - "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("fail_decision", ""))
PY
)"

ci_smoke_runtime_budget_status="$(python3 - "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("ci_smoke_runtime_budget_status", ""))
PY
)"

reason_key_value="$(python3 - "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("reason_key", ""))
PY
)"

echo "test_harness_loc_soft_budget_contract_status=$status"
echo "test_harness_loc_soft_budget_contract_within_decision=$within_decision"
echo "test_harness_loc_soft_budget_contract_exceeded_decision=$exceeded_decision"
echo "test_harness_loc_soft_budget_contract_warn_decision=$warn_decision"
echo "test_harness_loc_soft_budget_contract_fail_decision=$fail_decision"
echo "test_harness_loc_soft_budget_contract_ci_smoke_lane_cost_profile=low"
echo "test_harness_loc_soft_budget_contract_ci_smoke_runtime_budget_status=$ci_smoke_runtime_budget_status"
echo "test_harness_loc_soft_budget_contract_reason_key=$reason_key_value"
echo "test_harness_loc_soft_budget_contract_report=$(realpath "$OUTPUT_JSON")"

if [ "$status" != "pass" ]; then
  echo "Test-harness soft-budget contract lane exceeded runtime budget: ${elapsed_seconds}s > ${MAX_RUNTIME_SECONDS}s" >&2
  exit 1
fi

echo "Generic test harness LOC soft-budget contract lane tests passed."

#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_kolme_test_harness_loc_soft_budget_contract_lane.sh \
  --output-json <path> \
  [--max-runtime-seconds <int>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_kolme_test_harness_loc_soft_budget.sh"
TREND_SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_test_harness_loc_trend_report.sh"

OUTPUT_JSON=""
MAX_RUNTIME_SECONDS="${KAMN_KOLME_TEST_HARNESS_LOC_SOFT_BUDGET_CONTRACT_MAX_SECONDS:-120}"

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

if [ ! -x "$CHECK_SCRIPT" ]; then
  echo "expected Kolme soft-budget checker to be executable: $CHECK_SCRIPT" >&2
  exit 1
fi

if [ ! -x "$TREND_SCRIPT" ]; then
  echo "expected Kolme trend generator to be executable: $TREND_SCRIPT" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT_JSON")"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

start_epoch="$(date +%s)"

WITHIN_REPORT="$tmp_dir/kolme-harness-within.json"
WARN_REPORT="$tmp_dir/kolme-harness-warn.json"
FAIL_REPORT="$tmp_dir/kolme-harness-fail.json"

cat >"$WITHIN_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 62,
  "harness_shell_line_total": 8691
}
EOF_REPORT

cat >"$WARN_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 70,
  "harness_shell_line_total": 10350
}
EOF_REPORT

cat >"$FAIL_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 90,
  "harness_shell_line_total": 13000
}
EOF_REPORT

COMMAND_SURFACE_WITHIN_REPORT="$tmp_dir/kolme-command-surface-within.json"
COMMAND_SURFACE_FAIL_REPORT="$tmp_dir/kolme-command-surface-fail.json"

cat >"$COMMAND_SURFACE_WITHIN_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.script-surface-budget-report.v1",
  "status": "pass",
  "metrics": {
    "script_count": 56,
    "shell_line_total": 12958,
    "duplicate_basename": 0,
    "duplicate_content": 0
  },
  "deltas": {
    "script_count": 0,
    "shell_line_total": 0,
    "duplicate_basename": 0,
    "duplicate_content": 0
  }
}
EOF_REPORT

cat >"$COMMAND_SURFACE_FAIL_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.script-surface-budget-report.v1",
  "status": "pass",
  "metrics": {
    "script_count": 66,
    "shell_line_total": 15358,
    "duplicate_basename": 0,
    "duplicate_content": 0
  },
  "deltas": {
    "script_count": 10,
    "shell_line_total": 2401,
    "duplicate_basename": 0,
    "duplicate_content": 0
  }
}
EOF_REPORT

WITHIN_POLICY="$tmp_dir/kolme-soft-budget-within-policy.json"
WARN_POLICY="$tmp_dir/kolme-soft-budget-warn-policy.json"
FAIL_POLICY="$tmp_dir/kolme-soft-budget-fail-policy.json"
TREND_GO_REPORT="$tmp_dir/kolme-trend-go.json"
TREND_FAIL_REPORT="$tmp_dir/kolme-trend-fail.json"

bash "$CHECK_SCRIPT" --report-file "$WITHIN_REPORT" --output-json "$WITHIN_POLICY" >/dev/null
bash "$CHECK_SCRIPT" --report-file "$WARN_REPORT" --output-json "$WARN_POLICY" >/dev/null
bash "$CHECK_SCRIPT" --report-file "$FAIL_REPORT" --output-json "$FAIL_POLICY" >/dev/null

bash "$TREND_SCRIPT" \
  --report-file "$WITHIN_REPORT" \
  --command-surface-report-file "$COMMAND_SURFACE_WITHIN_REPORT" \
  --output-json "$TREND_GO_REPORT" >/dev/null

bash "$TREND_SCRIPT" \
  --report-file "$WITHIN_REPORT" \
  --command-surface-report-file "$COMMAND_SURFACE_FAIL_REPORT" \
  --output-json "$TREND_FAIL_REPORT" >/dev/null

python3 - "$WITHIN_POLICY" "$WARN_POLICY" "$FAIL_POLICY" "$TREND_GO_REPORT" "$TREND_FAIL_REPORT" "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path

within_path = Path(sys.argv[1])
warn_path = Path(sys.argv[2])
fail_path = Path(sys.argv[3])
trend_go_path = Path(sys.argv[4])
trend_fail_path = Path(sys.argv[5])
output_path = Path(sys.argv[6])

within = json.loads(within_path.read_text(encoding="utf-8"))
warn = json.loads(warn_path.read_text(encoding="utf-8"))
fail = json.loads(fail_path.read_text(encoding="utf-8"))
trend_go = json.loads(trend_go_path.read_text(encoding="utf-8"))
trend_fail = json.loads(trend_fail_path.read_text(encoding="utf-8"))


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


require_equal(within, "policy_decision", "GO", "within policy")
require_equal(within, "soft_budget_status", "within", "within policy")
require_equal(within, "trend_status", "within", "within policy")
if within.get("reason_codes") not in ([], None):
    raise SystemExit("within policy: expected no reason_codes")

require_equal(warn, "policy_decision", "WARN", "warn policy")
require_equal(warn, "soft_budget_status", "exceeded", "warn policy")
require_equal(warn, "trend_status", "warn", "warn policy")
require_reason(warn, "harness_shell_line_total_soft_max_exceeded", "warn policy")
require_reason(warn, "harness_shell_line_total_trend_warn_delta_exceeded", "warn policy")

require_equal(fail, "policy_decision", "NO-GO", "fail policy")
require_equal(fail, "trend_status", "fail", "fail policy")
require_reason(fail, "harness_script_count_trend_fail_delta_exceeded", "fail policy")
require_reason(fail, "harness_shell_line_total_trend_fail_delta_exceeded", "fail policy")

require_equal(trend_go, "policy_decision", "GO", "trend-go report")
require_equal(trend_go, "command_surface_policy_decision", "GO", "trend-go report")
require_equal(trend_go, "command_surface_trend_status", "within", "trend-go report")

require_equal(trend_fail, "policy_decision", "NO-GO", "trend-fail report")
require_equal(trend_fail, "command_surface_policy_decision", "NO-GO", "trend-fail report")
require_equal(trend_fail, "command_surface_trend_status", "fail", "trend-fail report")
require_reason(
    trend_fail,
    "command_surface_script_count_trend_fail_delta_exceeded",
    "trend-fail report",
)
require_reason(
    trend_fail,
    "command_surface_shell_line_total_trend_fail_delta_exceeded",
    "trend-fail report",
)

summary = {
    "schema_version": "kamn.ci.kolme-test-harness-loc-soft-budget-contract-report.v1",
    "status": "pass",
    "go_decision": within["policy_decision"],
    "warn_decision": warn["policy_decision"],
    "fail_decision": trend_fail["policy_decision"],
    "combined_reason_code_contract": "pass",
    "command_surface_fail_reason_contract": "pass",
    "within_reason_codes": within.get("reason_codes", []),
    "warn_reason_codes": warn.get("reason_codes", []),
    "harness_fail_reason_codes": fail.get("reason_codes", []),
    "trend_fail_reason_codes": trend_fail.get("reason_codes", []),
}
output_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
status="pass"
reason_key="kolme_test_harness_loc_soft_budget_contract_ok"
if [ "$elapsed_seconds" -gt "$MAX_RUNTIME_SECONDS" ]; then
  status="fail"
  reason_key="kolme_test_harness_loc_soft_budget_contract_runtime_budget_exceeded"
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
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

go_decision="$(python3 - "$OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("go_decision", ""))
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

echo "kolme_test_harness_loc_soft_budget_contract_status=$status"
echo "kolme_test_harness_loc_soft_budget_contract_go_decision=$go_decision"
echo "kolme_test_harness_loc_soft_budget_contract_warn_decision=$warn_decision"
echo "kolme_test_harness_loc_soft_budget_contract_fail_decision=$fail_decision"
echo "kolme_test_harness_loc_soft_budget_contract_report=$(realpath "$OUTPUT_JSON")"

if [ "$status" != "pass" ]; then
  echo "Kolme soft-budget contract lane exceeded runtime budget: ${elapsed_seconds}s > ${MAX_RUNTIME_SECONDS}s" >&2
  exit 1
fi

echo "Kolme test harness LOC soft-budget contract lane tests passed."

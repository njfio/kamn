#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_test_harness_loc_report.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_kolme_test_harness_loc_soft_budget.sh"
COMMAND_SURFACE_SCRIPT="$ROOT_DIR/scripts/ci/check_script_duplication_budget.py"
DEFAULT_COMMAND_SURFACE_BUDGET_FILE="$ROOT_DIR/.ci/kolme-command-surface-soft-budget.env"
DEFAULT_COMMAND_SURFACE_BASELINE_FILE="$ROOT_DIR/.ci/kolme-command-surface-baseline.env"
DEFAULT_COMMAND_SURFACE_TREND_THRESHOLD_FILE="$ROOT_DIR/.ci/kolme-command-surface-trend-thresholds.env"

output_json=""
input_report_file=""
input_command_surface_report_file=""
command_surface_budget_file="$DEFAULT_COMMAND_SURFACE_BUDGET_FILE"
command_surface_baseline_file="$DEFAULT_COMMAND_SURFACE_BASELINE_FILE"
command_surface_trend_threshold_file="$DEFAULT_COMMAND_SURFACE_TREND_THRESHOLD_FILE"
enforce_command_surface_fail=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --report-file)
      input_report_file="${2:-}"
      shift 2
      ;;
    --command-surface-report-file)
      input_command_surface_report_file="${2:-}"
      shift 2
      ;;
    --command-surface-budget-file)
      command_surface_budget_file="${2:-}"
      shift 2
      ;;
    --command-surface-baseline-file)
      command_surface_baseline_file="${2:-}"
      shift 2
      ;;
    --command-surface-trend-threshold-file)
      command_surface_trend_threshold_file="${2:-}"
      shift 2
      ;;
    --enforce-command-surface-fail)
      enforce_command_surface_fail=1
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [ -z "$output_json" ]; then
  echo "--output-json is required" >&2
  exit 2
fi

tmp_dir="$(mktemp -d)"
cleanup_tmp_dir=true
trap '[ "$cleanup_tmp_dir" = true ] && rm -rf "$tmp_dir"' EXIT

report_file="$input_report_file"
if [ -z "$report_file" ]; then
  report_file="$tmp_dir/kolme-test-harness-loc-report.json"
  bash "$REPORT_SCRIPT" --output-json "$report_file" >/dev/null
elif [ ! -f "$report_file" ]; then
  echo "report file not found: $report_file" >&2
  exit 2
fi

policy_file="$tmp_dir/kolme-test-harness-loc-soft-budget-report.json"

bash "$CHECK_SCRIPT" --report-file "$report_file" --output-json "$policy_file" >/dev/null

command_surface_report_file="$input_command_surface_report_file"
command_surface_checker_exit_code=0
if [ -z "$command_surface_report_file" ]; then
  command_surface_report_file="$tmp_dir/kolme-command-surface-budget-report.json"
  set +e
  python3 "$COMMAND_SURFACE_SCRIPT" \
    --scripts-root "$ROOT_DIR/scripts/kolme" \
    --budget-file "$command_surface_budget_file" \
    --baseline-file "$command_surface_baseline_file" \
    --output-json "$command_surface_report_file" >/dev/null
  command_surface_checker_exit_code=$?
  set -e
  if [ ! -f "$command_surface_report_file" ]; then
    echo "command-surface checker did not emit report: $command_surface_report_file" >&2
    exit 1
  fi
else
  if [ ! -f "$command_surface_report_file" ]; then
    echo "command-surface report file not found: $command_surface_report_file" >&2
    exit 2
  fi
fi

if [ ! -f "$command_surface_trend_threshold_file" ]; then
  echo "command-surface trend threshold file not found: $command_surface_trend_threshold_file" >&2
  exit 2
fi

output_dir="$(dirname "$output_json")"
mkdir -p "$output_dir"

python3 - "$report_file" "$policy_file" "$command_surface_report_file" "$command_surface_trend_threshold_file" "$output_json" "$command_surface_checker_exit_code" "$enforce_command_surface_fail" <<'PY'
import json
import sys
from pathlib import Path

report_path = Path(sys.argv[1])
policy_path = Path(sys.argv[2])
command_surface_report_path = Path(sys.argv[3])
command_surface_threshold_path = Path(sys.argv[4])
output_path = Path(sys.argv[5])
command_surface_checker_exit_code = int(sys.argv[6])
enforce_command_surface_fail = sys.argv[7] == "1"

report = json.loads(report_path.read_text(encoding="utf-8"))
policy = json.loads(policy_path.read_text(encoding="utf-8"))

command_surface_reason_codes: list[str] = []
command_surface_status = "fail"
command_surface_trend_status = "invalid"
command_surface_policy_decision = "NO-GO"
command_surface_script_count = -1
command_surface_shell_line_total = -1
command_surface_delta_script_count = 0
command_surface_delta_shell_line_total = 0
command_surface_warn_delta_script_count_max = 0
command_surface_fail_delta_script_count_max = 0
command_surface_warn_delta_shell_line_total_max = 0
command_surface_fail_delta_shell_line_total_max = 0

def parse_env_thresholds(path: Path) -> dict[str, int]:
    values: dict[str, int] = {}
    raw_map: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise ValueError(f"invalid threshold line: {raw_line}")
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key or not value:
            raise ValueError(f"invalid threshold line: {raw_line}")
        raw_map[key] = value

    required = (
        "COMMAND_SURFACE_SCRIPT_COUNT_WARN_DELTA_MAX",
        "COMMAND_SURFACE_SCRIPT_COUNT_FAIL_DELTA_MAX",
        "COMMAND_SURFACE_SHELL_LINE_TOTAL_WARN_DELTA_MAX",
        "COMMAND_SURFACE_SHELL_LINE_TOTAL_FAIL_DELTA_MAX",
    )
    for key in required:
        if key not in raw_map:
            raise ValueError(f"missing threshold key: {key}")
        if not raw_map[key].isdigit():
            raise ValueError(f"threshold value must be non-negative integer: {key}")
        values[key] = int(raw_map[key])

    if values["COMMAND_SURFACE_SCRIPT_COUNT_FAIL_DELTA_MAX"] < values["COMMAND_SURFACE_SCRIPT_COUNT_WARN_DELTA_MAX"]:
        raise ValueError("COMMAND_SURFACE_SCRIPT_COUNT_FAIL_DELTA_MAX must be >= COMMAND_SURFACE_SCRIPT_COUNT_WARN_DELTA_MAX")
    if values["COMMAND_SURFACE_SHELL_LINE_TOTAL_FAIL_DELTA_MAX"] < values["COMMAND_SURFACE_SHELL_LINE_TOTAL_WARN_DELTA_MAX"]:
        raise ValueError("COMMAND_SURFACE_SHELL_LINE_TOTAL_FAIL_DELTA_MAX must be >= COMMAND_SURFACE_SHELL_LINE_TOTAL_WARN_DELTA_MAX")
    return values

try:
    command_surface_thresholds = parse_env_thresholds(command_surface_threshold_path)
except ValueError:
    command_surface_thresholds = {}
    command_surface_reason_codes.append("command_surface_trend_threshold_file_invalid")

try:
    command_surface_report = json.loads(command_surface_report_path.read_text(encoding="utf-8"))
except json.JSONDecodeError:
    command_surface_report = {}
    command_surface_reason_codes.append("command_surface_report_json_invalid")

if command_surface_report.get("schema_version") != "kamn.ci.script-surface-budget-report.v1":
    command_surface_reason_codes.append("command_surface_report_schema_mismatch")

raw_command_surface_status = command_surface_report.get("status")
if raw_command_surface_status in ("pass", "fail"):
    command_surface_status = raw_command_surface_status
else:
    command_surface_reason_codes.append("command_surface_status_invalid")

metrics = command_surface_report.get("metrics")
deltas = command_surface_report.get("deltas")
if isinstance(metrics, dict):
    if isinstance(metrics.get("script_count"), int):
        command_surface_script_count = metrics["script_count"]
    else:
        command_surface_reason_codes.append("command_surface_script_count_invalid")
    if isinstance(metrics.get("shell_line_total"), int):
        command_surface_shell_line_total = metrics["shell_line_total"]
    else:
        command_surface_reason_codes.append("command_surface_shell_line_total_invalid")
else:
    command_surface_reason_codes.append("command_surface_metrics_missing")

if isinstance(deltas, dict):
    if isinstance(deltas.get("script_count"), int):
        command_surface_delta_script_count = deltas["script_count"]
    else:
        command_surface_reason_codes.append("command_surface_delta_script_count_invalid")
    if isinstance(deltas.get("shell_line_total"), int):
        command_surface_delta_shell_line_total = deltas["shell_line_total"]
    else:
        command_surface_reason_codes.append("command_surface_delta_shell_line_total_invalid")
else:
    command_surface_reason_codes.append("command_surface_deltas_missing")

if command_surface_thresholds:
    command_surface_warn_delta_script_count_max = command_surface_thresholds[
        "COMMAND_SURFACE_SCRIPT_COUNT_WARN_DELTA_MAX"
    ]
    command_surface_fail_delta_script_count_max = command_surface_thresholds[
        "COMMAND_SURFACE_SCRIPT_COUNT_FAIL_DELTA_MAX"
    ]
    command_surface_warn_delta_shell_line_total_max = command_surface_thresholds[
        "COMMAND_SURFACE_SHELL_LINE_TOTAL_WARN_DELTA_MAX"
    ]
    command_surface_fail_delta_shell_line_total_max = command_surface_thresholds[
        "COMMAND_SURFACE_SHELL_LINE_TOTAL_FAIL_DELTA_MAX"
    ]

if command_surface_checker_exit_code != 0 or command_surface_status != "pass":
    command_surface_reason_codes.append("command_surface_budget_status_fail")

has_delta_parse_errors = (
    "command_surface_delta_script_count_invalid" in command_surface_reason_codes
    or "command_surface_delta_shell_line_total_invalid" in command_surface_reason_codes
)
if command_surface_thresholds and not has_delta_parse_errors:
    if command_surface_delta_script_count > command_surface_fail_delta_script_count_max:
        command_surface_reason_codes.append("command_surface_script_count_trend_fail_delta_exceeded")
    elif command_surface_delta_script_count > command_surface_warn_delta_script_count_max:
        command_surface_reason_codes.append("command_surface_script_count_trend_warn_delta_exceeded")

    if command_surface_delta_shell_line_total > command_surface_fail_delta_shell_line_total_max:
        command_surface_reason_codes.append("command_surface_shell_line_total_trend_fail_delta_exceeded")
    elif command_surface_delta_shell_line_total > command_surface_warn_delta_shell_line_total_max:
        command_surface_reason_codes.append("command_surface_shell_line_total_trend_warn_delta_exceeded")

if (
    "command_surface_script_count_trend_fail_delta_exceeded" in command_surface_reason_codes
    or "command_surface_shell_line_total_trend_fail_delta_exceeded" in command_surface_reason_codes
):
    command_surface_trend_status = "fail"
elif (
    "command_surface_script_count_trend_warn_delta_exceeded" in command_surface_reason_codes
    or "command_surface_shell_line_total_trend_warn_delta_exceeded" in command_surface_reason_codes
):
    command_surface_trend_status = "warn"
elif not command_surface_thresholds:
    command_surface_trend_status = "invalid"
elif has_delta_parse_errors:
    command_surface_trend_status = "invalid"
else:
    command_surface_trend_status = "within"

if (
    command_surface_checker_exit_code != 0
    or command_surface_status == "fail"
    or command_surface_trend_status in ("fail", "invalid")
):
    command_surface_policy_decision = "NO-GO"
elif command_surface_trend_status == "warn":
    command_surface_policy_decision = "WARN"
else:
    command_surface_policy_decision = "GO"

def decision_rank(value: str) -> int:
    if value == "NO-GO":
        return 2
    if value == "WARN":
        return 1
    return 0

harness_policy_decision = policy.get("policy_decision")
if not isinstance(harness_policy_decision, str):
    harness_policy_decision = "NO-GO"

if decision_rank(harness_policy_decision) >= decision_rank(command_surface_policy_decision):
    combined_policy_decision = harness_policy_decision
else:
    combined_policy_decision = command_surface_policy_decision

harness_reason_codes = policy.get("reason_codes", [])
if not isinstance(harness_reason_codes, list):
    harness_reason_codes = []

combined_reason_codes: list[str] = []
for code in [*harness_reason_codes, *command_surface_reason_codes]:
    if isinstance(code, str) and code and code not in combined_reason_codes:
        combined_reason_codes.append(code)

combined_review_required = bool(policy.get("review_required")) or command_surface_policy_decision != "GO"
output_status = "ok"
if enforce_command_surface_fail and command_surface_policy_decision == "NO-GO":
    output_status = "fail"

summary = {
    "schema_version": "kamn.ci.kolme-test-harness-loc-trend-report.v1",
    "status": output_status,
    "source_report_file": str(report_path.resolve()),
    "source_policy_file": str(policy_path.resolve()),
    "source_command_surface_report_file": str(command_surface_report_path.resolve()),
    "command_surface_checker_exit_code": command_surface_checker_exit_code,
    "harness_trend_status": policy.get("trend_status"),
    "harness_policy_decision": harness_policy_decision,
    "harness_reason_codes": harness_reason_codes,
    "harness_script_count": report.get("harness_script_count"),
    "harness_shell_line_total": report.get("harness_shell_line_total"),
    "soft_budget_status": policy.get("soft_budget_status"),
    "trend_status": policy.get("trend_status"),
    "policy_decision": combined_policy_decision,
    "review_required": combined_review_required,
    "reason_codes": combined_reason_codes,
    "delta_harness_script_count": policy.get("delta_harness_script_count"),
    "delta_harness_shell_line_total": policy.get("delta_harness_shell_line_total"),
    "command_surface_status": command_surface_status,
    "command_surface_trend_status": command_surface_trend_status,
    "command_surface_policy_decision": command_surface_policy_decision,
    "command_surface_reason_codes": command_surface_reason_codes,
    "command_surface_script_count": command_surface_script_count,
    "command_surface_shell_line_total": command_surface_shell_line_total,
    "command_surface_delta_script_count": command_surface_delta_script_count,
    "command_surface_delta_shell_line_total": command_surface_delta_shell_line_total,
    "command_surface_warn_delta_script_count_max": command_surface_warn_delta_script_count_max,
    "command_surface_fail_delta_script_count_max": command_surface_fail_delta_script_count_max,
    "command_surface_warn_delta_shell_line_total_max": command_surface_warn_delta_shell_line_total_max,
    "command_surface_fail_delta_shell_line_total_max": command_surface_fail_delta_shell_line_total_max,
    "enforce_command_surface_fail": enforce_command_surface_fail,
}

output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

trend_status="$(python3 - "$policy_file" <<'PY'
import json
import sys
from pathlib import Path

policy = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(policy.get("trend_status", ""))
PY
)"

policy_decision="$(python3 - "$output_json" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("policy_decision", ""))
PY
)"

reason_codes="$(python3 - "$output_json" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
codes = payload.get("reason_codes", [])
if not codes:
    print("none")
else:
    print(",".join(codes))
PY
)"

command_surface_trend_status="$(python3 - "$output_json" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("command_surface_trend_status", ""))
PY
)"

command_surface_policy_decision="$(python3 - "$output_json" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("command_surface_policy_decision", ""))
PY
)"

status_value="$(python3 - "$output_json" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("status", ""))
PY
)"

echo "status=$status_value"
echo "trend_status=$trend_status"
echo "policy_decision=$policy_decision"
echo "command_surface_trend_status=$command_surface_trend_status"
echo "command_surface_policy_decision=$command_surface_policy_decision"
echo "reason_codes=$reason_codes"
echo "report_file=$(realpath "$output_json")"

if [ "$enforce_command_surface_fail" -eq 1 ] && [ "$command_surface_policy_decision" = "NO-GO" ]; then
  exit 1
fi

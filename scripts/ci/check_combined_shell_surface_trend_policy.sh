#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEFAULT_THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/combined_shell_surface_trend_thresholds.json"
DEFAULT_REPORT_GENERATOR="$ROOT_DIR/scripts/ci/generate_combined_shell_surface_trend_report.sh"

report_file=""
threshold_file="$DEFAULT_THRESHOLD_FILE"
output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-file)
      report_file="${2:-}"
      shift 2
      ;;
    --threshold-file)
      threshold_file="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

tmp_dir="$(mktemp -d)"
cleanup_tmp_dir=true
trap '[ "$cleanup_tmp_dir" = true ] && rm -rf "$tmp_dir"' EXIT

if [[ -z "$report_file" ]]; then
  report_file="$tmp_dir/combined-shell-surface-trend-report.json"
  bash "$DEFAULT_REPORT_GENERATOR" --output-json "$report_file" >/dev/null
fi

if [[ ! -f "$report_file" ]]; then
  echo "report file not found: $report_file" >&2
  exit 2
fi
if [[ ! -f "$threshold_file" ]]; then
  echo "threshold file not found: $threshold_file" >&2
  exit 2
fi

if [[ -z "$output_json" ]]; then
  output_json="$tmp_dir/combined-shell-surface-trend-policy-report.json"
fi
mkdir -p "$(dirname "$output_json")"

python3 - "$report_file" "$threshold_file" "$output_json" <<'PY'
import json
import sys
from pathlib import Path

REASON_TAXONOMY_VERSION = (
    "kamn.ci.combined-shell-surface-trend-policy-reason-taxonomy.v1"
)
REASON_CODES_CSV = ",".join(
    [
        "combined_shell_surface_budget_status_fail",
        "combined_shell_surface_delta_ratio_invalid",
        "combined_shell_surface_delta_script_count_invalid",
        "combined_shell_surface_delta_shell_line_total_invalid",
        "combined_shell_surface_ratio_delta_fail_exceeded",
        "combined_shell_surface_ratio_delta_warn_exceeded",
        "combined_shell_surface_ratio_fail_ceiling_exceeded",
        "combined_shell_surface_ratio_invalid",
        "combined_shell_surface_ratio_warn_ceiling_exceeded",
        "combined_shell_surface_report_schema_mismatch",
        "combined_shell_surface_rust_line_total_invalid",
        "combined_shell_surface_script_count_delta_fail_exceeded",
        "combined_shell_surface_script_count_delta_warn_exceeded",
        "combined_shell_surface_script_count_invalid",
        "combined_shell_surface_shell_line_total_delta_fail_exceeded",
        "combined_shell_surface_shell_line_total_delta_warn_exceeded",
        "combined_shell_surface_shell_line_total_invalid",
        "combined_shell_surface_threshold_order_invalid",
        "combined_shell_surface_threshold_schema_mismatch",
        "combined_shell_surface_threshold_value_invalid",
    ]
)

report_path = Path(sys.argv[1])
threshold_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])

report = json.loads(report_path.read_text(encoding="utf-8"))
thresholds = json.loads(threshold_path.read_text(encoding="utf-8"))

reason_codes: list[str] = []

if report.get("schema_version") != "kamn.ci.combined-shell-surface-trend-report.v1":
    reason_codes.append("combined_shell_surface_report_schema_mismatch")
if thresholds.get("schema_version") != "kamn.ci.combined-shell-surface-trend-thresholds.v1":
    reason_codes.append("combined_shell_surface_threshold_schema_mismatch")

current = report.get("current", {})
deltas = report.get("deltas", {})
script_budget = report.get("script_budget", {})

script_count = current.get("script_count")
shell_line_total = current.get("shell_line_total")
rust_line_total = current.get("rust_line_total")
shell_to_rust_ratio = current.get("shell_to_rust_ratio")
delta_script_count = deltas.get("script_count")
delta_shell_line_total = deltas.get("shell_line_total")
delta_shell_to_rust_ratio = deltas.get("shell_to_rust_ratio")
script_budget_status = script_budget.get("status")

if script_budget_status != "pass":
    reason_codes.append("combined_shell_surface_budget_status_fail")

warn_codes: list[str] = []
fail_codes: list[str] = []

try:
    warn_script_count_increase = int(thresholds["warn_script_count_increase"])
    fail_script_count_increase = int(thresholds["fail_script_count_increase"])
    warn_shell_line_total_increase = int(thresholds["warn_shell_line_total_increase"])
    fail_shell_line_total_increase = int(thresholds["fail_shell_line_total_increase"])
    warn_shell_to_rust_ratio = float(thresholds["warn_shell_to_rust_ratio"])
    fail_shell_to_rust_ratio = float(thresholds["fail_shell_to_rust_ratio"])
    warn_shell_to_rust_ratio_increase = float(thresholds["warn_shell_to_rust_ratio_increase"])
    fail_shell_to_rust_ratio_increase = float(thresholds["fail_shell_to_rust_ratio_increase"])
except Exception:
    reason_codes.append("combined_shell_surface_threshold_value_invalid")
    warn_script_count_increase = fail_script_count_increase = 0
    warn_shell_line_total_increase = fail_shell_line_total_increase = 0
    warn_shell_to_rust_ratio = fail_shell_to_rust_ratio = 0.0
    warn_shell_to_rust_ratio_increase = fail_shell_to_rust_ratio_increase = 0.0

if warn_script_count_increase >= fail_script_count_increase:
    reason_codes.append("combined_shell_surface_threshold_order_invalid")
if warn_shell_line_total_increase >= fail_shell_line_total_increase:
    reason_codes.append("combined_shell_surface_threshold_order_invalid")
if warn_shell_to_rust_ratio >= fail_shell_to_rust_ratio:
    reason_codes.append("combined_shell_surface_threshold_order_invalid")
if warn_shell_to_rust_ratio_increase >= fail_shell_to_rust_ratio_increase:
    reason_codes.append("combined_shell_surface_threshold_order_invalid")

if not isinstance(script_count, int):
    reason_codes.append("combined_shell_surface_script_count_invalid")
if not isinstance(shell_line_total, int):
    reason_codes.append("combined_shell_surface_shell_line_total_invalid")
if not isinstance(rust_line_total, int) or rust_line_total <= 0:
    reason_codes.append("combined_shell_surface_rust_line_total_invalid")
if not isinstance(shell_to_rust_ratio, (int, float)):
    reason_codes.append("combined_shell_surface_ratio_invalid")
if not isinstance(delta_script_count, int):
    reason_codes.append("combined_shell_surface_delta_script_count_invalid")
if not isinstance(delta_shell_line_total, int):
    reason_codes.append("combined_shell_surface_delta_shell_line_total_invalid")
if not isinstance(delta_shell_to_rust_ratio, (int, float)):
    reason_codes.append("combined_shell_surface_delta_ratio_invalid")

if not reason_codes:
    if delta_script_count > fail_script_count_increase:
        fail_codes.append("combined_shell_surface_script_count_delta_fail_exceeded")
    elif delta_script_count > warn_script_count_increase:
        warn_codes.append("combined_shell_surface_script_count_delta_warn_exceeded")

    if delta_shell_line_total > fail_shell_line_total_increase:
        fail_codes.append("combined_shell_surface_shell_line_total_delta_fail_exceeded")
    elif delta_shell_line_total > warn_shell_line_total_increase:
        warn_codes.append("combined_shell_surface_shell_line_total_delta_warn_exceeded")

    if shell_to_rust_ratio > fail_shell_to_rust_ratio:
        fail_codes.append("combined_shell_surface_ratio_fail_ceiling_exceeded")
    elif shell_to_rust_ratio > warn_shell_to_rust_ratio:
        warn_codes.append("combined_shell_surface_ratio_warn_ceiling_exceeded")

    if delta_shell_to_rust_ratio > fail_shell_to_rust_ratio_increase:
        fail_codes.append("combined_shell_surface_ratio_delta_fail_exceeded")
    elif delta_shell_to_rust_ratio > warn_shell_to_rust_ratio_increase:
        warn_codes.append("combined_shell_surface_ratio_delta_warn_exceeded")

all_reason_codes = reason_codes + fail_codes + warn_codes

if reason_codes or fail_codes:
    trend_status = "fail"
    policy_decision = "NO-GO"
    status = "fail"
elif warn_codes:
    trend_status = "warn"
    policy_decision = "WARN"
    status = "ok"
else:
    trend_status = "within"
    policy_decision = "GO"
    status = "ok"

if fail_codes:
    remediation = "Prioritize wrapper-family consolidation and shared dispatch extraction in non-Kolme lanes."
elif warn_codes:
    remediation = "Monitor shell-surface growth and schedule next tranche before fail threshold breach."
else:
    remediation = "none"

policy_report = {
    "schema_version": "kamn.ci.combined-shell-surface-trend-policy-report.v1",
    "status": status,
    "policy_decision": policy_decision,
    "trend_status": trend_status,
    "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
    "reason_codes_csv": REASON_CODES_CSV,
    "reason_codes": all_reason_codes,
    "reason_codes_value": "none" if not all_reason_codes else ",".join(all_reason_codes),
    "remediation": remediation,
    "current": {
        "script_count": script_count,
        "shell_line_total": shell_line_total,
        "rust_line_total": rust_line_total,
        "shell_to_rust_ratio": shell_to_rust_ratio,
    },
    "deltas": {
        "script_count": delta_script_count,
        "shell_line_total": delta_shell_line_total,
        "shell_to_rust_ratio": delta_shell_to_rust_ratio,
    },
    "thresholds": {
        "warn_script_count_increase": warn_script_count_increase,
        "fail_script_count_increase": fail_script_count_increase,
        "warn_shell_line_total_increase": warn_shell_line_total_increase,
        "fail_shell_line_total_increase": fail_shell_line_total_increase,
        "warn_shell_to_rust_ratio": warn_shell_to_rust_ratio,
        "fail_shell_to_rust_ratio": fail_shell_to_rust_ratio,
        "warn_shell_to_rust_ratio_increase": warn_shell_to_rust_ratio_increase,
        "fail_shell_to_rust_ratio_increase": fail_shell_to_rust_ratio_increase,
    },
}

output_path.write_text(json.dumps(policy_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

reason_marker = "none" if not all_reason_codes else ",".join(all_reason_codes)
print(f"status={status}")
print(f"policy_decision={policy_decision}")
print(f"trend_status={trend_status}")
print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
print(f"reason_codes_csv={REASON_CODES_CSV}")
print(f"reason_codes={reason_marker}")
print(f"reason_codes_value={reason_marker}")
print(f"script_count={script_count}")
print(f"shell_line_total={shell_line_total}")
print(f"rust_line_total={rust_line_total}")
print(f"shell_to_rust_ratio={shell_to_rust_ratio}")
print(f"delta_script_count={delta_script_count}")
print(f"delta_shell_line_total={delta_shell_line_total}")
print(f"delta_shell_to_rust_ratio={delta_shell_to_rust_ratio}")
print(f"remediation={remediation}")

if status != "ok":
    raise SystemExit(1)
PY

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEFAULT_REPORT_GENERATOR="$ROOT_DIR/scripts/ci/generate_combined_shell_surface_trend_report.sh"
REASON_TAXONOMY_VERSION="kamn.ci.shell-rust-loc-telemetry-reason-taxonomy.v1"
REASON_CODES_CSV="shell_rust_loc_telemetry_argument_invalid,shell_rust_loc_telemetry_generator_failed,shell_rust_loc_telemetry_metric_type_invalid,shell_rust_loc_telemetry_metrics_missing,shell_rust_loc_telemetry_output_json_required,shell_rust_loc_telemetry_output_write_failed,shell_rust_loc_telemetry_report_missing,shell_rust_loc_telemetry_report_parse_failed,shell_rust_loc_telemetry_report_schema_mismatch,shell_rust_loc_telemetry_script_budget_exit_nonzero,shell_rust_loc_telemetry_script_budget_status_fail"

emit_failure_markers() {
  local reason_codes="$1"
  echo "status=fail"
  echo "final_decision=NO-GO"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes_csv=$REASON_CODES_CSV"
  echo "reason_codes=$reason_codes"
  echo "reason_codes_value=$reason_codes"
}

output_json=""
report_generator="$DEFAULT_REPORT_GENERATOR"
report_file=""
forwarded_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --report-generator)
      report_generator="${2:-}"
      shift 2
      ;;
    --report-file)
      report_file="${2:-}"
      shift 2
      ;;
    --scripts-root|--rust-root|--budget-file|--script-baseline-file|--combined-baseline-file)
      if [[ -z "${2:-}" ]]; then
        echo "missing value for argument: $1" >&2
        emit_failure_markers "shell_rust_loc_telemetry_argument_invalid"
        exit 2
      fi
      forwarded_args+=("$1" "$2")
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      emit_failure_markers "shell_rust_loc_telemetry_argument_invalid"
      exit 2
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  echo "--output-json is required" >&2
  emit_failure_markers "shell_rust_loc_telemetry_output_json_required"
  exit 2
fi

if [[ -n "$report_file" && ${#forwarded_args[@]} -gt 0 ]]; then
  echo "--report-file cannot be combined with generator source arguments" >&2
  emit_failure_markers "shell_rust_loc_telemetry_argument_invalid"
  exit 2
fi

tmp_dir="$(mktemp -d)"
cleanup_tmp_dir=true
trap '[ "$cleanup_tmp_dir" = true ] && rm -rf "$tmp_dir"' EXIT

generated_report=""
if [[ -n "$report_file" ]]; then
  generated_report="$report_file"
else
  if [[ ! -f "$report_generator" ]]; then
    echo "report generator not found: $report_generator" >&2
    emit_failure_markers "shell_rust_loc_telemetry_generator_failed"
    exit 1
  fi

  generated_report="$tmp_dir/combined-shell-surface-trend-report.json"
  set +e
  generator_output="$(bash "$report_generator" --output-json "$generated_report" "${forwarded_args[@]}" 2>&1)"
  generator_exit_code=$?
  set -e

  if [[ "$generator_exit_code" -ne 0 ]]; then
    printf '%s\n' "$generator_output" >&2
    emit_failure_markers "shell_rust_loc_telemetry_generator_failed"
    exit 1
  fi
fi

if [[ ! -f "$generated_report" ]]; then
  emit_failure_markers "shell_rust_loc_telemetry_report_missing"
  exit 1
fi

mkdir -p "$(dirname "$output_json")"

set +e
python_output="$(
python3 - "$generated_report" "$output_json" <<'PY'
import json
import sys
from pathlib import Path

REASON_TAXONOMY_VERSION = "kamn.ci.shell-rust-loc-telemetry-reason-taxonomy.v1"
REASON_CODES_CSV = ",".join(
    [
        "shell_rust_loc_telemetry_argument_invalid",
        "shell_rust_loc_telemetry_generator_failed",
        "shell_rust_loc_telemetry_metric_type_invalid",
        "shell_rust_loc_telemetry_metrics_missing",
        "shell_rust_loc_telemetry_output_json_required",
        "shell_rust_loc_telemetry_output_write_failed",
        "shell_rust_loc_telemetry_report_missing",
        "shell_rust_loc_telemetry_report_parse_failed",
        "shell_rust_loc_telemetry_report_schema_mismatch",
        "shell_rust_loc_telemetry_script_budget_exit_nonzero",
        "shell_rust_loc_telemetry_script_budget_status_fail",
    ]
)


def emit_markers(
    *,
    status: str,
    final_decision: str,
    reason_codes: list[str],
    metrics: dict,
    output_path: Path,
) -> int:
    reason_codes_value = "none" if not reason_codes else ",".join(reason_codes)
    payload = {
        "schema_version": "kamn.ci.shell-rust-loc-telemetry-report.v1",
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes": reason_codes,
        "reason_codes_value": reason_codes_value,
        "metrics": metrics,
    }

    try:
        output_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    except Exception:
        failure_codes = ["shell_rust_loc_telemetry_output_write_failed"]
        failure_codes_value = ",".join(failure_codes)
        print("status=fail")
        print("final_decision=NO-GO")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print(f"reason_codes_csv={REASON_CODES_CSV}")
        print(f"reason_codes={failure_codes_value}")
        print(f"reason_codes_value={failure_codes_value}")
        return 1

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes={reason_codes_value}")
    print(f"reason_codes_value={reason_codes_value}")

    for key in (
        "script_count",
        "shell_line_total",
        "rust_line_total",
        "shell_to_rust_ratio",
        "delta_script_count",
        "delta_shell_line_total",
        "delta_rust_line_total",
        "delta_shell_to_rust_ratio",
        "script_budget_status",
        "script_budget_checker_exit_code",
    ):
        value = metrics.get(key, "unknown")
        print(f"{key}={value}")

    print(f"output_json={output_path}")
    return 0 if status == "ok" else 1


def add_reason(reason_codes: list[str], code: str) -> None:
    if code not in reason_codes:
        reason_codes.append(code)


report_path = Path(sys.argv[1])
output_path = Path(sys.argv[2])

reason_codes: list[str] = []
metrics = {
    "script_count": "unknown",
    "shell_line_total": "unknown",
    "rust_line_total": "unknown",
    "shell_to_rust_ratio": "unknown",
    "delta_script_count": "unknown",
    "delta_shell_line_total": "unknown",
    "delta_rust_line_total": "unknown",
    "delta_shell_to_rust_ratio": "unknown",
    "script_budget_status": "unknown",
    "script_budget_checker_exit_code": "unknown",
}

try:
    report = json.loads(report_path.read_text(encoding="utf-8"))
except Exception:
    add_reason(reason_codes, "shell_rust_loc_telemetry_report_parse_failed")
    sys.exit(
        emit_markers(
            status="fail",
            final_decision="NO-GO",
            reason_codes=reason_codes,
            metrics=metrics,
            output_path=output_path,
        )
    )

if report.get("schema_version") != "kamn.ci.combined-shell-surface-trend-report.v1":
    add_reason(reason_codes, "shell_rust_loc_telemetry_report_schema_mismatch")

current = report.get("current")
deltas = report.get("deltas")
script_budget = report.get("script_budget")

if not isinstance(current, dict) or not isinstance(deltas, dict) or not isinstance(script_budget, dict):
    add_reason(reason_codes, "shell_rust_loc_telemetry_metrics_missing")
else:
    script_count = current.get("script_count")
    shell_line_total = current.get("shell_line_total")
    rust_line_total = current.get("rust_line_total")
    shell_to_rust_ratio = current.get("shell_to_rust_ratio")
    delta_script_count = deltas.get("script_count")
    delta_shell_line_total = deltas.get("shell_line_total")
    delta_rust_line_total = deltas.get("rust_line_total")
    delta_shell_to_rust_ratio = deltas.get("shell_to_rust_ratio")
    script_budget_status = script_budget.get("status")
    script_budget_checker_exit_code = script_budget.get("checker_exit_code")

    metrics.update(
        {
            "script_count": script_count,
            "shell_line_total": shell_line_total,
            "rust_line_total": rust_line_total,
            "shell_to_rust_ratio": shell_to_rust_ratio,
            "delta_script_count": delta_script_count,
            "delta_shell_line_total": delta_shell_line_total,
            "delta_rust_line_total": delta_rust_line_total,
            "delta_shell_to_rust_ratio": delta_shell_to_rust_ratio,
            "script_budget_status": script_budget_status,
            "script_budget_checker_exit_code": script_budget_checker_exit_code,
        }
    )

    if not isinstance(script_count, int) or script_count <= 0:
        add_reason(reason_codes, "shell_rust_loc_telemetry_metric_type_invalid")
    if not isinstance(shell_line_total, int) or shell_line_total <= 0:
        add_reason(reason_codes, "shell_rust_loc_telemetry_metric_type_invalid")
    if not isinstance(rust_line_total, int) or rust_line_total <= 0:
        add_reason(reason_codes, "shell_rust_loc_telemetry_metric_type_invalid")
    if not isinstance(shell_to_rust_ratio, (int, float)):
        add_reason(reason_codes, "shell_rust_loc_telemetry_metric_type_invalid")
    if not isinstance(delta_script_count, int):
        add_reason(reason_codes, "shell_rust_loc_telemetry_metric_type_invalid")
    if not isinstance(delta_shell_line_total, int):
        add_reason(reason_codes, "shell_rust_loc_telemetry_metric_type_invalid")
    if not isinstance(delta_rust_line_total, int):
        add_reason(reason_codes, "shell_rust_loc_telemetry_metric_type_invalid")
    if not isinstance(delta_shell_to_rust_ratio, (int, float)):
        add_reason(reason_codes, "shell_rust_loc_telemetry_metric_type_invalid")

    if script_budget_status != "pass":
        add_reason(reason_codes, "shell_rust_loc_telemetry_script_budget_status_fail")
    if not isinstance(script_budget_checker_exit_code, int) or script_budget_checker_exit_code != 0:
        add_reason(reason_codes, "shell_rust_loc_telemetry_script_budget_exit_nonzero")

status = "ok" if not reason_codes else "fail"
final_decision = "GO" if status == "ok" else "NO-GO"
sys.exit(
    emit_markers(
        status=status,
        final_decision=final_decision,
        reason_codes=reason_codes,
        metrics=metrics,
        output_path=output_path,
    )
)
PY
)"
python_exit_code=$?
set -e

printf '%s\n' "$python_output"
if [[ "$python_exit_code" -ne 0 ]]; then
  exit 1
fi

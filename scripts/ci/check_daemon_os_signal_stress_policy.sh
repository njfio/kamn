#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: check_daemon_os_signal_stress_policy.sh --report-file <path> --output-json <path> [options]

Options:
  --report-file <path>              Daemon stress matrix report JSON.
  --threshold-file <path>           Threshold fixture env file.
  --ci-tools-script <path>          CI tools regression script to enforce fast-mode guard markers.
  --expected-final-decision <value> Expected report final decision (default: GO).
  --output-json <path>              Output policy report JSON (required).
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_FILE=""
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/daemon_os_signal_stress_policy_thresholds.env"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
EXPECTED_FINAL_DECISION="GO"
OUTPUT_JSON=""

REASON_TAXONOMY_VERSION="kamn.ci.daemon-os-signal-stress-policy-reason-taxonomy.v1"
REASON_CODES_CSV="overload_policy_argument_invalid,overload_policy_ci_tools_fast_mode_heavy_run_leaked,overload_policy_ci_tools_fast_mode_missing_overload_test,overload_policy_ci_tools_script_missing,overload_policy_expected_decision_mismatch,overload_policy_output_json_required,overload_policy_reason_code_unknown,overload_policy_report_file_missing,overload_policy_report_json_invalid,overload_policy_report_schema_mismatch,overload_policy_runtime_budget_exceeded,overload_policy_threshold_file_missing,overload_policy_threshold_key_missing,overload_policy_threshold_value_invalid"

emit_result() {
  local status="$1"
  local final_decision="$2"
  local reason_codes="$3"
  local report_schema_version="$4"
  local runtime_seconds="$5"
  local max_runtime_seconds="$6"
  local allowed_reason_codes_csv="$7"

  echo "status=$status"
  echo "final_decision=$final_decision"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=$reason_codes"
  echo "reason_codes_csv=$REASON_CODES_CSV"
  echo "report_schema_version=$report_schema_version"
  echo "runtime_seconds=$runtime_seconds"
  echo "max_runtime_seconds=$max_runtime_seconds"
  echo "allowed_reason_codes_csv=$allowed_reason_codes_csv"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-file)
      REPORT_FILE="${2:-}"
      shift 2
      ;;
    --threshold-file)
      THRESHOLD_FILE="${2:-}"
      shift 2
      ;;
    --ci-tools-script)
      CI_TOOLS_SCRIPT="${2:-}"
      shift 2
      ;;
    --expected-final-decision)
      EXPECTED_FINAL_DECISION="${2:-}"
      shift 2
      ;;
    --output-json)
      OUTPUT_JSON="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      emit_result "fail" "NO-GO" "overload_policy_argument_invalid" "unknown" "unknown" "unknown" "unknown"
      exit 1
      ;;
  esac
done

if [[ -z "$OUTPUT_JSON" ]]; then
  emit_result "fail" "NO-GO" "overload_policy_output_json_required" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ -z "$REPORT_FILE" ]]; then
  emit_result "fail" "NO-GO" "overload_policy_report_file_missing" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -f "$REPORT_FILE" ]]; then
  emit_result "fail" "NO-GO" "overload_policy_report_file_missing" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -f "$THRESHOLD_FILE" ]]; then
  emit_result "fail" "NO-GO" "overload_policy_threshold_file_missing" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -f "$CI_TOOLS_SCRIPT" ]]; then
  emit_result "fail" "NO-GO" "overload_policy_ci_tools_script_missing" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

set +e
python_output="$(python3 - "$REPORT_FILE" "$THRESHOLD_FILE" "$CI_TOOLS_SCRIPT" "$EXPECTED_FINAL_DECISION" "$OUTPUT_JSON" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report_file = pathlib.Path(sys.argv[1])
threshold_file = pathlib.Path(sys.argv[2])
ci_tools_script = pathlib.Path(sys.argv[3])
expected_final_decision = sys.argv[4]
output_json = pathlib.Path(sys.argv[5])

REASON_TAXONOMY_VERSION = "kamn.ci.daemon-os-signal-stress-policy-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "overload_policy_argument_invalid,"
    "overload_policy_ci_tools_fast_mode_heavy_run_leaked,"
    "overload_policy_ci_tools_fast_mode_missing_overload_test,"
    "overload_policy_ci_tools_script_missing,"
    "overload_policy_expected_decision_mismatch,"
    "overload_policy_output_json_required,"
    "overload_policy_reason_code_unknown,"
    "overload_policy_report_file_missing,"
    "overload_policy_report_json_invalid,"
    "overload_policy_report_schema_mismatch,"
    "overload_policy_runtime_budget_exceeded,"
    "overload_policy_threshold_file_missing,"
    "overload_policy_threshold_key_missing,"
    "overload_policy_threshold_value_invalid"
)


def parse_thresholds(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise ValueError("invalid-threshold-line")
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def emit(
    *,
    status: str,
    final_decision: str,
    reason_codes: str,
    report_schema_version: str,
    runtime_seconds: str,
    max_runtime_seconds: str,
    allowed_reason_codes_csv: str,
    errors: list[str] | None = None,
) -> None:
    payload = {
        "schema_version": "kamn.ci.daemon-os-signal-stress-policy-report.v1",
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": reason_codes,
        "reason_codes_csv": REASON_CODES_CSV,
        "report_schema_version": report_schema_version,
        "runtime_seconds": runtime_seconds,
        "max_runtime_seconds": max_runtime_seconds,
        "allowed_reason_codes_csv": allowed_reason_codes_csv,
        "errors": errors or [],
        "report_file": str(report_file),
        "threshold_file": str(threshold_file),
        "ci_tools_script": str(ci_tools_script),
        "expected_final_decision": expected_final_decision,
    }
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes={reason_codes}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"report_schema_version={report_schema_version}")
    print(f"runtime_seconds={runtime_seconds}")
    print(f"max_runtime_seconds={max_runtime_seconds}")
    print(f"allowed_reason_codes_csv={allowed_reason_codes_csv}")
    if errors:
        for error in errors:
            print(f"overload dry-run policy failed: {error}", file=sys.stderr)


def fail(reason_codes: list[str], errors: list[str], report_schema_version: str, runtime_seconds: str, max_runtime_seconds: str, allowed_reason_codes_csv: str) -> int:
    emit(
        status="fail",
        final_decision="NO-GO",
        reason_codes=",".join(reason_codes),
        report_schema_version=report_schema_version,
        runtime_seconds=runtime_seconds,
        max_runtime_seconds=max_runtime_seconds,
        allowed_reason_codes_csv=allowed_reason_codes_csv,
        errors=errors,
    )
    return 1


thresholds: dict[str, str]
try:
    thresholds = parse_thresholds(threshold_file)
except Exception:
    raise SystemExit(
        fail(
            ["overload_policy_threshold_value_invalid"],
            ["threshold file parsing failed"],
            "unknown",
            "unknown",
            "unknown",
            "unknown",
        )
    )

required_threshold_keys = {
    "REPORT_SCHEMA_VERSION",
    "MAX_RUNTIME_SECONDS",
    "ALLOWED_REASON_CODES_CSV",
    "CI_TOOLS_REQUIRED_ENTRY",
    "CI_TOOLS_FORBIDDEN_ENTRY",
}
missing_keys = sorted(required_threshold_keys - set(thresholds.keys()))
if missing_keys:
    raise SystemExit(
        fail(
            ["overload_policy_threshold_key_missing"],
            [f"missing threshold keys: {','.join(missing_keys)}"],
            "unknown",
            "unknown",
            "unknown",
            "unknown",
        )
    )

report_schema_version = thresholds["REPORT_SCHEMA_VERSION"]
allowed_reason_codes_csv = thresholds["ALLOWED_REASON_CODES_CSV"]
allowed_reason_codes = [entry.strip() for entry in allowed_reason_codes_csv.split(",") if entry.strip()]
if not allowed_reason_codes:
    raise SystemExit(
        fail(
            ["overload_policy_threshold_value_invalid"],
            ["ALLOWED_REASON_CODES_CSV must contain at least one code"],
            report_schema_version,
            "unknown",
            "unknown",
            allowed_reason_codes_csv,
        )
    )

try:
    max_runtime_seconds_value = int(thresholds["MAX_RUNTIME_SECONDS"])
except Exception:
    raise SystemExit(
        fail(
            ["overload_policy_threshold_value_invalid"],
            ["MAX_RUNTIME_SECONDS must be an integer"],
            report_schema_version,
            "unknown",
            "unknown",
            allowed_reason_codes_csv,
        )
    )

if max_runtime_seconds_value < 0:
    raise SystemExit(
        fail(
            ["overload_policy_threshold_value_invalid"],
            ["MAX_RUNTIME_SECONDS must be non-negative"],
            report_schema_version,
            "unknown",
            str(max_runtime_seconds_value),
            allowed_reason_codes_csv,
        )
    )

try:
    report_payload = json.loads(report_file.read_text(encoding="utf-8"))
except Exception:
    raise SystemExit(
        fail(
            ["overload_policy_report_json_invalid"],
            ["report JSON is invalid"],
            report_schema_version,
            "unknown",
            str(max_runtime_seconds_value),
            allowed_reason_codes_csv,
        )
    )

errors: list[str] = []
reason_codes: list[str] = []

actual_schema = report_payload.get("schema_version")
if actual_schema != report_schema_version:
    reason_codes.append("overload_policy_report_schema_mismatch")
    errors.append("report schema_version mismatch")

runtime_seconds = report_payload.get("runtime_seconds")
if not isinstance(runtime_seconds, int) or runtime_seconds < 0:
    reason_codes.append("overload_policy_report_json_invalid")
    errors.append("runtime_seconds must be a non-negative integer")
    runtime_seconds_rendered = "unknown"
else:
    runtime_seconds_rendered = str(runtime_seconds)
    if runtime_seconds > max_runtime_seconds_value:
        reason_codes.append("overload_policy_runtime_budget_exceeded")
        errors.append("runtime budget exceeded")

actual_final_decision = report_payload.get("final_decision")
if actual_final_decision != expected_final_decision:
    reason_codes.append("overload_policy_expected_decision_mismatch")
    errors.append("final_decision does not match expected value")

actual_reason_code = report_payload.get("reason_code")
if not isinstance(actual_reason_code, str) or not actual_reason_code:
    reason_codes.append("overload_policy_report_json_invalid")
    errors.append("reason_code must be a non-empty string")
elif actual_reason_code not in allowed_reason_codes:
    reason_codes.append("overload_policy_reason_code_unknown")
    errors.append("reason_code is not in ALLOWED_REASON_CODES_CSV")

ci_tools_content = ci_tools_script.read_text(encoding="utf-8", errors="ignore")
required_entry = thresholds["CI_TOOLS_REQUIRED_ENTRY"]
forbidden_entry = thresholds["CI_TOOLS_FORBIDDEN_ENTRY"]
if required_entry not in ci_tools_content:
    reason_codes.append("overload_policy_ci_tools_fast_mode_missing_overload_test")
    errors.append("ci tools script is missing required overload test entry")
if f"scripts/ci/{forbidden_entry}" in ci_tools_content:
    reason_codes.append("overload_policy_ci_tools_fast_mode_heavy_run_leaked")
    errors.append("ci tools script leaks direct overload heavy-run command")

if reason_codes:
    deduped = sorted(dict.fromkeys(reason_codes))
    raise SystemExit(
        fail(
            deduped,
            errors,
            report_schema_version,
            runtime_seconds_rendered,
            str(max_runtime_seconds_value),
            allowed_reason_codes_csv,
        )
    )

emit(
    status="pass",
    final_decision=expected_final_decision,
    reason_codes="none",
    report_schema_version=report_schema_version,
    runtime_seconds=runtime_seconds_rendered,
    max_runtime_seconds=str(max_runtime_seconds_value),
    allowed_reason_codes_csv=allowed_reason_codes_csv,
)
PY
)"
python_status=$?
set -e

printf '%s\n' "$python_output"
exit "$python_status"

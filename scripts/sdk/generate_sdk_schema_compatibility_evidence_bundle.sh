#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/sdk/generate_sdk_schema_compatibility_evidence_bundle.sh \
    --output-file <path> \
    --lane contract|deep \
    --matrix-report-file <path> \
    --compatibility-suite-status pass|fail \
    --runtime-budget-status within|exceeded \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file=""
lane=""
matrix_report_file=""
compatibility_suite_status=""
runtime_budget_status=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --lane)
      lane="${2:-}"
      shift 2
      ;;
    --matrix-report-file)
      matrix_report_file="${2:-}"
      shift 2
      ;;
    --compatibility-suite-status)
      compatibility_suite_status="${2:-}"
      shift 2
      ;;
    --runtime-budget-status)
      runtime_budget_status="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$output_file" || -z "$lane" || -z "$matrix_report_file" || -z "$compatibility_suite_status" || -z "$runtime_budget_status" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all sdk schema compatibility evidence bundle arguments are required"
fi

if [[ ! -f "$matrix_report_file" ]]; then
  fail "matrix report file not found: $matrix_report_file"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$lane" "$matrix_report_file" "$compatibility_suite_status" "$runtime_budget_status" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys
from typing import Any, Dict, List


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    lane,
    matrix_report_file,
    compatibility_suite_status,
    runtime_budget_status,
    ci_fast_gate,
) = sys.argv[1:]

if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")
if compatibility_suite_status not in {"pass", "fail"}:
    fail("compatibility_suite_status must be pass or fail")
if runtime_budget_status not in {"within", "exceeded"}:
    fail("runtime_budget_status must be within or exceeded")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

matrix_report = json.loads(pathlib.Path(matrix_report_file).read_text(encoding="utf-8"))
required_fields = (
    "schema_version",
    "status",
    "fixture",
    "case_count",
    "failed_count",
    "failed_case_ids",
    "cases",
)
for field in required_fields:
    if field not in matrix_report:
        fail(f"matrix report missing field: {field}")

if matrix_report["schema_version"] != "kamn.sdk.parity.matrix.v1":
    fail("matrix report schema_version mismatch")
if matrix_report["status"] not in {"pass", "fail"}:
    fail("matrix report status must be pass or fail")
if not isinstance(matrix_report["fixture"], str) or not matrix_report["fixture"]:
    fail("matrix report fixture must be a non-empty string")
if not isinstance(matrix_report["case_count"], int):
    fail("matrix report case_count must be an integer")
if not isinstance(matrix_report["failed_count"], int):
    fail("matrix report failed_count must be an integer")
if not isinstance(matrix_report["failed_case_ids"], list) or not all(
    isinstance(item, str) and item for item in matrix_report["failed_case_ids"]
):
    fail("matrix report failed_case_ids must contain non-empty strings")
if matrix_report["failed_case_ids"] != sorted(matrix_report["failed_case_ids"]):
    fail("matrix report failed_case_ids must be sorted and deterministic")
if not isinstance(matrix_report["cases"], list):
    fail("matrix report cases must be an array")

normalized_cases: List[Dict[str, Any]] = []
for index, case in enumerate(matrix_report["cases"]):
    if not isinstance(case, dict):
        fail(f"matrix report case[{index}] must be an object")
    case_id = case.get("id")
    passed = case.get("passed")
    if not isinstance(case_id, str) or not case_id:
        fail(f"matrix report case[{index}] id must be a non-empty string")
    if not isinstance(passed, bool):
        fail(f"matrix report case[{index}] passed must be a boolean")
    normalized_cases.append({"id": case_id, "passed": passed})

failed_case_ids = list(matrix_report["failed_case_ids"])
matrix_passed = (
    matrix_report["status"] == "pass"
    and matrix_report["failed_count"] == 0
    and len(failed_case_ids) == 0
    and all(case["passed"] for case in normalized_cases)
)

policy_checks = {
    "matrix_passed": matrix_passed,
    "compatibility_suite_passed": compatibility_suite_status == "pass",
    "runtime_budget_within": runtime_budget_status == "within",
    "ci_fast_gate_passed": ci_fast_gate == "PASS",
}

is_go = all(policy_checks.values())
final_decision = "GO" if is_go else "NO-GO"

reason_codes: List[str] = []
if matrix_report["status"] != "pass":
    reason_codes.append("matrix_status_not_pass")
if matrix_report["failed_count"] != 0 or failed_case_ids:
    reason_codes.append("matrix_failures_present")
if compatibility_suite_status != "pass":
    reason_codes.append("compatibility_suite_failed")
if runtime_budget_status != "within":
    reason_codes.append("runtime_budget_exceeded")
if ci_fast_gate != "PASS":
    reason_codes.append("ci_fast_gate_failed")
reason_codes = sorted(reason_codes)

payload = {
    "schema_version": "kamn.sdk.schema-compatibility-evidence.v1",
    "generated_at": generated_at,
    "lane": lane,
    "evidence_key": f"sdk_schema_compatibility_contract:{lane}:v1",
    "reason_key": f"sdk_schema_compatibility_reason_codes:{final_decision}:v1",
    "matrix_summary": {
        "schema_version": matrix_report["schema_version"],
        "fixture": matrix_report["fixture"],
        "status": matrix_report["status"],
        "case_count": matrix_report["case_count"],
        "failed_count": matrix_report["failed_count"],
        "failed_case_ids": failed_case_ids,
    },
    "compatibility_suite_status": compatibility_suite_status,
    "runtime_budget_status": runtime_budget_status,
    "ci_fast_gate": ci_fast_gate,
    "policy_checks": policy_checks,
    "reason_codes": reason_codes,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'schema_version=kamn.sdk.schema-compatibility-evidence.v1\n'
printf 'evidence_key=sdk_schema_compatibility_contract:%s:v1\n' "$lane"
printf 'reason_key=sdk_schema_compatibility_reason_codes:%s:v1\n' "$final_decision"
printf 'final_decision=%s\n' "$final_decision"

#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/sdk/check_sdk_schema_compatibility_policy.sh \
    --bundle-file <path>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

bundle_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-file)
      bundle_file="${2:-}"
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

if [[ -z "$bundle_file" ]]; then
  usage
  fail "--bundle-file is required"
fi

if [[ ! -f "$bundle_file" ]]; then
  fail "bundle file not found: $bundle_file"
fi

output="$(
  python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys
from typing import List


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


bundle_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(bundle_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "lane",
    "evidence_key",
    "reason_key",
    "matrix_summary",
    "compatibility_suite_status",
    "runtime_budget_status",
    "ci_fast_gate",
    "policy_checks",
    "reason_codes",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.sdk.schema-compatibility-evidence.v1":
    fail("unexpected schema_version for sdk schema compatibility evidence bundle")

lane = payload["lane"]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")

expected_evidence_key = f"sdk_schema_compatibility_contract:{lane}:v1"
if payload["evidence_key"] != expected_evidence_key:
    fail(
        "evidence_key mismatch: "
        f"expected {expected_evidence_key}, found {payload['evidence_key']}"
    )

compatibility_suite_status = payload["compatibility_suite_status"]
if compatibility_suite_status not in {"pass", "fail"}:
    fail("compatibility_suite_status must be pass or fail")

runtime_budget_status = payload["runtime_budget_status"]
if runtime_budget_status not in {"within", "exceeded"}:
    fail("runtime_budget_status must be within or exceeded")

ci_fast_gate = payload["ci_fast_gate"]
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

matrix_summary = payload["matrix_summary"]
if not isinstance(matrix_summary, dict):
    fail("matrix_summary must be an object")
for field in (
    "schema_version",
    "fixture",
    "status",
    "case_count",
    "failed_count",
    "failed_case_ids",
):
    if field not in matrix_summary:
        fail(f"matrix_summary missing field: {field}")

if matrix_summary["schema_version"] != "kamn.sdk.parity.matrix.v1":
    fail("matrix_summary.schema_version must be kamn.sdk.parity.matrix.v1")
if not isinstance(matrix_summary["fixture"], str) or not matrix_summary["fixture"]:
    fail("matrix_summary.fixture must be a non-empty string")
if matrix_summary["status"] not in {"pass", "fail"}:
    fail("matrix_summary.status must be pass or fail")
if not isinstance(matrix_summary["case_count"], int):
    fail("matrix_summary.case_count must be an integer")
if not isinstance(matrix_summary["failed_count"], int):
    fail("matrix_summary.failed_count must be an integer")

failed_case_ids = matrix_summary["failed_case_ids"]
if not isinstance(failed_case_ids, list) or not all(
    isinstance(item, str) and item for item in failed_case_ids
):
    fail("matrix_summary.failed_case_ids must contain non-empty strings")
if failed_case_ids != sorted(failed_case_ids):
    fail("matrix_summary.failed_case_ids must be sorted and deterministic")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")
for field in (
    "matrix_passed",
    "compatibility_suite_passed",
    "runtime_budget_within",
    "ci_fast_gate_passed",
):
    if field not in policy_checks:
        fail(f"policy_checks missing field: {field}")
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

derived_matrix_passed = (
    matrix_summary["status"] == "pass"
    and matrix_summary["failed_count"] == 0
    and len(failed_case_ids) == 0
)
derived_checks = {
    "matrix_passed": derived_matrix_passed,
    "compatibility_suite_passed": compatibility_suite_status == "pass",
    "runtime_budget_within": runtime_budget_status == "within",
    "ci_fast_gate_passed": ci_fast_gate == "PASS",
}

for key, value in derived_checks.items():
    if policy_checks[key] != value:
        fail(f"policy_checks.{key} does not match derived policy")

expected_decision = "GO" if all(derived_checks.values()) else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

expected_reason_key = f"sdk_schema_compatibility_reason_codes:{actual_decision}:v1"
if payload["reason_key"] != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {payload['reason_key']}"
    )

failed_checks: List[str] = []
if matrix_summary["status"] != "pass":
    failed_checks.append("matrix_status_not_pass")
if matrix_summary["failed_count"] != 0 or failed_case_ids:
    failed_checks.append("matrix_failures_present")
if compatibility_suite_status != "pass":
    failed_checks.append("compatibility_suite_failed")
if runtime_budget_status != "within":
    failed_checks.append("runtime_budget_exceeded")
if ci_fast_gate != "PASS":
    failed_checks.append("ci_fast_gate_failed")
failed_checks = sorted(failed_checks)

reason_codes = payload["reason_codes"]
if not isinstance(reason_codes, list):
    fail("reason_codes must be an array")
if not all(isinstance(item, str) and item for item in reason_codes):
    fail("reason_codes must contain non-empty strings")
if reason_codes != sorted(reason_codes):
    fail("reason_codes must be sorted and deterministic")
if reason_codes != failed_checks:
    fail(
        "reason_codes mismatch: "
        f"expected reason_codes={failed_checks}, found {reason_codes}"
    )

failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"schema_version={payload['schema_version']}")
print(f"reason_key={payload['reason_key']}")
print(f"final_decision={actual_decision}")
print(f"failed_checks={failed_checks_value}")
PY
)"

printf '%s\n' "$output"

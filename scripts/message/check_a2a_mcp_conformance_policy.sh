#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/message/check_a2a_mcp_conformance_policy.sh \
    --report-file <path>
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

report_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-file)
      report_file="${2:-}"
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

if [[ -z "$report_file" ]]; then
  usage
  fail "--report-file is required"
fi

if [[ ! -f "$report_file" ]]; then
  fail "report file not found: $report_file"
fi

output="$(
  python3 - "$report_file" <<'PY'
import json
import pathlib
import sys
from typing import Dict, List, Tuple


HEADER_TO_CONCEPTS: Dict[str, Tuple[str, str]] = {
    "Request": ("task.invoke", "tool_call"),
    "Response": ("task.result", "tool_result"),
    "Event": ("event.notify", "notification"),
}

SUPPORTED_TASK_STATES = {
    "Submitted",
    "InProgress",
    "Blocked",
    "Completed",
    "Failed",
    "Cancelled",
}


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


report_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(report_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as exc:
    fail(f"report file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "fixture_file",
    "total_cases",
    "matched_cases",
    "mismatched_cases",
    "reason_key",
    "reason_codes",
    "final_decision",
    "case_results",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing report field: {field}")

if payload["schema_version"] != "kamn.a2a_mcp.conformance-report.v1":
    fail("unsupported A2A/MCP conformance report schema version")

for field in ("total_cases", "matched_cases"):
    if not isinstance(payload[field], int):
        fail(f"{field} must be an integer")
    if payload[field] < 0:
        fail(f"{field} must be non-negative")

case_results = payload["case_results"]
if not isinstance(case_results, list) or not case_results:
    fail("case_results must be a non-empty list")

if payload["total_cases"] != len(case_results):
    fail("total_cases does not match case_results length")

declared_mismatched_cases = payload["mismatched_cases"]
if not isinstance(declared_mismatched_cases, list):
    fail("mismatched_cases must be a list")

validated_mismatched_cases: List[str] = []

for case in case_results:
    required_case_fields = (
        "case_id",
        "kamn_envelope_header",
        "kamn_task_state",
        "a2a_concept",
        "mcp_concept",
        "lifecycle_mapping_valid",
        "evidence_hash_verified",
        "ci_fast_gate",
        "expected_decision",
        "decision",
        "expectation_match",
        "policy_checks",
        "reason_codes",
    )
    for field in required_case_fields:
        if field not in case:
            fail(f"case is missing required field: {field}")

    case_id = str(case["case_id"])
    kamn_envelope_header = str(case["kamn_envelope_header"])
    kamn_task_state = str(case["kamn_task_state"])
    a2a_concept = str(case["a2a_concept"])
    mcp_concept = str(case["mcp_concept"])
    lifecycle_mapping_valid = case["lifecycle_mapping_valid"]
    evidence_hash_verified = str(case["evidence_hash_verified"])
    ci_fast_gate = str(case["ci_fast_gate"])
    expected_decision = str(case["expected_decision"])
    decision = str(case["decision"])
    expectation_match = case["expectation_match"]

    if not isinstance(lifecycle_mapping_valid, bool):
        fail(f"lifecycle_mapping_valid must be boolean for case {case_id}")
    if evidence_hash_verified not in {"PASS", "FAIL"}:
        fail(f"evidence_hash_verified must be PASS or FAIL for case {case_id}")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail(f"ci_fast_gate must be PASS or FAIL for case {case_id}")
    if expected_decision not in {"GO", "NO-GO"}:
        fail(f"expected_decision must be GO or NO-GO for case {case_id}")
    if decision not in {"GO", "NO-GO"}:
        fail(f"decision must be GO or NO-GO for case {case_id}")
    if not isinstance(expectation_match, bool):
        fail(f"expectation_match must be boolean for case {case_id}")

    expected_concepts = HEADER_TO_CONCEPTS.get(kamn_envelope_header)
    header_mapping_valid = expected_concepts == (a2a_concept, mcp_concept)
    task_state_supported = kamn_task_state in SUPPORTED_TASK_STATES
    evidence_hash_valid = evidence_hash_verified == "PASS"
    ci_fast_gate_passed = ci_fast_gate == "PASS"

    derived_checks: Dict[str, bool] = {
        "header_mapping_valid": header_mapping_valid,
        "task_state_supported": task_state_supported,
        "lifecycle_mapping_valid": lifecycle_mapping_valid,
        "evidence_hash_valid": evidence_hash_valid,
        "ci_fast_gate_passed": ci_fast_gate_passed,
    }

    policy_checks = case["policy_checks"]
    if not isinstance(policy_checks, dict):
        fail(f"policy_checks must be an object for case {case_id}")
    for key, expected in derived_checks.items():
        if key not in policy_checks:
            fail(f"policy_checks missing {key} for case {case_id}")
        if not isinstance(policy_checks[key], bool):
            fail(f"policy_checks.{key} must be boolean for case {case_id}")
        if policy_checks[key] != expected:
            fail(f"policy_checks.{key} mismatch for case {case_id}")

    derived_reason_codes: List[str] = []
    if not header_mapping_valid:
        derived_reason_codes.append("header_mapping_mismatch")
    if not task_state_supported:
        derived_reason_codes.append("task_state_unsupported")
    if not lifecycle_mapping_valid:
        derived_reason_codes.append("lifecycle_mapping_invalid")
    if not evidence_hash_valid:
        derived_reason_codes.append("evidence_hash_verification_failed")
    if not ci_fast_gate_passed:
        derived_reason_codes.append("ci_fast_gate_failed")
    derived_reason_codes = sorted(derived_reason_codes)

    case_reason_codes = case["reason_codes"]
    if not isinstance(case_reason_codes, list):
        fail(f"reason_codes must be an array for case {case_id}")
    if not all(isinstance(item, str) and item for item in case_reason_codes):
        fail(f"reason_codes must contain non-empty strings for case {case_id}")
    if case_reason_codes != sorted(case_reason_codes):
        fail(f"reason_codes must be sorted for case {case_id}")
    if case_reason_codes != derived_reason_codes:
        fail(
            f"reason_codes mismatch for case {case_id}: "
            f"expected {derived_reason_codes}, found {case_reason_codes}"
        )

    expected_case_decision = "GO" if not derived_reason_codes else "NO-GO"
    if decision != expected_case_decision:
        fail(
            f"case decision mismatch for {case_id}: "
            f"expected decision={expected_case_decision}, found {decision}"
        )

    expected_expectation_match = decision == expected_decision
    if expectation_match != expected_expectation_match:
        fail(
            f"expectation_match mismatch for {case_id}: "
            f"expected {expected_expectation_match}, found {expectation_match}"
        )

    if not expectation_match:
        validated_mismatched_cases.append(case_id)

if sorted(validated_mismatched_cases) != sorted(str(item) for item in declared_mismatched_cases):
    fail(
        "mismatched_cases mismatch: "
        f"expected {sorted(validated_mismatched_cases)}, "
        f"found {sorted(str(item) for item in declared_mismatched_cases)}"
    )

expected_matched_cases = payload["total_cases"] - len(validated_mismatched_cases)
if payload["matched_cases"] != expected_matched_cases:
    fail(
        "matched_cases mismatch: "
        f"expected {expected_matched_cases}, found {payload['matched_cases']}"
    )

expected_final_decision = "GO" if not validated_mismatched_cases else "NO-GO"
actual_final_decision = payload["final_decision"]
if actual_final_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_final_decision != expected_final_decision:
    fail(
        "final_decision mismatch: "
        f"expected {expected_final_decision}, found {actual_final_decision}"
    )

expected_reason_key = f"a2a_mcp_conformance_reason_codes:{actual_final_decision}:v1"
if payload["reason_key"] != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {payload['reason_key']}"
    )

expected_reason_codes = [] if actual_final_decision == "GO" else ["expected_decision_mismatch"]
if payload["reason_codes"] != expected_reason_codes:
    fail(
        "report reason_codes mismatch: "
        f"expected {expected_reason_codes}, found {payload['reason_codes']}"
    )

failed_cases_value = ",".join(validated_mismatched_cases) if validated_mismatched_cases else "none"
print("status=ok")
print(f"report_file={report_path}")
print(f"final_decision={actual_final_decision}")
print(f"failed_cases={failed_cases_value}")
PY
)"

printf '%s\n' "$output"

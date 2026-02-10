#!/usr/bin/env python3
"""A2A/MCP conformance report policy checker."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json  # noqa: E402

HEADER_TO_CONCEPTS: dict[str, tuple[str, str]] = {
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


def check_report(args: argparse.Namespace) -> int:
    if not args.report_file:
        fail("--report-file is required")

    report_path = Path(args.report_file)
    if not report_path.is_file():
        fail(f"report file not found: {report_path}")

    payload = load_json(report_path)

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
    for field_name in required_fields:
        if field_name not in payload:
            fail(f"missing report field: {field_name}")

    if payload["schema_version"] != "kamn.a2a_mcp.conformance-report.v1":
        fail("unsupported A2A/MCP conformance report schema version")

    for field_name in ("total_cases", "matched_cases"):
        value = payload[field_name]
        if not isinstance(value, int):
            fail(f"{field_name} must be an integer")
        if value < 0:
            fail(f"{field_name} must be non-negative")

    case_results = payload["case_results"]
    if not isinstance(case_results, list) or not case_results:
        fail("case_results must be a non-empty list")
    if payload["total_cases"] != len(case_results):
        fail("total_cases does not match case_results length")

    declared_mismatched_cases = payload["mismatched_cases"]
    if not isinstance(declared_mismatched_cases, list):
        fail("mismatched_cases must be a list")

    validated_mismatched_cases: list[str] = []

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
        for field_name in required_case_fields:
            if field_name not in case:
                fail(f"case is missing required field: {field_name}")

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

        derived_checks: dict[str, bool] = {
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

        derived_reason_codes: list[str] = []
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

    if sorted(validated_mismatched_cases) != sorted(
        str(item) for item in declared_mismatched_cases
    ):
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

    expected_reason_codes: list[str] = (
        [] if actual_final_decision == "GO" else ["expected_decision_mismatch"]
    )
    if payload["reason_codes"] != expected_reason_codes:
        fail(
            "report reason_codes mismatch: "
            f"expected {expected_reason_codes}, found {payload['reason_codes']}"
        )

    failed_cases_value = (
        ",".join(validated_mismatched_cases) if validated_mismatched_cases else "none"
    )
    print("status=ok")
    print(f"report_file={report_path}")
    print(f"final_decision={actual_final_decision}")
    print(f"failed_cases={failed_cases_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="A2A/MCP conformance report policy checker."
    )
    parser.add_argument("--report-file")
    parser.set_defaults(handler=check_report)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

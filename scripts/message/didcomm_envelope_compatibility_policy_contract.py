#!/usr/bin/env python3
"""DIDComm envelope compatibility report policy checker."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json  # noqa: E402

SUPPORTED_VARIANTS = {"plaintext_request", "signed_response", "encrypted_event"}


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

    if payload["schema_version"] != "kamn.didcomm.envelope-compatibility-report.v1":
        fail("unsupported DIDComm envelope compatibility report schema version")

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
        case_required_fields = (
            "case_id",
            "message_variant",
            "recipient_key_reference_present",
            "signature_status",
            "attachment_mapping_supported",
            "metadata_version",
            "ci_fast_gate",
            "expected_decision",
            "decision",
            "expectation_match",
            "policy_checks",
            "reason_codes",
        )
        for field_name in case_required_fields:
            if field_name not in case:
                fail(f"case is missing required field: {field_name}")

        case_id = str(case["case_id"])
        message_variant = str(case["message_variant"])
        recipient_key_reference_present = case["recipient_key_reference_present"]
        signature_status = str(case["signature_status"])
        attachment_mapping_supported = case["attachment_mapping_supported"]
        metadata_version = str(case["metadata_version"])
        ci_fast_gate = str(case["ci_fast_gate"])
        expected_decision = str(case["expected_decision"])
        decision = str(case["decision"])
        expectation_match = case["expectation_match"]

        if expected_decision not in {"GO", "NO-GO"}:
            fail(f"expected_decision must be GO or NO-GO for case {case_id}")
        if decision not in {"GO", "NO-GO"}:
            fail(f"decision must be GO or NO-GO for case {case_id}")
        if not isinstance(expectation_match, bool):
            fail(f"expectation_match must be boolean for case {case_id}")
        if signature_status not in {"PASS", "FAIL"}:
            fail(f"signature_status must be PASS or FAIL for case {case_id}")
        if ci_fast_gate not in {"PASS", "FAIL"}:
            fail(f"ci_fast_gate must be PASS or FAIL for case {case_id}")
        if not isinstance(recipient_key_reference_present, bool):
            fail(f"recipient_key_reference_present must be boolean for case {case_id}")
        if not isinstance(attachment_mapping_supported, bool):
            fail(f"attachment_mapping_supported must be boolean for case {case_id}")

        policy_checks = case["policy_checks"]
        if not isinstance(policy_checks, dict):
            fail(f"policy_checks must be an object for case {case_id}")

        required_policy_fields = (
            "message_variant_supported",
            "recipient_key_reference_present",
            "signature_valid",
            "attachment_mapping_supported",
            "metadata_version_present",
            "ci_fast_gate_passed",
        )
        for field_name in required_policy_fields:
            if field_name not in policy_checks:
                fail(f"policy_checks missing {field_name} for case {case_id}")
            if not isinstance(policy_checks[field_name], bool):
                fail(f"policy_checks.{field_name} must be boolean for case {case_id}")

        derived_checks: dict[str, bool] = {
            "message_variant_supported": message_variant in SUPPORTED_VARIANTS,
            "recipient_key_reference_present": recipient_key_reference_present,
            "signature_valid": signature_status == "PASS",
            "attachment_mapping_supported": attachment_mapping_supported,
            "metadata_version_present": bool(metadata_version.strip()),
            "ci_fast_gate_passed": ci_fast_gate == "PASS",
        }
        for key, expected in derived_checks.items():
            if policy_checks[key] != expected:
                fail(f"policy_checks.{key} mismatch for case {case_id}")

        derived_reason_codes: list[str] = []
        if not derived_checks["message_variant_supported"]:
            derived_reason_codes.append("message_variant_unsupported")
        if not derived_checks["recipient_key_reference_present"]:
            derived_reason_codes.append("recipient_key_reference_missing")
        if not derived_checks["signature_valid"]:
            derived_reason_codes.append("signature_validation_failed")
        if not derived_checks["attachment_mapping_supported"]:
            derived_reason_codes.append("attachment_mapping_unsupported")
        if not derived_checks["metadata_version_present"]:
            derived_reason_codes.append("metadata_version_missing")
        if not derived_checks["ci_fast_gate_passed"]:
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

    expected_reason_key = (
        f"didcomm_envelope_compatibility_reason_codes:{actual_final_decision}:v1"
    )
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
        description="DIDComm envelope compatibility report policy checker."
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

#!/usr/bin/env python3
"""Run deterministic DIDComm envelope compatibility replay fixtures."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List


SUPPORTED_VARIANTS = {"plaintext_request", "signed_response", "encrypted_event"}


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def load_fixture(path: pathlib.Path) -> Dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"fixture file not found: {path}")
    except json.JSONDecodeError as exc:
        fail(f"fixture file is not valid JSON: {exc}")

    if payload.get("schema_version") != "kamn.didcomm.envelope-compatibility-fixture.v1":
        fail("unsupported DIDComm envelope fixture schema version")
    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        fail("fixture must include a non-empty cases list")
    return payload


def evaluate_case(case: Dict[str, Any]) -> Dict[str, Any]:
    required_fields = (
        "case_id",
        "message_variant",
        "recipient_key_reference_present",
        "signature_status",
        "attachment_mapping_supported",
        "metadata_version",
        "ci_fast_gate",
        "expected_decision",
    )
    for field in required_fields:
        if field not in case:
            fail(f"fixture case missing required field: {field}")

    case_id = str(case["case_id"])
    message_variant = str(case["message_variant"])
    expected_decision = str(case["expected_decision"])
    if expected_decision not in {"GO", "NO-GO"}:
        fail(f"expected_decision must be GO or NO-GO for case {case_id}")

    signature_status = str(case["signature_status"])
    if signature_status not in {"PASS", "FAIL"}:
        fail(f"signature_status must be PASS or FAIL for case {case_id}")
    ci_fast_gate = str(case["ci_fast_gate"])
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail(f"ci_fast_gate must be PASS or FAIL for case {case_id}")

    recipient_key_reference_present = bool(case["recipient_key_reference_present"])
    attachment_mapping_supported = bool(case["attachment_mapping_supported"])
    metadata_version_present = bool(str(case["metadata_version"]).strip())

    message_variant_supported = message_variant in SUPPORTED_VARIANTS
    signature_valid = signature_status == "PASS"
    ci_fast_gate_passed = ci_fast_gate == "PASS"

    policy_checks = {
        "message_variant_supported": message_variant_supported,
        "recipient_key_reference_present": recipient_key_reference_present,
        "signature_valid": signature_valid,
        "attachment_mapping_supported": attachment_mapping_supported,
        "metadata_version_present": metadata_version_present,
        "ci_fast_gate_passed": ci_fast_gate_passed,
    }

    reason_codes: List[str] = []
    if not message_variant_supported:
        reason_codes.append("message_variant_unsupported")
    if not recipient_key_reference_present:
        reason_codes.append("recipient_key_reference_missing")
    if not signature_valid:
        reason_codes.append("signature_validation_failed")
    if not attachment_mapping_supported:
        reason_codes.append("attachment_mapping_unsupported")
    if not metadata_version_present:
        reason_codes.append("metadata_version_missing")
    if not ci_fast_gate_passed:
        reason_codes.append("ci_fast_gate_failed")
    reason_codes = sorted(reason_codes)

    decision = "GO" if not reason_codes else "NO-GO"
    expectation_match = decision == expected_decision

    return {
        "case_id": case_id,
        "message_variant": message_variant,
        "recipient_key_reference_present": recipient_key_reference_present,
        "signature_status": signature_status,
        "attachment_mapping_supported": attachment_mapping_supported,
        "metadata_version": str(case["metadata_version"]),
        "ci_fast_gate": ci_fast_gate,
        "expected_decision": expected_decision,
        "decision": decision,
        "expectation_match": expectation_match,
        "policy_checks": policy_checks,
        "reason_codes": reason_codes,
    }


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Run deterministic DIDComm envelope compatibility replay fixtures."
    )
    parser.add_argument("--fixture", required=True, help="Path to replay fixture JSON")
    parser.add_argument("--output-json", required=True, help="Path to output report JSON")
    parser.add_argument(
        "--max-cases",
        type=int,
        default=None,
        help="Optional max number of cases to replay (for bounded smoke lanes)",
    )
    args = parser.parse_args()

    fixture_path = pathlib.Path(args.fixture)
    output_path = pathlib.Path(args.output_json)

    fixture = load_fixture(fixture_path)
    cases = fixture["cases"]
    if args.max_cases is not None:
        if args.max_cases <= 0:
            fail("--max-cases must be greater than zero")
        cases = cases[: args.max_cases]
        if not cases:
            fail("--max-cases filtered out all fixture cases")

    case_results = [evaluate_case(case) for case in cases]
    mismatched_cases = [
        result["case_id"] for result in case_results if not result["expectation_match"]
    ]
    final_decision = "GO" if not mismatched_cases else "NO-GO"
    reason_codes = [] if final_decision == "GO" else ["expected_decision_mismatch"]
    reason_key = f"didcomm_envelope_compatibility_reason_codes:{final_decision}:v1"

    report = {
        "schema_version": "kamn.didcomm.envelope-compatibility-report.v1",
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "fixture_file": str(fixture_path),
        "total_cases": len(case_results),
        "matched_cases": len(case_results) - len(mismatched_cases),
        "mismatched_cases": mismatched_cases,
        "reason_key": reason_key,
        "reason_codes": reason_codes,
        "final_decision": final_decision,
        "case_results": case_results,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    status = "pass" if final_decision == "GO" else "fail"
    print(
        "status={status};cases={cases};matched={matched};decision={decision}".format(
            status=status,
            cases=report["total_cases"],
            matched=report["matched_cases"],
            decision=final_decision,
        )
    )
    print(f"report_file={output_path}")
    print(f"reason_key={reason_key}")

    if final_decision != "GO":
        raise SystemExit(1)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Run deterministic A2A/MCP conformance fixture matrix."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List, Tuple


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


def load_fixture(path: pathlib.Path) -> Dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"fixture file not found: {path}")
    except json.JSONDecodeError as exc:
        fail(f"fixture file is not valid JSON: {exc}")

    if payload.get("schema_version") != "kamn.a2a_mcp.conformance-fixture.v1":
        fail("unsupported A2A/MCP conformance fixture schema version")
    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        fail("fixture must include a non-empty cases list")
    return payload


def evaluate_case(case: Dict[str, Any]) -> Dict[str, Any]:
    required_fields = (
        "case_id",
        "kamn_envelope_header",
        "kamn_task_state",
        "a2a_concept",
        "mcp_concept",
        "lifecycle_mapping_valid",
        "evidence_hash_verified",
        "ci_fast_gate",
        "expected_decision",
    )
    for field in required_fields:
        if field not in case:
            fail(f"fixture case missing required field: {field}")

    case_id = str(case["case_id"])
    kamn_envelope_header = str(case["kamn_envelope_header"])
    kamn_task_state = str(case["kamn_task_state"])
    a2a_concept = str(case["a2a_concept"])
    mcp_concept = str(case["mcp_concept"])
    expected_decision = str(case["expected_decision"])
    if expected_decision not in {"GO", "NO-GO"}:
        fail(f"expected_decision must be GO or NO-GO for case {case_id}")

    evidence_hash_verified = str(case["evidence_hash_verified"])
    if evidence_hash_verified not in {"PASS", "FAIL"}:
        fail(f"evidence_hash_verified must be PASS or FAIL for case {case_id}")

    ci_fast_gate = str(case["ci_fast_gate"])
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail(f"ci_fast_gate must be PASS or FAIL for case {case_id}")

    lifecycle_mapping_valid = bool(case["lifecycle_mapping_valid"])
    expected_concepts = HEADER_TO_CONCEPTS.get(kamn_envelope_header)
    header_mapping_valid = expected_concepts == (a2a_concept, mcp_concept)
    task_state_supported = kamn_task_state in SUPPORTED_TASK_STATES
    evidence_hash_valid = evidence_hash_verified == "PASS"
    ci_fast_gate_passed = ci_fast_gate == "PASS"

    policy_checks = {
        "header_mapping_valid": header_mapping_valid,
        "task_state_supported": task_state_supported,
        "lifecycle_mapping_valid": lifecycle_mapping_valid,
        "evidence_hash_valid": evidence_hash_valid,
        "ci_fast_gate_passed": ci_fast_gate_passed,
    }

    reason_codes: List[str] = []
    if not header_mapping_valid:
        reason_codes.append("header_mapping_mismatch")
    if not task_state_supported:
        reason_codes.append("task_state_unsupported")
    if not lifecycle_mapping_valid:
        reason_codes.append("lifecycle_mapping_invalid")
    if not evidence_hash_valid:
        reason_codes.append("evidence_hash_verification_failed")
    if not ci_fast_gate_passed:
        reason_codes.append("ci_fast_gate_failed")
    reason_codes = sorted(reason_codes)

    decision = "GO" if not reason_codes else "NO-GO"
    expectation_match = decision == expected_decision

    return {
        "case_id": case_id,
        "kamn_envelope_header": kamn_envelope_header,
        "kamn_task_state": kamn_task_state,
        "a2a_concept": a2a_concept,
        "mcp_concept": mcp_concept,
        "lifecycle_mapping_valid": lifecycle_mapping_valid,
        "evidence_hash_verified": evidence_hash_verified,
        "ci_fast_gate": ci_fast_gate,
        "expected_decision": expected_decision,
        "decision": decision,
        "expectation_match": expectation_match,
        "policy_checks": policy_checks,
        "reason_codes": reason_codes,
    }


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Run deterministic A2A/MCP conformance fixture matrix."
    )
    parser.add_argument("--fixture", required=True, help="Path to conformance fixture JSON")
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
    reason_key = f"a2a_mcp_conformance_reason_codes:{final_decision}:v1"

    report = {
        "schema_version": "kamn.a2a_mcp.conformance-report.v1",
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

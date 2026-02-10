#!/usr/bin/env python3
"""SDK schema compatibility evidence generator and policy checker."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.sdk.schema-compatibility-evidence.v1"
MATRIX_SCHEMA_VERSION = "kamn.sdk.parity.matrix.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _load_matrix_report(report_path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"matrix report file is not valid JSON: {error}")

    if not isinstance(payload, dict):
        fail("matrix report payload must be a JSON object")
    return payload


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.lane,
        args.matrix_report_file,
        args.compatibility_suite_status,
        args.runtime_budget_status,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all sdk schema compatibility evidence bundle arguments are required")

    lane = args.lane
    if lane not in {"contract", "deep"}:
        fail("lane must be contract or deep")

    compatibility_suite_status = args.compatibility_suite_status
    if compatibility_suite_status not in {"pass", "fail"}:
        fail("compatibility_suite_status must be pass or fail")

    runtime_budget_status = args.runtime_budget_status
    if runtime_budget_status not in {"within", "exceeded"}:
        fail("runtime_budget_status must be within or exceeded")

    ci_fast_gate = args.ci_fast_gate
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    matrix_report_path = Path(args.matrix_report_file)
    if not matrix_report_path.is_file():
        fail(f"matrix report file not found: {matrix_report_path}")

    matrix_report = _load_matrix_report(matrix_report_path)
    required_fields = (
        "schema_version",
        "status",
        "fixture",
        "case_count",
        "failed_count",
        "failed_case_ids",
        "cases",
    )
    require_keys(matrix_report, required_fields)

    if matrix_report["schema_version"] != MATRIX_SCHEMA_VERSION:
        fail("matrix report schema_version mismatch")
    if matrix_report["status"] not in {"pass", "fail"}:
        fail("matrix report status must be pass or fail")
    if not isinstance(matrix_report["fixture"], str) or not matrix_report["fixture"]:
        fail("matrix report fixture must be a non-empty string")
    if not isinstance(matrix_report["case_count"], int):
        fail("matrix report case_count must be an integer")
    if not isinstance(matrix_report["failed_count"], int):
        fail("matrix report failed_count must be an integer")

    failed_case_ids = matrix_report["failed_case_ids"]
    if not isinstance(failed_case_ids, list) or not all(
        isinstance(item, str) and item for item in failed_case_ids
    ):
        fail("matrix report failed_case_ids must contain non-empty strings")
    if failed_case_ids != sorted(failed_case_ids):
        fail("matrix report failed_case_ids must be sorted and deterministic")

    cases = matrix_report["cases"]
    if not isinstance(cases, list):
        fail("matrix report cases must be an array")

    normalized_cases: list[dict[str, Any]] = []
    for index, case in enumerate(cases):
        if not isinstance(case, dict):
            fail(f"matrix report case[{index}] must be an object")
        case_id = case.get("id")
        passed = case.get("passed")
        if not isinstance(case_id, str) or not case_id:
            fail(f"matrix report case[{index}] id must be a non-empty string")
        if not isinstance(passed, bool):
            fail(f"matrix report case[{index}] passed must be a boolean")
        normalized_cases.append({"id": case_id, "passed": passed})

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

    final_decision = GO_DECISION if all(policy_checks.values()) else NO_GO_DECISION

    reason_codes: list[str] = []
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

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
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

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"schema_version={SCHEMA_VERSION}")
    print(f"evidence_key=sdk_schema_compatibility_contract:{lane}:v1")
    print(f"reason_key=sdk_schema_compatibility_reason_codes:{final_decision}:v1")
    print(f"final_decision={final_decision}")
    return 0


def check_bundle(args: argparse.Namespace) -> int:
    if not args.bundle_file:
        fail("--bundle-file is required")

    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        fail(f"bundle file not found: {bundle_path}")

    payload = load_json(bundle_path)
    require_keys(
        payload,
        (
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
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
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
    for field_name in (
        "schema_version",
        "fixture",
        "status",
        "case_count",
        "failed_count",
        "failed_case_ids",
    ):
        if field_name not in matrix_summary:
            fail(f"matrix_summary missing field: {field_name}")

    if matrix_summary["schema_version"] != MATRIX_SCHEMA_VERSION:
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
    for field_name in (
        "matrix_passed",
        "compatibility_suite_passed",
        "runtime_budget_within",
        "ci_fast_gate_passed",
    ):
        if field_name not in policy_checks:
            fail(f"policy_checks missing field: {field_name}")
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

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

    expected_decision = GO_DECISION if all(derived_checks.values()) else NO_GO_DECISION
    actual_decision = payload["final_decision"]
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
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

    failed_checks: list[str] = []
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
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "SDK schema compatibility evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--lane")
    generate.add_argument("--matrix-report-file")
    generate.add_argument("--compatibility-suite-status")
    generate.add_argument("--runtime-budget-status")
    generate.add_argument("--ci-fast-gate")
    generate.set_defaults(handler=generate_bundle)

    check = subparsers.add_parser("check")
    check.add_argument("--bundle-file")
    check.set_defaults(handler=check_bundle)

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

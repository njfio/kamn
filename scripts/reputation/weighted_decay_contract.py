#!/usr/bin/env python3
"""Weighted decay property evidence generator and policy checker."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any, Mapping

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.reputation.weighted-decay.property-evidence.v1"
MATRIX_SCHEMA_VERSION = "kamn.reputation.weighted-decay.matrix.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_lane(raw_value: str) -> str:
    if raw_value in {"contract", "deep"}:
        return raw_value
    fail("lane must be contract or deep")


def _parse_property_suite_status(raw_value: str) -> str:
    if raw_value in {"pass", "fail"}:
        return raw_value
    fail("property-suite-status must be pass or fail")


def _parse_runtime_budget_status(raw_value: str) -> str:
    if raw_value in {"within", "exceeded"}:
        return raw_value
    fail("runtime-budget-status must be within or exceeded")


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("ci-fast-gate must be PASS or FAIL")


def _parse_report(path: str, label: str) -> dict[str, Any]:
    report_path = Path(path)
    try:
        report = json.loads(report_path.read_text())
    except json.JSONDecodeError as exc:
        fail(f"{label} report is not valid JSON: {exc}")

    if report.get("schema_version") != MATRIX_SCHEMA_VERSION:
        fail(f"{label} report schema_version mismatch")

    status = report.get("status")
    if status not in {"pass", "fail"}:
        fail(f"{label} report status must be pass or fail")

    case_count = report.get("case_count")
    failed_count = report.get("failed_count")
    failed_case_ids = report.get("failed_case_ids")
    cases = report.get("cases")

    if not isinstance(case_count, int):
        fail(f"{label} report case_count must be an integer")
    if not isinstance(failed_count, int):
        fail(f"{label} report failed_count must be an integer")
    if not isinstance(failed_case_ids, list) or not all(
        isinstance(item, str) and item for item in failed_case_ids
    ):
        fail(f"{label} report failed_case_ids must be an array of non-empty strings")
    if failed_case_ids != sorted(failed_case_ids):
        fail(f"{label} report failed_case_ids must be sorted and deterministic")
    if not isinstance(cases, list):
        fail(f"{label} report cases must be an array")

    normalized_cases: list[dict[str, Any]] = []
    for index, case in enumerate(cases):
        if not isinstance(case, dict):
            fail(f"{label} report case[{index}] must be an object")
        expected_kind = case.get("expected_abuse_penalty_kind")
        actual_kind = case.get("actual_abuse_penalty_kind")
        passed = case.get("passed")
        if not isinstance(expected_kind, str) or not expected_kind:
            fail(
                f"{label} report case[{index}] expected_abuse_penalty_kind must be a string"
            )
        if not isinstance(actual_kind, str) or not actual_kind:
            fail(
                f"{label} report case[{index}] actual_abuse_penalty_kind must be a string"
            )
        if not isinstance(passed, bool):
            fail(f"{label} report case[{index}] passed must be a boolean")
        normalized_cases.append(
            {
                "expected_abuse_penalty_kind": expected_kind,
                "actual_abuse_penalty_kind": actual_kind,
                "passed": passed,
            }
        )

    return {
        "status": status,
        "case_count": case_count,
        "failed_count": failed_count,
        "failed_case_ids": failed_case_ids,
        "cases": normalized_cases,
    }


def _penalty_observed(cases: list[dict[str, Any]], kind: str) -> bool:
    return any(
        case["expected_abuse_penalty_kind"] == kind
        and case["actual_abuse_penalty_kind"] == kind
        and case["passed"]
        for case in cases
    )


def _compute_coverage(cases: list[dict[str, Any]]) -> Mapping[str, bool]:
    return {
        "reciprocity_penalty_observed": _penalty_observed(cases, "ReciprocityRing"),
        "burst_penalty_observed": _penalty_observed(cases, "BurstSpam"),
        "churn_penalty_observed": _penalty_observed(cases, "ChurnSpike"),
    }


def _compute_decision(
    *,
    lane: str,
    compact: Mapping[str, Any],
    adversarial: Mapping[str, Any],
    coverage: Mapping[str, bool],
    property_suite_status: str,
    runtime_budget_status: str,
    ci_fast_gate: str,
) -> tuple[str, list[str]]:
    decision_reasons: list[str] = []
    if compact["status"] != "pass":
        decision_reasons.append("compact_matrix_not_pass")
    if adversarial["status"] != "pass":
        decision_reasons.append("adversarial_matrix_not_pass")
    if compact["failed_count"] != 0:
        decision_reasons.append("compact_matrix_failures_present")
    if adversarial["failed_count"] != 0:
        decision_reasons.append("adversarial_matrix_failures_present")
    if not coverage["reciprocity_penalty_observed"]:
        decision_reasons.append("reciprocity_penalty_not_observed")
    if not coverage["burst_penalty_observed"]:
        decision_reasons.append("burst_penalty_not_observed")
    if not coverage["churn_penalty_observed"]:
        decision_reasons.append("churn_penalty_not_observed")
    if property_suite_status != "pass":
        decision_reasons.append("property_suite_failed")
    if runtime_budget_status != "within":
        decision_reasons.append("runtime_budget_exceeded")
    if lane == "contract" and ci_fast_gate != "PASS":
        decision_reasons.append("ci_fast_gate_failed")

    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    if not decision_reasons:
        decision_reasons.append("all weighted decay anti-gaming property checks passed")
    return final_decision, decision_reasons


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.lane,
        args.compact_report_file,
        args.adversarial_report_file,
        args.property_suite_status,
        args.runtime_budget_status,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all weighted decay property evidence bundle arguments are required")

    if not Path(args.compact_report_file).is_file():
        fail(f"compact report file not found: {args.compact_report_file}")
    if not Path(args.adversarial_report_file).is_file():
        fail(f"adversarial report file not found: {args.adversarial_report_file}")

    lane = _parse_lane(args.lane)
    property_suite_status = _parse_property_suite_status(args.property_suite_status)
    runtime_budget_status = _parse_runtime_budget_status(args.runtime_budget_status)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)

    compact = _parse_report(args.compact_report_file, "compact")
    adversarial = _parse_report(args.adversarial_report_file, "adversarial")
    combined_cases = compact["cases"] + adversarial["cases"]
    coverage = _compute_coverage(combined_cases)

    final_decision, decision_reasons = _compute_decision(
        lane=lane,
        compact=compact,
        adversarial=adversarial,
        coverage=coverage,
        property_suite_status=property_suite_status,
        runtime_budget_status=runtime_budget_status,
        ci_fast_gate=ci_fast_gate,
    )

    evidence_key = f"weighted_decay_property_contract:{lane}:v1"
    reason_key = f"weighted_decay_property_contract_reason:{final_decision}:v1"
    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "lane": lane,
        "evidence_key": evidence_key,
        "reason_key": reason_key,
        "compact_matrix": compact,
        "adversarial_matrix": adversarial,
        "anti_gaming_coverage": coverage,
        "property_suite_status": property_suite_status,
        "runtime_budget_status": runtime_budget_status,
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"schema_version={SCHEMA_VERSION}")
    print(f"evidence_key={evidence_key}")
    print(f"reason_key={reason_key}")
    print(f"final_decision={final_decision}")
    return 0


def _validate_matrix(matrix: Any, label: str) -> dict[str, Any]:
    if not isinstance(matrix, dict):
        fail(f"{label} matrix must be an object")
    for field_name in ("status", "case_count", "failed_count", "failed_case_ids", "cases"):
        if field_name not in matrix:
            fail(f"{label} matrix missing field: {field_name}")

    if matrix["status"] not in {"pass", "fail"}:
        fail(f"{label} matrix status must be pass or fail")
    if not isinstance(matrix["case_count"], int):
        fail(f"{label} matrix case_count must be an integer")
    if not isinstance(matrix["failed_count"], int):
        fail(f"{label} matrix failed_count must be an integer")
    if not isinstance(matrix["failed_case_ids"], list) or not all(
        isinstance(item, str) and item for item in matrix["failed_case_ids"]
    ):
        fail(f"{label} matrix failed_case_ids must be an array of strings")
    if matrix["failed_case_ids"] != sorted(matrix["failed_case_ids"]):
        fail(f"{label} matrix failed_case_ids must be sorted and deterministic")
    if not isinstance(matrix["cases"], list):
        fail(f"{label} matrix cases must be an array")

    normalized_cases: list[dict[str, Any]] = []
    for index, case in enumerate(matrix["cases"]):
        if not isinstance(case, dict):
            fail(f"{label} matrix case[{index}] must be an object")
        for field_name in (
            "expected_abuse_penalty_kind",
            "actual_abuse_penalty_kind",
            "passed",
        ):
            if field_name not in case:
                fail(f"{label} matrix case[{index}] missing field: {field_name}")
        if not isinstance(case["expected_abuse_penalty_kind"], str) or not case[
            "expected_abuse_penalty_kind"
        ]:
            fail(
                f"{label} matrix case[{index}] expected_abuse_penalty_kind must be a non-empty string"
            )
        if not isinstance(case["actual_abuse_penalty_kind"], str) or not case[
            "actual_abuse_penalty_kind"
        ]:
            fail(
                f"{label} matrix case[{index}] actual_abuse_penalty_kind must be a non-empty string"
            )
        if not isinstance(case["passed"], bool):
            fail(f"{label} matrix case[{index}] passed must be a boolean")
        normalized_cases.append(case)

    return {
        "status": matrix["status"],
        "failed_count": matrix["failed_count"],
        "cases": normalized_cases,
    }


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
            "compact_matrix",
            "adversarial_matrix",
            "anti_gaming_coverage",
            "property_suite_status",
            "runtime_budget_status",
            "ci_fast_gate",
            "decision_reasons",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unexpected schema_version for weighted decay property evidence bundle")

    lane = payload["lane"]
    if lane not in {"contract", "deep"}:
        fail("lane must be contract or deep")

    expected_evidence_key = f"weighted_decay_property_contract:{lane}:v1"
    if payload["evidence_key"] != expected_evidence_key:
        fail(
            "evidence_key mismatch: "
            f"expected {expected_evidence_key}, found {payload['evidence_key']}"
        )

    property_suite_status = payload["property_suite_status"]
    if property_suite_status not in {"pass", "fail"}:
        fail("property_suite_status must be pass or fail")

    runtime_budget_status = payload["runtime_budget_status"]
    if runtime_budget_status not in {"within", "exceeded"}:
        fail("runtime_budget_status must be within or exceeded")

    ci_fast_gate = payload["ci_fast_gate"]
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    decision_reasons = payload["decision_reasons"]
    if not isinstance(decision_reasons, list) or not all(
        isinstance(item, str) and item for item in decision_reasons
    ):
        fail("decision_reasons must be an array of non-empty strings")

    compact = _validate_matrix(payload["compact_matrix"], "compact")
    adversarial = _validate_matrix(payload["adversarial_matrix"], "adversarial")
    combined_cases = compact["cases"] + adversarial["cases"]

    coverage = payload["anti_gaming_coverage"]
    if not isinstance(coverage, dict):
        fail("anti_gaming_coverage must be an object")
    for field_name in (
        "reciprocity_penalty_observed",
        "burst_penalty_observed",
        "churn_penalty_observed",
    ):
        if field_name not in coverage:
            fail(f"anti_gaming_coverage missing field: {field_name}")
        if not isinstance(coverage[field_name], bool):
            fail(f"anti_gaming_coverage.{field_name} must be a boolean")

    derived_coverage = _compute_coverage(combined_cases)
    for key, value in derived_coverage.items():
        if coverage[key] != value:
            fail(
                "anti_gaming_coverage mismatch for "
                f"{key}: expected {value}, found {coverage[key]}"
            )

    expected_go = True
    if compact["status"] != "pass":
        expected_go = False
    if adversarial["status"] != "pass":
        expected_go = False
    if compact["failed_count"] != 0:
        expected_go = False
    if adversarial["failed_count"] != 0:
        expected_go = False
    if not coverage["reciprocity_penalty_observed"]:
        expected_go = False
    if not coverage["burst_penalty_observed"]:
        expected_go = False
    if not coverage["churn_penalty_observed"]:
        expected_go = False
    if property_suite_status != "pass":
        expected_go = False
    if runtime_budget_status != "within":
        expected_go = False
    if lane == "contract" and ci_fast_gate != "PASS":
        expected_go = False

    expected_decision = GO_DECISION if expected_go else NO_GO_DECISION
    actual_decision = payload["final_decision"]
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    expected_reason_key = f"weighted_decay_property_contract_reason:{actual_decision}:v1"
    if payload["reason_key"] != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {payload['reason_key']}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"schema_version={payload['schema_version']}")
    print(f"evidence_key={payload['evidence_key']}")
    print(f"reason_key={payload['reason_key']}")
    print(f"final_decision={actual_decision}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Weighted decay property evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--lane")
    generate.add_argument("--compact-report-file")
    generate.add_argument("--adversarial-report-file")
    generate.add_argument("--property-suite-status")
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

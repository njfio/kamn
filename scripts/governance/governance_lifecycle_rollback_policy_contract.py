#!/usr/bin/env python3
"""Governance lifecycle rollback report policy checker."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json  # noqa: E402

REASON_TAXONOMY_VERSION = "kamn.governance.lifecycle-rollback-reason-taxonomy.v1"
REASON_TAXONOMY_CODES_CSV = (
    "docs_contract_missing,governance_lifecycle_lane_failed,lifecycle_contract_missing,"
    "rollback_contract_missing,rollback_gate_progress_stalled,"
    "runbook_marker_parity_bypass_detected,runtime_budget_exceeded"
)


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
        "max_runtime_seconds",
        "runtime_seconds",
        "checks",
        "commands",
        "reason_taxonomy_version",
        "reason_taxonomy_codes_csv",
        "decision_reasons",
        "final_decision",
        "reason_key",
    )
    for field_name in required_fields:
        if field_name not in payload:
            fail(f"missing report field: {field_name}")

    if payload["schema_version"] != "kamn.governance.lifecycle-rollback-report.v1":
        fail("unexpected governance lifecycle/rollback report schema_version")

    max_runtime_seconds = payload["max_runtime_seconds"]
    runtime_seconds = payload["runtime_seconds"]
    if not isinstance(max_runtime_seconds, int) or max_runtime_seconds < 0:
        fail("max_runtime_seconds must be an integer >= 0")
    if not isinstance(runtime_seconds, int) or runtime_seconds < 0:
        fail("runtime_seconds must be an integer >= 0")

    checks = payload["checks"]
    if not isinstance(checks, dict):
        fail("checks must be an object")

    for field_name in (
        "lane_failed",
        "lifecycle_contract_present",
        "rollback_contract_present",
        "docs_contract_present",
        "runtime_budget_ok",
    ):
        if field_name not in checks:
            fail(f"missing checks field: {field_name}")
        if not isinstance(checks[field_name], bool):
            fail(f"checks.{field_name} must be a boolean")

    commands = payload["commands"]
    if not isinstance(commands, list) or any(
        not isinstance(item, str) for item in commands
    ):
        fail("commands must be an array of strings")

    if payload["reason_taxonomy_version"] != REASON_TAXONOMY_VERSION:
        fail(
            "reason_taxonomy_version mismatch: "
            f"expected {REASON_TAXONOMY_VERSION}, found {payload['reason_taxonomy_version']}"
        )
    if payload["reason_taxonomy_codes_csv"] != REASON_TAXONOMY_CODES_CSV:
        fail(
            "reason_taxonomy_codes_csv mismatch: "
            f"expected {REASON_TAXONOMY_CODES_CSV}, found {payload['reason_taxonomy_codes_csv']}"
        )

    actual_reasons = payload["decision_reasons"]
    if not isinstance(actual_reasons, list) or any(
        not isinstance(item, str) for item in actual_reasons
    ):
        fail("decision_reasons must be an array of strings")

    runtime_budget_ok_expected = runtime_seconds <= max_runtime_seconds
    if checks["runtime_budget_ok"] != runtime_budget_ok_expected:
        fail(
            "checks.runtime_budget_ok mismatch: "
            f"expected {runtime_budget_ok_expected}, found {checks['runtime_budget_ok']}"
        )

    expected_reasons: list[str] = []
    if checks["lane_failed"]:
        expected_reasons.append("governance_lifecycle_lane_failed")
        expected_reasons.append("rollback_gate_progress_stalled")
    if not checks["lifecycle_contract_present"]:
        expected_reasons.append("lifecycle_contract_missing")
    if not checks["rollback_contract_present"]:
        expected_reasons.append("rollback_contract_missing")
    if not checks["docs_contract_present"]:
        expected_reasons.append("docs_contract_missing")
        expected_reasons.append("runbook_marker_parity_bypass_detected")
    if not runtime_budget_ok_expected:
        expected_reasons.append("runtime_budget_exceeded")

    if actual_reasons != expected_reasons:
        fail(
            "decision_reasons mismatch: "
            f"expected {expected_reasons}, found {actual_reasons}"
        )

    expected_decision = "GO" if not expected_reasons else "NO-GO"
    actual_decision = payload["final_decision"]
    if actual_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    expected_reason_key = f"governance_lifecycle_rollback_reason_codes:{expected_decision}:v1"
    actual_reason_key: Any = payload["reason_key"]
    if actual_reason_key != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {actual_reason_key}"
        )

    print("status=ok")
    print(f"report_file={report_path}")
    print(f"final_decision={actual_decision}")
    print(f"reason_key={actual_reason_key}")
    print(f"reason_taxonomy_version={payload['reason_taxonomy_version']}")
    print(f"reason_taxonomy_codes_csv={payload['reason_taxonomy_codes_csv']}")
    print(f"runtime_seconds={runtime_seconds}")
    print(f"max_runtime_seconds={max_runtime_seconds}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Governance lifecycle rollback report policy checker."
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

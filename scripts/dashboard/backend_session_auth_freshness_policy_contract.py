#!/usr/bin/env python3
"""Dashboard backend session/auth freshness report policy checker."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json  # noqa: E402


def check_report(args: argparse.Namespace) -> int:
    if not args.report_file:
        fail("--report-file is required")

    report_path = Path(args.report_file)
    if not report_path.is_file():
        fail(f"report file not found: {report_path}")

    payload = load_json(report_path)

    required_fields = (
        "schema_version",
        "evidence_key",
        "status",
        "final_decision",
        "reason_key",
        "elapsed_seconds",
        "max_seconds",
        "skip_commands",
        "dashboard_package_exit_code",
        "command_count",
        "commands",
        "frontend_contract_passed",
        "session_guard_passed",
        "freshness_guard_passed",
        "docs_contract_passed",
        "reason_codes",
    )
    for field_name in required_fields:
        if field_name not in payload:
            fail(f"missing field: {field_name}")

    if payload["schema_version"] != "kamn.dashboard.backend-session-auth-freshness-report.v1":
        fail("unexpected schema_version for dashboard backend session/auth freshness report")
    if payload["evidence_key"] != "dashboard_backend_session_auth_freshness:v1":
        fail("unexpected evidence_key for dashboard backend session/auth freshness report")

    status = payload["status"]
    if status not in {"pass", "fail"}:
        fail("status must be pass or fail")

    final_decision = payload["final_decision"]
    if final_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")

    expected_reason_key = f"dashboard_backend_session_auth_freshness_reason_codes:{final_decision}:v1"
    if payload["reason_key"] != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {payload['reason_key']}"
        )

    if not isinstance(payload["elapsed_seconds"], int):
        fail("elapsed_seconds must be an integer")
    if not isinstance(payload["max_seconds"], int):
        fail("max_seconds must be an integer")
    if payload["max_seconds"] < 0:
        fail("max_seconds must be non-negative")

    if not isinstance(payload["skip_commands"], bool):
        fail("skip_commands must be boolean")
    if not isinstance(payload["dashboard_package_exit_code"], int):
        fail("dashboard_package_exit_code must be an integer")

    commands = payload["commands"]
    if not isinstance(commands, list):
        fail("commands must be an array")
    if not all(isinstance(item, str) and item for item in commands):
        fail("commands must contain non-empty strings")

    command_count = payload["command_count"]
    if not isinstance(command_count, int):
        fail("command_count must be an integer")
    if command_count != len(commands):
        fail("command_count must match commands length")

    for field_name in (
        "frontend_contract_passed",
        "session_guard_passed",
        "freshness_guard_passed",
        "docs_contract_passed",
    ):
        if not isinstance(payload[field_name], bool):
            fail(f"{field_name} must be boolean")

    reason_codes = payload["reason_codes"]
    if not isinstance(reason_codes, list):
        fail("reason_codes must be an array")
    if not all(isinstance(item, str) and item for item in reason_codes):
        fail("reason_codes must contain non-empty strings")
    if reason_codes != sorted(reason_codes):
        fail("reason_codes must be sorted and deterministic")

    expected_reasons: list[str] = []
    if not payload["frontend_contract_passed"]:
        expected_reasons.append("backend_lane_failed")
    if not payload["session_guard_passed"]:
        expected_reasons.append("session_guard_missing")
    if not payload["freshness_guard_passed"]:
        expected_reasons.append("freshness_guard_missing")
    if not payload["docs_contract_passed"]:
        expected_reasons.append("docs_contract_missing")
    if payload["elapsed_seconds"] > payload["max_seconds"]:
        expected_reasons.append("runtime_budget_exceeded")
    expected_reasons = sorted(expected_reasons)

    expected_status = "pass" if not expected_reasons else "fail"
    expected_decision = "GO" if not expected_reasons else "NO-GO"

    if status != expected_status:
        fail(f"status mismatch: expected {expected_status}, found {status}")
    if final_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {final_decision}"
        )
    if reason_codes != expected_reasons:
        fail(
            "reason_codes mismatch: "
            f"expected reason_codes={expected_reasons}, found {reason_codes}"
        )

    failed_checks = ",".join(expected_reasons) if expected_reasons else "none"
    print("status=ok")
    print(f"report_file={report_path}")
    print(f"reason_key={payload['reason_key']}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Dashboard backend session/auth freshness report policy checker."
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

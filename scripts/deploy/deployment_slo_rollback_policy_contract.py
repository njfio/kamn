#!/usr/bin/env python3
"""Deployment SLO rollback report policy checker."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys
from typing import Any

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
        "dr_bundle_file",
        "generator_exit_code",
        "policy_exit_code",
        "dr_bundle_final_decision",
        "policy_final_decision",
        "deployment_lane_passed",
        "slo_gate_passed",
        "rollback_automation_passed",
        "docs_contract_passed",
        "command_count",
        "commands",
        "reason_codes",
    )
    for field_name in required_fields:
        if field_name not in payload:
            fail(f"missing field: {field_name}")

    if payload["schema_version"] != "kamn.deploy.slo-rollback-report.v1":
        fail("unexpected schema_version for deployment slo/rollback report")
    if payload["evidence_key"] != "deployment_slo_rollback:v1":
        fail("unexpected evidence_key for deployment slo/rollback report")

    status = payload["status"]
    if status not in {"pass", "fail"}:
        fail("status must be pass or fail")

    final_decision = payload["final_decision"]
    if final_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")

    expected_reason_key = f"deployment_slo_rollback_reason_codes:{final_decision}:v1"
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
    if not isinstance(payload["dr_bundle_file"], str) or not payload["dr_bundle_file"]:
        fail("dr_bundle_file must be a non-empty string")

    for field_name in ("generator_exit_code", "policy_exit_code", "command_count"):
        if not isinstance(payload[field_name], int):
            fail(f"{field_name} must be an integer")

    for field_name in ("dr_bundle_final_decision", "policy_final_decision"):
        if not isinstance(payload[field_name], str) or not payload[field_name]:
            fail(f"{field_name} must be a non-empty string")

    for field_name in (
        "deployment_lane_passed",
        "slo_gate_passed",
        "rollback_automation_passed",
        "docs_contract_passed",
    ):
        if not isinstance(payload[field_name], bool):
            fail(f"{field_name} must be boolean")

    commands = payload["commands"]
    if not isinstance(commands, list):
        fail("commands must be an array")
    if not all(isinstance(item, str) and item for item in commands):
        fail("commands must contain non-empty strings")
    if payload["command_count"] != len(commands):
        fail("command_count must match commands length")

    reason_codes = payload["reason_codes"]
    if not isinstance(reason_codes, list):
        fail("reason_codes must be an array")
    if not all(isinstance(item, str) and item for item in reason_codes):
        fail("reason_codes must contain non-empty strings")
    if reason_codes != sorted(reason_codes):
        fail("reason_codes must be sorted and deterministic")

    expected_reasons: list[str] = []
    if not payload["deployment_lane_passed"]:
        expected_reasons.append("deployment_lane_failed")
    if not payload["slo_gate_passed"]:
        expected_reasons.append("slo_gate_missing")
    if not payload["rollback_automation_passed"]:
        expected_reasons.append("rollback_automation_missing")
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
        description="Deployment SLO rollback report policy contract checker."
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

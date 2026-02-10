#!/usr/bin/env python3
"""Frontend shell determinism matrix report policy checker."""

from __future__ import annotations

import json
from pathlib import Path
import sys


def usage() -> None:
    """Print CLI usage."""
    print(
        "Usage:\n"
        "  bash scripts/frontend/check_dashboard_shell_determinism_matrix_policy.sh \\\n"
        "    --report-file <path>"
    )


def fail(message: str) -> int:
    """Emit an error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def parse_args(argv: list[str]) -> tuple[int, str | None]:
    """Parse CLI arguments and return exit code/report path."""
    report_file: str | None = None
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--report-file":
            if index + 1 >= len(argv):
                return fail("unknown argument: --report-file"), None
            report_file = argv[index + 1]
            index += 2
            continue
        if argument in {"--help", "-h"}:
            usage()
            return 0, None
        return fail(f"unknown argument: {argument}"), None

    if not report_file:
        usage()
        return fail("--report-file is required"), None

    return 200, report_file


def main(argv: list[str]) -> int:
    """Validate dashboard shell matrix report payload."""
    parse_status, report_file = parse_args(argv)
    if parse_status != 200:
        return parse_status

    report_path = Path(report_file)
    if not report_path.is_file():
        return fail(f"report file not found: {report_path}")

    payload = json.loads(report_path.read_text(encoding="utf-8"))

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
        "frontend_lane_passed",
        "healthy_state_passed",
        "stale_critical_state_passed",
        "error_state_passed",
        "docs_contract_passed",
        "reason_codes",
    )
    for field in required_fields:
        if field not in payload:
            return fail(f"missing field: {field}")

    if payload["schema_version"] != "kamn.frontend.shell-matrix-report.v1":
        return fail("unexpected schema_version for dashboard shell matrix report")
    if payload["evidence_key"] != "frontend_shell_matrix:v1":
        return fail("unexpected evidence_key for dashboard shell matrix report")

    status = payload["status"]
    if status not in {"pass", "fail"}:
        return fail("status must be pass or fail")

    final_decision = payload["final_decision"]
    if final_decision not in {"GO", "NO-GO"}:
        return fail("final_decision must be GO or NO-GO")

    expected_reason_key = f"frontend_shell_matrix_reason_codes:{final_decision}:v1"
    if payload["reason_key"] != expected_reason_key:
        return fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {payload['reason_key']}"
        )

    if not isinstance(payload["elapsed_seconds"], int):
        return fail("elapsed_seconds must be an integer")
    if not isinstance(payload["max_seconds"], int):
        return fail("max_seconds must be an integer")
    if payload["max_seconds"] < 0:
        return fail("max_seconds must be non-negative")

    if not isinstance(payload["skip_commands"], bool):
        return fail("skip_commands must be boolean")
    if not isinstance(payload["dashboard_package_exit_code"], int):
        return fail("dashboard_package_exit_code must be an integer")

    commands = payload["commands"]
    if not isinstance(commands, list):
        return fail("commands must be an array")
    if not all(isinstance(item, str) and item for item in commands):
        return fail("commands must contain non-empty strings")

    if not isinstance(payload["command_count"], int):
        return fail("command_count must be an integer")
    if payload["command_count"] != len(commands):
        return fail("command_count must match commands length")

    for field in (
        "frontend_lane_passed",
        "healthy_state_passed",
        "stale_critical_state_passed",
        "error_state_passed",
        "docs_contract_passed",
    ):
        if not isinstance(payload[field], bool):
            return fail(f"{field} must be boolean")

    reason_codes = payload["reason_codes"]
    if not isinstance(reason_codes, list):
        return fail("reason_codes must be an array")
    if not all(isinstance(item, str) and item for item in reason_codes):
        return fail("reason_codes must contain non-empty strings")
    if reason_codes != sorted(reason_codes):
        return fail("reason_codes must be sorted and deterministic")

    expected_reasons: list[str] = []
    if not payload["frontend_lane_passed"]:
        expected_reasons.append("frontend_lane_failed")
    if not payload["healthy_state_passed"]:
        expected_reasons.append("healthy_state_missing")
    if not payload["stale_critical_state_passed"]:
        expected_reasons.append("stale_critical_state_missing")
    if not payload["error_state_passed"]:
        expected_reasons.append("error_state_missing")
    if not payload["docs_contract_passed"]:
        expected_reasons.append("docs_contract_missing")
    if payload["elapsed_seconds"] > payload["max_seconds"]:
        expected_reasons.append("runtime_budget_exceeded")
    expected_reasons = sorted(expected_reasons)

    expected_status = "pass" if not expected_reasons else "fail"
    expected_decision = "GO" if not expected_reasons else "NO-GO"

    if status != expected_status:
        return fail(f"status mismatch: expected {expected_status}, found {status}")
    if final_decision != expected_decision:
        return fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {final_decision}"
        )
    if reason_codes != expected_reasons:
        return fail(
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


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

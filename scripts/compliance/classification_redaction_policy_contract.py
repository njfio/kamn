#!/usr/bin/env python3
"""Classification/redaction compliance policy checker."""

from __future__ import annotations

import json
from pathlib import Path
import sys


def usage() -> None:
    """Print CLI usage."""
    print(
        "Usage:\n"
        "  bash scripts/compliance/check_classification_redaction_policy.sh \\\n"
        "    --report-file <path>"
    )


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def parse_args(argv: list[str]) -> tuple[int, str | None]:
    """Parse CLI args and return exit code/report file path."""
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
    """Validate classification/redaction compliance report."""
    parse_status, report_file = parse_args(argv)
    if parse_status != 200:
        return parse_status

    report_path = Path(report_file)
    if not report_path.is_file():
        return fail(f"report file not found: {report_path}")

    try:
        payload = json.loads(report_path.read_text())
    except json.JSONDecodeError as exc:
        return fail(f"report file is not valid JSON: {exc}")

    required_fields = (
        "schema_version",
        "generated_at",
        "max_runtime_seconds",
        "runtime_seconds",
        "checks",
        "commands",
        "decision_reasons",
        "final_decision",
        "reason_key",
    )
    for field in required_fields:
        if field not in payload:
            return fail(f"missing report field: {field}")

    if payload["schema_version"] != "kamn.compliance.classification-redaction-report.v1":
        return fail("unexpected classification/redaction report schema_version")

    if not isinstance(payload["max_runtime_seconds"], int) or payload["max_runtime_seconds"] < 0:
        return fail("max_runtime_seconds must be an integer >= 0")
    if not isinstance(payload["runtime_seconds"], int) or payload["runtime_seconds"] < 0:
        return fail("runtime_seconds must be an integer >= 0")

    checks = payload["checks"]
    if not isinstance(checks, dict):
        return fail("checks must be an object")
    for field in (
        "lane_failed",
        "classification_contract_present",
        "redaction_contract_present",
        "docs_contract_present",
        "runtime_budget_ok",
    ):
        if field not in checks:
            return fail(f"missing checks field: {field}")
        if not isinstance(checks[field], bool):
            return fail(f"checks.{field} must be a boolean")

    commands = payload["commands"]
    if not isinstance(commands, list) or any(not isinstance(item, str) for item in commands):
        return fail("commands must be an array of strings")

    actual_reasons = payload["decision_reasons"]
    if not isinstance(actual_reasons, list) or any(
        not isinstance(item, str) for item in actual_reasons
    ):
        return fail("decision_reasons must be an array of strings")

    runtime_budget_ok_expected = payload["runtime_seconds"] <= payload["max_runtime_seconds"]
    if checks["runtime_budget_ok"] != runtime_budget_ok_expected:
        return fail(
            "checks.runtime_budget_ok mismatch: "
            f"expected {runtime_budget_ok_expected}, found {checks['runtime_budget_ok']}"
        )

    expected_reasons: list[str] = []
    if checks["lane_failed"]:
        expected_reasons.append("classification_redaction_lane_failed")
    if not checks["classification_contract_present"]:
        expected_reasons.append("classification_contract_missing")
    if not checks["redaction_contract_present"]:
        expected_reasons.append("redaction_contract_missing")
    if not checks["docs_contract_present"]:
        expected_reasons.append("docs_contract_missing")
    if not runtime_budget_ok_expected:
        expected_reasons.append("runtime_budget_exceeded")

    if actual_reasons != expected_reasons:
        return fail(
            "decision_reasons mismatch: "
            f"expected {expected_reasons}, found {actual_reasons}"
        )

    expected_decision = "GO" if not expected_reasons else "NO-GO"
    actual_decision = payload["final_decision"]
    if actual_decision not in {"GO", "NO-GO"}:
        return fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        return fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    expected_reason_key = f"classification_redaction_reason_codes:{expected_decision}:v1"
    actual_reason_key = payload["reason_key"]
    if actual_reason_key != expected_reason_key:
        return fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {actual_reason_key}"
        )

    print("status=ok")
    print(f"report_file={report_path}")
    print(f"final_decision={actual_decision}")
    print(f"reason_key={actual_reason_key}")
    print(f"runtime_seconds={payload['runtime_seconds']}")
    print(f"max_runtime_seconds={payload['max_runtime_seconds']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

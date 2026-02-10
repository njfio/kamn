#!/usr/bin/env python3
"""Live transport smoke parity report policy checker."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail  # noqa: E402


def check_report(args: argparse.Namespace) -> int:
    if not args.report_file:
        fail("--report-file is required")

    report_path = Path(args.report_file)
    if not report_path.is_file():
        fail(f"report file not found: {report_path}")

    payload = json.loads(report_path.read_text(encoding="utf-8"))

    required_fields = (
        "schema_version",
        "status",
        "final_decision",
        "elapsed_seconds",
        "max_seconds",
        "max_retries",
        "retry_attempts",
        "retry_used",
        "retry_final_status",
        "languages",
        "skip_commands",
        "force_failure",
        "command_count",
        "commands",
        "reason_codes",
    )
    for field_name in required_fields:
        if field_name not in payload:
            fail(f"missing field: {field_name}")

    if payload["schema_version"] != "kamn.sdk.live-transport-smoke-parity-report.v1":
        fail("unexpected schema_version for sdk live transport smoke parity report")

    status = payload["status"]
    if status not in {"pass", "fail"}:
        fail("status must be pass or fail")

    final_decision = payload["final_decision"]
    if final_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")

    if not isinstance(payload["elapsed_seconds"], int):
        fail("elapsed_seconds must be an integer")
    if not isinstance(payload["max_seconds"], int):
        fail("max_seconds must be an integer")
    if payload["max_seconds"] < 0:
        fail("max_seconds must be non-negative")

    if not isinstance(payload["max_retries"], int):
        fail("max_retries must be an integer")
    if payload["max_retries"] < 0 or payload["max_retries"] > 2:
        fail("max_retries must be between 0 and 2")

    if not isinstance(payload["retry_attempts"], int):
        fail("retry_attempts must be an integer")
    if payload["retry_attempts"] < 1:
        fail("retry_attempts must be at least 1")

    retry_used = payload["retry_used"]
    if not isinstance(retry_used, bool):
        fail("retry_used must be boolean")

    retry_final_status = payload["retry_final_status"]
    if retry_final_status not in {"passed", "failed"}:
        fail("retry_final_status must be passed or failed")

    languages = payload["languages"]
    if not isinstance(languages, list) or not languages:
        fail("languages must be a non-empty array")
    if not all(
        isinstance(item, str) and item in {"rust", "python", "typescript"}
        for item in languages
    ):
        fail("languages must contain only rust/python/typescript values")

    if not isinstance(payload["skip_commands"], bool):
        fail("skip_commands must be boolean")
    if not isinstance(payload["force_failure"], bool):
        fail("force_failure must be boolean")

    commands = payload["commands"]
    if not isinstance(commands, list):
        fail("commands must be an array")
    if not all(isinstance(item, str) and item for item in commands):
        fail("commands must contain non-empty command strings")

    command_count = payload["command_count"]
    if not isinstance(command_count, int):
        fail("command_count must be an integer")
    if command_count != len(commands):
        fail("command_count must match commands array length")

    reason_codes = payload["reason_codes"]
    if not isinstance(reason_codes, list):
        fail("reason_codes must be an array")
    if not all(isinstance(item, str) and item for item in reason_codes):
        fail("reason_codes must contain non-empty strings")
    if reason_codes != sorted(reason_codes):
        fail("reason_codes must be sorted and deterministic")

    max_attempts = payload["max_retries"] + 1
    retry_attempts = payload["retry_attempts"]
    if retry_attempts > max_attempts:
        fail("retry_attempts exceeds retry budget")
    if not retry_used and retry_attempts > 1:
        fail("retry_used=false is invalid when retry_attempts > 1")

    expected_reasons: list[str] = []
    if retry_final_status != "passed":
        expected_reasons.append("smoke_lane_failed")
    if retry_final_status != "passed" and retry_used:
        expected_reasons.append("retry_budget_exceeded")
    if payload["elapsed_seconds"] > payload["max_seconds"]:
        expected_reasons.append("runtime_budget_exceeded")
    expected_reasons = sorted(expected_reasons)

    expected_status = "pass" if not expected_reasons else "fail"
    if status != expected_status:
        fail(f"status mismatch: expected {expected_status}, found {status}")

    expected_decision = "GO" if not expected_reasons else "NO-GO"
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
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Live transport smoke parity report policy checker."
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

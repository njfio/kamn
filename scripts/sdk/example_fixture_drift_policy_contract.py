#!/usr/bin/env python3
"""SDK example fixture drift policy checker contract."""

from __future__ import annotations

import json
from pathlib import Path
import sys


EXPECTED_SCHEMA = "kamn.sdk.example-fixture-drift-report.v1"


def usage() -> None:
    """Print usage text."""
    print(
        "Usage:\n"
        "  bash scripts/sdk/check_example_fixture_drift_policy.sh --report-file <path>"
    )


def fail(reason: str) -> int:
    """Emit stable fail markers."""
    print("status=fail")
    print(f"reason={reason}")
    return 1


def parse_args(argv: list[str]) -> tuple[int, str | None]:
    """Parse CLI arguments."""
    report_file: str | None = None
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--report-file":
            if index + 1 >= len(argv):
                return fail("unknown-argument:--report-file"), None
            report_file = argv[index + 1]
            index += 2
            continue
        if argument in {"--help", "-h"}:
            usage()
            return 0, None
        return fail(f"unknown-argument:{argument}"), None

    if report_file is None or report_file == "":
        return fail("missing-report-file"), None
    return 200, report_file


def main(argv: list[str]) -> int:
    parse_status, report_file = parse_args(argv)
    if parse_status != 200:
        return parse_status

    report_path = Path(report_file)
    if not report_path.is_file():
        return fail("report-file-not-found")

    try:
        payload = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return fail("invalid-json")

    if payload.get("schema_version") != EXPECTED_SCHEMA:
        return fail("invalid-schema-version")

    status = payload.get("status")
    if status not in {"pass", "fail"}:
        return fail("invalid-status")

    reason_codes = payload.get("reason_codes")
    if not isinstance(reason_codes, list) or not reason_codes:
        return fail("invalid-reason-codes")

    if status == "pass":
        if reason_codes != ["none"]:
            return fail("unexpected-pass-reason-codes")
        print("status=ok")
        print("final_decision=GO")
        print("reason_codes=none")
        return 0

    if "none" in reason_codes:
        return fail("unexpected-none-reason-code")

    print("status=ok")
    print("final_decision=NO-GO")
    print(f"reason_codes={','.join(reason_codes)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

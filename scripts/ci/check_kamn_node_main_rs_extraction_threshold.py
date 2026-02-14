#!/usr/bin/env python3
"""Enforce kamn-node main.rs extraction threshold growth policy."""

from __future__ import annotations

import argparse
from datetime import date, datetime
import json
from pathlib import Path
import sys
from typing import Any

THRESHOLD_SCHEMA = "kamn.ci.kamn-node-main-rs-extraction-thresholds.v1"
EXCEPTION_SCHEMA = "kamn.ci.kamn-node-main-rs-extraction-threshold-exception.v1"
OUTPUT_SCHEMA = "kamn.ci.kamn-node-main-rs-extraction-threshold-report.v1"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check kamn-node main.rs extraction threshold policy."
    )
    parser.add_argument(
        "--source-file",
        default="crates/kamn-node/src/main.rs",
        help="Path to main.rs source file.",
    )
    parser.add_argument(
        "--threshold-file",
        default="fixtures/ci/kamn_node_main_rs_extraction_thresholds.json",
        help="Path to extraction threshold policy file.",
    )
    parser.add_argument(
        "--exception-file",
        default="",
        help="Optional tracked exception file for fail-threshold overflow.",
    )
    parser.add_argument(
        "--line-count-override",
        default="",
        help="Optional deterministic line-count override for tests.",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for checker report JSON.",
    )
    return parser.parse_args(argv)


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def print_result(
    *,
    status: str,
    policy_decision: str,
    reason_codes: list[str],
    line_count: int,
    warn_line_count: int,
    fail_line_count: int,
    exception_status: str,
    exception_tracking_issue: str,
    remediation: str,
    output_json: str,
) -> int:
    payload: dict[str, Any] = {
        "schema_version": OUTPUT_SCHEMA,
        "status": status,
        "policy_decision": policy_decision,
        "reason_codes": reason_codes,
        "line_count": line_count,
        "warn_line_count": warn_line_count,
        "fail_line_count": fail_line_count,
        "exception_status": exception_status,
        "exception_tracking_issue": exception_tracking_issue,
        "remediation": remediation,
    }
    if output_json:
        Path(output_json).write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )

    print(f"status={status}")
    print(f"policy_decision={policy_decision}")
    print(f"line_count={line_count}")
    print(f"warn_line_count={warn_line_count}")
    print(f"fail_line_count={fail_line_count}")
    print(f"exception_status={exception_status}")
    print(f"exception_tracking_issue={exception_tracking_issue}")
    print("reason_codes=" + (",".join(reason_codes) if reason_codes else "none"))
    print(f"remediation={remediation}")
    return 0 if policy_decision in {"GO", "WARN"} else 1


def fail(
    *,
    reason_code: str,
    line_count: int,
    warn_line_count: int,
    fail_line_count: int,
    exception_status: str,
    exception_tracking_issue: str,
    remediation: str,
    output_json: str,
) -> int:
    return print_result(
        status="fail",
        policy_decision="NO-GO",
        reason_codes=[reason_code],
        line_count=line_count,
        warn_line_count=warn_line_count,
        fail_line_count=fail_line_count,
        exception_status=exception_status,
        exception_tracking_issue=exception_tracking_issue,
        remediation=remediation,
        output_json=output_json,
    )


def parse_thresholds(path: Path) -> tuple[int, int]:
    try:
        payload = read_json(path)
    except json.JSONDecodeError as error:
        raise ValueError(f"threshold_json_invalid:threshold JSON invalid: {error}") from error
    if payload.get("schema_version") != THRESHOLD_SCHEMA:
        raise ValueError("threshold_schema_mismatch:unexpected threshold schema")
    warn_line_count = payload.get("warn_line_count")
    fail_line_count = payload.get("fail_line_count")
    if not isinstance(warn_line_count, int) or warn_line_count <= 0:
        raise ValueError("threshold_warn_line_count_invalid:warn_line_count must be positive int")
    if not isinstance(fail_line_count, int) or fail_line_count <= 0:
        raise ValueError("threshold_fail_line_count_invalid:fail_line_count must be positive int")
    if fail_line_count <= warn_line_count:
        raise ValueError("threshold_order_invalid:fail_line_count must be greater than warn_line_count")
    return warn_line_count, fail_line_count


def parse_exception(path: Path) -> tuple[str, date, int]:
    try:
        payload = read_json(path)
    except json.JSONDecodeError as error:
        raise ValueError(f"main_rs_threshold_exception_json_invalid:exception JSON invalid: {error}") from error
    if payload.get("schema_version") != EXCEPTION_SCHEMA:
        raise ValueError("main_rs_threshold_exception_schema_mismatch:unexpected exception schema")
    tracking_issue = payload.get("tracking_issue")
    if not isinstance(tracking_issue, str) or not tracking_issue.strip():
        raise ValueError(
            "main_rs_threshold_exception_tracking_issue_invalid:tracking_issue must be non-empty string"
        )
    expires_on = payload.get("expires_on")
    if not isinstance(expires_on, str) or not expires_on.strip():
        raise ValueError(
            "main_rs_threshold_exception_expires_on_invalid:expires_on must be non-empty string"
        )
    try:
        expiry_date = datetime.strptime(expires_on, "%Y-%m-%d").date()
    except ValueError as error:
        raise ValueError(
            "main_rs_threshold_exception_expires_on_format_invalid:expires_on must be YYYY-MM-DD"
        ) from error
    if expiry_date < date.today():
        raise ValueError("main_rs_threshold_exception_expired:tracked exception has expired")
    max_line_count = payload.get("max_line_count")
    if not isinstance(max_line_count, int) or max_line_count <= 0:
        raise ValueError(
            "main_rs_threshold_exception_max_line_count_invalid:max_line_count must be positive int"
        )
    return tracking_issue.strip(), expiry_date, max_line_count


def resolve_line_count(source_file: Path, line_count_override: str) -> int:
    if line_count_override:
        if not line_count_override.isdigit():
            raise ValueError("line_count_override_invalid:line-count-override must be a positive integer")
        value = int(line_count_override)
        if value <= 0:
            raise ValueError("line_count_override_invalid:line-count-override must be a positive integer")
        return value
    with source_file.open("r", encoding="utf-8") as handle:
        return sum(1 for _ in handle)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    root_dir = Path(__file__).resolve().parents[2]
    source_file = (root_dir / args.source_file).resolve()
    threshold_file = (root_dir / args.threshold_file).resolve()

    if not source_file.is_file():
        return fail(
            reason_code="source_file_not_found",
            line_count=0,
            warn_line_count=0,
            fail_line_count=0,
            exception_status="not-required",
            exception_tracking_issue="none",
            remediation=f"source file not found: {source_file}",
            output_json=args.output_json,
        )
    if not threshold_file.is_file():
        return fail(
            reason_code="threshold_file_not_found",
            line_count=0,
            warn_line_count=0,
            fail_line_count=0,
            exception_status="not-required",
            exception_tracking_issue="none",
            remediation=f"threshold file not found: {threshold_file}",
            output_json=args.output_json,
        )

    try:
        line_count = resolve_line_count(source_file, args.line_count_override)
    except ValueError as error:
        reason_code, _, detail = str(error).partition(":")
        return fail(
            reason_code=reason_code,
            line_count=0,
            warn_line_count=0,
            fail_line_count=0,
            exception_status="not-required",
            exception_tracking_issue="none",
            remediation=detail.strip() or "fix line-count override argument",
            output_json=args.output_json,
        )

    try:
        warn_line_count, fail_line_count = parse_thresholds(threshold_file)
    except ValueError as error:
        reason_code, _, detail = str(error).partition(":")
        return fail(
            reason_code=reason_code,
            line_count=line_count,
            warn_line_count=0,
            fail_line_count=0,
            exception_status="not-required",
            exception_tracking_issue="none",
            remediation=detail.strip() or "fix threshold file contract",
            output_json=args.output_json,
        )

    if line_count <= warn_line_count:
        return print_result(
            status="pass",
            policy_decision="GO",
            reason_codes=[],
            line_count=line_count,
            warn_line_count=warn_line_count,
            fail_line_count=fail_line_count,
            exception_status="not-required",
            exception_tracking_issue="none",
            remediation="none",
            output_json=args.output_json,
        )

    if line_count <= fail_line_count:
        return print_result(
            status="warn",
            policy_decision="WARN",
            reason_codes=["main_rs_line_count_warn_threshold_exceeded"],
            line_count=line_count,
            warn_line_count=warn_line_count,
            fail_line_count=fail_line_count,
            exception_status="not-required",
            exception_tracking_issue="none",
            remediation="extract high-churn main.rs logic into dedicated modules before fail threshold",
            output_json=args.output_json,
        )

    reason_codes = ["main_rs_line_count_fail_threshold_exceeded"]
    exception_status = "not-provided"
    exception_tracking_issue = "none"
    remediation = (
        "extract high-churn main.rs logic below fail threshold or provide a short-lived tracked exception"
    )

    if args.exception_file:
        exception_file = (root_dir / args.exception_file).resolve()
        if not exception_file.is_file():
            reason_codes.append("main_rs_threshold_exception_file_not_found")
            exception_status = "invalid"
            remediation = f"exception file not found: {exception_file}"
        else:
            try:
                exception_tracking_issue, _, max_line_count = parse_exception(exception_file)
            except ValueError as error:
                error_code, _, detail = str(error).partition(":")
                reason_codes.append(error_code)
                exception_status = "invalid"
                remediation = detail.strip() or "fix tracked exception file contract"
            else:
                if line_count > max_line_count:
                    reason_codes.append("main_rs_threshold_exception_cap_exceeded")
                    exception_status = "cap-exceeded"
                    remediation = "reduce main.rs line count under exception max_line_count or refresh exception policy"
                else:
                    return print_result(
                        status="warn",
                        policy_decision="WARN",
                        reason_codes=["main_rs_threshold_exception_applied"],
                        line_count=line_count,
                        warn_line_count=warn_line_count,
                        fail_line_count=fail_line_count,
                        exception_status="applied",
                        exception_tracking_issue=exception_tracking_issue,
                        remediation="tracked exception applied; reduce main.rs below fail threshold before exception expiry",
                        output_json=args.output_json,
                    )

    return print_result(
        status="fail",
        policy_decision="NO-GO",
        reason_codes=reason_codes,
        line_count=line_count,
        warn_line_count=warn_line_count,
        fail_line_count=fail_line_count,
        exception_status=exception_status,
        exception_tracking_issue=exception_tracking_issue,
        remediation=remediation,
        output_json=args.output_json,
    )


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))


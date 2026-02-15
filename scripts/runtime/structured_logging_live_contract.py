#!/usr/bin/env python3
"""Structured logging live validation policy contracts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

SUMMARY_SCHEMA_VERSION = "kamn.runtime.structured-logging-live-validation.v1"
POLICY_SCHEMA_VERSION = "kamn.runtime.structured-logging-live-policy-report.v1"
REASON_TAXONOMY_VERSION = (
    "kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1"
)
EXPECTED_FAIL_CLOSED_REASON_CODE = "invalid_log_config_level"


def _dedupe(values: list[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result


def _load_json(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise SystemExit(f"structured_logging_policy_report_file_unreadable:{exc}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(
            "structured_logging_policy_report_parse_error:"
            f"line={exc.lineno}:column={exc.colno}"
        ) from exc
    if not isinstance(payload, dict):
        raise SystemExit("structured_logging_policy_report_not_object")
    return payload


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def _check_policy(args: argparse.Namespace) -> int:
    report = _load_json(Path(args.report_file).resolve())

    reason_codes: list[str] = []
    expected_final_decision = args.expected_final_decision
    ci_fast_gate = args.ci_fast_gate

    if report.get("schema_version") != SUMMARY_SCHEMA_VERSION:
        reason_codes.append("structured_logging_policy_schema_version_mismatch")

    if report.get("status") != "pass":
        reason_codes.append("structured_logging_policy_status_mismatch")

    observed_final_decision = str(report.get("final_decision", "missing"))
    if observed_final_decision != expected_final_decision:
        reason_codes.append("structured_logging_policy_final_decision_mismatch")

    required_markers = {
        "structured_logging_contract_status": "verified",
        "correlation_contract_status": "verified",
        "docs_contract_status": "verified",
        "fail_closed_status": "verified",
        "performance_budget_status": "verified",
    }
    for marker, expected in required_markers.items():
        if marker not in report:
            reason_codes.append(f"structured_logging_policy_marker_missing:{marker}")
            continue
        if report.get(marker) != expected:
            reason_codes.append(f"structured_logging_policy_marker_value_mismatch:{marker}")

    if "fail_closed_reason_code" not in report:
        reason_codes.append("structured_logging_policy_marker_missing:fail_closed_reason_code")
    elif report.get("fail_closed_reason_code") != EXPECTED_FAIL_CLOSED_REASON_CODE:
        reason_codes.append("structured_logging_policy_fail_closed_reason_code_mismatch")

    if "reason_taxonomy_version" not in report:
        reason_codes.append("structured_logging_policy_marker_missing:reason_taxonomy_version")
    elif report.get("reason_taxonomy_version") != REASON_TAXONOMY_VERSION:
        reason_codes.append("structured_logging_policy_reason_taxonomy_version_mismatch")

    reason_codes = _dedupe(reason_codes)
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"
        policy_status = "failed"
        fail_closed_reason_code = reason_codes[0]
    else:
        status = "pass"
        final_decision = "GO"
        policy_status = "verified"
        reason_codes = ["none"]
        fail_closed_reason_code = "none"

    output = {
        "schema_version": POLICY_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "structured_logging_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": observed_final_decision,
        "ci_fast_gate": ci_fast_gate,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": reason_codes,
        "fail_closed_reason_code": fail_closed_reason_code,
    }
    if args.output_json:
        _write_json(Path(args.output_json).resolve(), output)

    if status == "pass":
        print("status=ok")
        print("final_decision=GO")
        print("structured_logging_policy_status=verified")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print("reason_codes_csv=none")
        return 0

    for reason in reason_codes:
        print(reason)
    return 1


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Structured logging live validation contract helpers."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_policy = subparsers.add_parser(
        "check-policy",
        help="Validate structured logging live validation summary against deterministic policy.",
    )
    check_policy.add_argument("--report-file", required=True, help="Validation summary JSON path.")
    check_policy.add_argument(
        "--expected-final-decision",
        choices=("GO", "NO-GO"),
        required=True,
        help="Expected final decision marker.",
    )
    check_policy.add_argument(
        "--ci-fast-gate",
        choices=("PASS", "FAIL"),
        required=True,
        help="Expected CI fast-gate selector.",
    )
    check_policy.add_argument(
        "--output-json",
        help="Optional output policy report JSON path.",
    )

    return parser


def main() -> int:
    parser = _build_parser()
    args = parser.parse_args()
    if args.command == "check-policy":
        return _check_policy(args)
    parser.error(f"unsupported command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())

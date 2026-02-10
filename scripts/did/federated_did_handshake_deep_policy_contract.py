#!/usr/bin/env python3
"""Federated DID handshake deep-lane policy checker."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_keys,
)

SCHEMA_VERSION = "kamn.did.federated-handshake.deep-summary.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _csv(values: list[str]) -> str:
    if not values:
        return "none"
    return ",".join(values)


def _check_report(report_path: Path) -> int:
    if not report_path.is_file():
        fail(f"report file not found: {report_path}")

    payload = load_json(report_path)
    require_keys(
        payload,
        (
            "schema_version",
            "event_name",
            "cadence",
            "contract_lane_status",
            "matrix_status",
            "matrix_case_count",
            "matrix_failed_count",
            "matrix_report_file",
            "elapsed_seconds",
            "max_seconds",
            "budget_status",
            "reason_codes",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unsupported schema_version for federated DID handshake deep summary")

    event_name = payload["event_name"]
    if event_name not in {"schedule", "workflow_dispatch"}:
        fail("event_name must be schedule or workflow_dispatch")

    cadence = payload["cadence"]
    expected_cadence = "scheduled" if event_name == "schedule" else "manual"
    if cadence != expected_cadence:
        fail(f"cadence mismatch: expected {expected_cadence}, found {cadence}")

    contract_lane_status = payload["contract_lane_status"]
    if contract_lane_status not in {"pass", "fail"}:
        fail("contract_lane_status must be pass or fail")

    matrix_status = payload["matrix_status"]
    if matrix_status not in {"pass", "fail"}:
        fail("matrix_status must be pass or fail")

    matrix_case_count = payload["matrix_case_count"]
    matrix_failed_count = payload["matrix_failed_count"]
    if not isinstance(matrix_case_count, int) or matrix_case_count < 0:
        fail("matrix_case_count must be a non-negative integer")
    if not isinstance(matrix_failed_count, int) or matrix_failed_count < 0:
        fail("matrix_failed_count must be a non-negative integer")
    if matrix_status == "pass" and matrix_failed_count != 0:
        fail("matrix_failed_count must be zero when matrix_status is pass")
    if matrix_status == "fail" and matrix_failed_count == 0:
        fail("matrix_failed_count must be positive when matrix_status is fail")

    if not isinstance(payload["matrix_report_file"], str) or not payload["matrix_report_file"]:
        fail("matrix_report_file must be a non-empty string")

    elapsed_seconds = payload["elapsed_seconds"]
    max_seconds = payload["max_seconds"]
    if not isinstance(elapsed_seconds, int) or elapsed_seconds < 0:
        fail("elapsed_seconds must be a non-negative integer")
    if not isinstance(max_seconds, int) or max_seconds <= 0:
        fail("max_seconds must be a positive integer")

    budget_status = payload["budget_status"]
    if budget_status not in {"within", "exceeded"}:
        fail("budget_status must be within or exceeded")
    if budget_status == "within" and elapsed_seconds > max_seconds:
        fail("budget_status within is inconsistent with elapsed_seconds/max_seconds")
    if budget_status == "exceeded" and elapsed_seconds <= max_seconds:
        fail("budget_status exceeded is inconsistent with elapsed_seconds/max_seconds")

    reason_codes = payload["reason_codes"]
    if not isinstance(reason_codes, list) or not all(isinstance(item, str) for item in reason_codes):
        fail("reason_codes must be a string array")
    if len(reason_codes) != len(set(reason_codes)):
        fail("reason_codes must not contain duplicates")

    expected_reason_codes: list[str] = []
    if contract_lane_status != "pass":
        expected_reason_codes.append("contract_lane_failed")
    if matrix_status != "pass":
        expected_reason_codes.append("matrix_failed")
    if budget_status != "within":
        expected_reason_codes.append("runtime_budget_exceeded")

    actual_reason_codes = sorted(reason_codes)
    expected_reason_codes_sorted = sorted(expected_reason_codes)
    if actual_reason_codes != expected_reason_codes_sorted:
        fail(
            "reason_codes mismatch: "
            f"expected {_csv(expected_reason_codes_sorted)}, found {_csv(actual_reason_codes)}"
        )

    expected_final_decision = GO_DECISION if not expected_reason_codes else NO_GO_DECISION
    actual_final_decision = payload["final_decision"]
    if actual_final_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_final_decision != expected_final_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_final_decision}, found {actual_final_decision}"
        )

    print("status=ok")
    print(f"report_file={report_path}")
    print(f"final_decision={actual_final_decision}")
    print(f"failed_checks={_csv(expected_reason_codes_sorted)}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Federated DID handshake deep-lane policy checker"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check = subparsers.add_parser("check")
    check.add_argument("--report-file", required=True)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "check":
        return _check_report(Path(args.report_file))

    fail(f"unsupported command: {args.command}")
    return 1


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

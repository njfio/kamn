#!/usr/bin/env python3
"""Service API reason-code compatibility live policy contracts."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys
import time
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    load_json,
    require_enum,
    write_json,
)

REPORT_SCHEMA = "kamn.runtime.service-api-reason-code-compatibility-live-validation.v1"
POLICY_SCHEMA = "kamn.runtime.service-api-reason-code-compatibility-live-policy-report.v1"
EXPECTED_FAIL_CLOSED_REASON_CODE = "service_api_payload_structure_invalid"

REQUIRED_REPORT_FIELDS = [
    "schema_version",
    "status",
    "final_decision",
    "reason_registry_status",
    "route_error_mapping_status",
    "replay_error_mapping_status",
    "websocket_error_mapping_status",
    "fail_closed_status",
    "performance_budget_status",
    "fail_closed_reason_code",
    "elapsed_seconds",
]

REQUIRED_VERIFIED_FIELDS = [
    "reason_registry_status",
    "route_error_mapping_status",
    "replay_error_mapping_status",
    "websocket_error_mapping_status",
    "fail_closed_status",
    "performance_budget_status",
]


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    if not report_file.is_file():
        fail(f"report file not found: {report_file}")

    report = load_json(report_file)
    missing_fields = [field_name for field_name in REQUIRED_REPORT_FIELDS if field_name not in report]
    if missing_fields:
        fail(f"missing required report fields: {','.join(missing_fields)}")

    expected_final_decision = require_enum(
        "--expected-final-decision",
        args.expected_final_decision,
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    observed_status = report.get("status")
    observed_final_decision = report.get("final_decision")

    decision = DecisionAccumulator()
    decision.reject_if(
        report.get("schema_version") != REPORT_SCHEMA,
        "service_api_reason_code_policy_schema_mismatch",
    )
    decision.reject_if(
        observed_status not in {"pass", "fail"},
        "service_api_reason_code_policy_status_invalid",
    )
    decision.reject_if(
        observed_final_decision not in {"GO", "NO-GO"},
        "service_api_reason_code_policy_final_decision_invalid",
    )
    decision.reject_if(
        observed_final_decision != expected_final_decision,
        "service_api_reason_code_policy_final_decision_mismatch",
    )

    for field_name in REQUIRED_VERIFIED_FIELDS:
        decision.reject_if(
            report.get(field_name) != "verified",
            f"service_api_reason_code_policy_marker_missing:{field_name}",
        )

    decision.reject_if(
        report.get("fail_closed_reason_code") != EXPECTED_FAIL_CLOSED_REASON_CODE,
        "service_api_reason_code_policy_fail_closed_reason_code_mismatch",
    )
    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "service_api_reason_code_policy_elapsed_seconds_invalid",
    )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report: dict[str, Any] = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "service_api_reason_code_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_status": observed_status,
        "observed_final_decision": observed_final_decision,
        "reason_codes": reason_codes,
        "ci_fast_gate": ci_fast_gate,
        "fail_closed_reason_code": report.get("fail_closed_reason_code"),
        "source_report_file": str(report_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, policy_report)

    reason_codes_csv = ",".join(reason_codes)
    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(f"service_api_reason_code_policy_status={policy_status}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"service api reason-code compatibility live policy rejected: {reason_codes_csv}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Service API reason-code compatibility live policy checker contract."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate service API reason-code compatibility live report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to service API reason-code compatibility live validation report JSON.",
    )
    check_policy_parser.add_argument(
        "--expected-final-decision",
        default="GO",
        help="Expected report final decision (GO|NO-GO).",
    )
    check_policy_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        help="CI fast-gate marker (PASS|FAIL).",
    )
    check_policy_parser.add_argument(
        "--output-json",
        help="Optional output path for policy report JSON.",
    )

    args = parser.parse_args()

    try:
        if args.command == "check-policy":
            return _check_policy(args)
    except ContractError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    print(f"unknown command: {args.command}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

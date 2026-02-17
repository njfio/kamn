#!/usr/bin/env python3
"""Service API websocket live policy contracts."""

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

REPORT_SCHEMA = "kamn.runtime.service-api-websocket-live-validation.v1"
POLICY_SCHEMA = "kamn.runtime.service-api-websocket-live-policy-report.v1"
EXPECTED_API_IDLE_TIMEOUT_DEFAULT_MS = 5_000
EXPECTED_WEBSOCKET_LIFECYCLE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1"
)
EXPECTED_WEBSOCKET_LIFECYCLE_REASON_CODES_CSV = (
    "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,"
    "service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,"
    "service_api_ws_key_header_missing"
)

REQUIRED_REPORT_FIELDS = [
    "schema_version",
    "status",
    "final_decision",
    "websocket_upgrade_status",
    "websocket_session_lifecycle_status",
    "websocket_heartbeat_timeout_status",
    "websocket_idle_timeout_contract_status",
    "fail_closed_status",
    "probe_status",
    "websocket_reason_registry_status",
    "websocket_lifecycle_reason_taxonomy_version",
    "websocket_lifecycle_reason_codes_csv",
    "api_idle_timeout_default_ms",
    "elapsed_seconds",
]

REQUIRED_VERIFIED_FIELDS = [
    "websocket_upgrade_status",
    "websocket_session_lifecycle_status",
    "websocket_heartbeat_timeout_status",
    "websocket_idle_timeout_contract_status",
    "fail_closed_status",
    "probe_status",
    "websocket_reason_registry_status",
]


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _is_positive_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value > 0


def _check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    if not report_file.is_file():
        fail(f"report file not found: {report_file}")

    report = load_json(report_file)
    decision = DecisionAccumulator()
    for field_name in REQUIRED_REPORT_FIELDS:
        decision.reject_if(
            field_name not in report,
            f"service_api_websocket_policy_required_field_missing:{field_name}",
        )

    expected_final_decision = require_enum(
        "--expected-final-decision",
        args.expected_final_decision,
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    observed_status = report.get("status")
    observed_final_decision = report.get("final_decision")

    decision.reject_if(
        report.get("schema_version") != REPORT_SCHEMA,
        "service_api_websocket_policy_schema_mismatch",
    )
    decision.reject_if(
        observed_status not in {"pass", "fail"},
        "service_api_websocket_policy_status_invalid",
    )
    decision.reject_if(
        observed_final_decision not in {"GO", "NO-GO"},
        "service_api_websocket_policy_final_decision_invalid",
    )
    decision.reject_if(
        observed_final_decision != expected_final_decision,
        "service_api_websocket_policy_final_decision_mismatch",
    )

    for field_name in REQUIRED_VERIFIED_FIELDS:
        decision.reject_if(
            report.get(field_name) != "verified",
            f"service_api_websocket_policy_marker_missing:{field_name}",
        )

    decision.reject_if(
        not _is_positive_int(report.get("api_idle_timeout_default_ms")),
        "service_api_websocket_policy_api_idle_timeout_default_invalid",
    )
    decision.reject_if(
        report.get("api_idle_timeout_default_ms") != EXPECTED_API_IDLE_TIMEOUT_DEFAULT_MS,
        "service_api_websocket_policy_api_idle_timeout_default_mismatch",
    )
    decision.reject_if(
        report.get("websocket_lifecycle_reason_taxonomy_version")
        != EXPECTED_WEBSOCKET_LIFECYCLE_REASON_TAXONOMY_VERSION,
        "service_api_websocket_policy_websocket_lifecycle_reason_taxonomy_version_mismatch",
    )
    decision.reject_if(
        report.get("websocket_lifecycle_reason_codes_csv")
        != EXPECTED_WEBSOCKET_LIFECYCLE_REASON_CODES_CSV,
        "service_api_websocket_policy_websocket_lifecycle_reason_codes_csv_mismatch",
    )
    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "service_api_websocket_policy_elapsed_seconds_invalid",
    )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"
    reason_codes_value = ",".join(reason_codes)

    policy_report: dict[str, Any] = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "service_api_websocket_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_status": observed_status,
        "observed_final_decision": observed_final_decision,
        "reason_codes": reason_codes,
        "reason_codes_value": reason_codes_value,
        "ci_fast_gate": ci_fast_gate,
        "websocket_lifecycle_reason_taxonomy_version": (
            EXPECTED_WEBSOCKET_LIFECYCLE_REASON_TAXONOMY_VERSION
        ),
        "websocket_lifecycle_reason_codes_csv": (
            EXPECTED_WEBSOCKET_LIFECYCLE_REASON_CODES_CSV
        ),
        "source_report_file": str(report_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, policy_report)

    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(f"service_api_websocket_policy_status={policy_status}")
    print(f"reason_codes={reason_codes_value}")
    print(f"reason_codes_value={reason_codes_value}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"service api websocket live policy rejected: {reason_codes_value}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Service API websocket live policy checker contract."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate service API websocket live report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to service API websocket live validation report JSON.",
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

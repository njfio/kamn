#!/usr/bin/env python3
"""Service API axum ingress live policy contracts."""

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

REPORT_SCHEMA = "kamn.runtime.service-api-axum-ingress-live-validation.v1"
POLICY_SCHEMA = "kamn.runtime.service-api-axum-ingress-live-policy-report.v1"
EXPECTED_FAIL_CLOSED_REASON_CODE = "service_api_axum_oversized_body_rejected"
EXPECTED_BODY_SIZE_LIMIT_BYTES = 64 * 1024
EXPECTED_API_MAX_REQUESTS_DEFAULT = 1
EXPECTED_API_IDLE_TIMEOUT_DEFAULT_MS = 5_000
EXPECTED_API_CONCURRENCY_LIMIT_DEFAULT = 32
EXPECTED_API_RATE_LIMIT_PER_SECOND_DEFAULT = 120
EXPECTED_PROTOCOL_COMPLIANCE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1"
)
EXPECTED_PROTOCOL_COMPLIANCE_REASON_CODES_CSV = (
    "method_path_contract_mismatch,payload_shape_contract_mismatch,"
    "route_contract_bypass_detected"
)
EXPECTED_INGRESS_RESILIENCE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-ingress-resilience-reason-taxonomy.v1"
)
EXPECTED_INGRESS_RESILIENCE_REASON_CODES_CSV = (
    "ingress_readiness_progress_stalled,websocket_upgrade_parity_mismatch,"
    "ci_local_promotion_budget_boundary_exceeded"
)
EXPECTED_REQUEST_VALIDATION_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-request-validation-reason-taxonomy.v1"
)
EXPECTED_REQUEST_VALIDATION_REASON_CODES_CSV = (
    "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,"
    "service_api_method_not_allowed,service_api_route_not_found,"
    "service_api_payload_json_syntax_invalid,"
    "service_api_payload_structure_invalid"
)
EXPECTED_ERROR_ENVELOPE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-error-envelope-reason-taxonomy.v1"
)
EXPECTED_ERROR_ENVELOPE_REASON_CODES_CSV = (
    "service_api_ws_upgrade_header_missing,service_api_method_not_allowed,"
    "service_api_route_not_found"
)

REQUIRED_REPORT_FIELDS = [
    "schema_version",
    "status",
    "final_decision",
    "keep_alive_status",
    "body_size_guard_status",
    "concurrency_status",
    "websocket_status",
    "ingress_limit_config_status",
    "docs_ingress_limit_matrix_status",
    "request_validation_status",
    "error_envelope_field_status",
    "method_path_classification_status",
    "ingress_resilience_gate_status",
    "websocket_upgrade_parity_status",
    "ci_local_promotion_budget_boundary_status",
    "protocol_compliance_status",
    "route_contract_parity_status",
    "protocol_compliance_reason_taxonomy_version",
    "protocol_compliance_reason_codes_csv",
    "ingress_resilience_reason_taxonomy_version",
    "ingress_resilience_reason_codes_csv",
    "request_validation_reason_registry_status",
    "error_envelope_source_contract_status",
    "request_validation_reason_taxonomy_version",
    "request_validation_reason_codes_csv",
    "error_envelope_reason_taxonomy_version",
    "error_envelope_reason_codes_csv",
    "api_max_requests_default",
    "api_idle_timeout_default_ms",
    "body_size_limit_bytes",
    "api_concurrency_limit_default",
    "api_rate_limit_per_second_default",
    "fail_closed_status",
    "ci_fast_gate_exclusion_status",
    "performance_budget_status",
    "fail_closed_reason_code",
    "elapsed_seconds",
]

REQUIRED_VERIFIED_FIELDS = [
    "keep_alive_status",
    "body_size_guard_status",
    "concurrency_status",
    "websocket_status",
    "ingress_limit_config_status",
    "docs_ingress_limit_matrix_status",
    "request_validation_status",
    "error_envelope_field_status",
    "method_path_classification_status",
    "ingress_resilience_gate_status",
    "websocket_upgrade_parity_status",
    "ci_local_promotion_budget_boundary_status",
    "protocol_compliance_status",
    "route_contract_parity_status",
    "request_validation_reason_registry_status",
    "error_envelope_source_contract_status",
    "fail_closed_status",
    "ci_fast_gate_exclusion_status",
    "performance_budget_status",
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
        "service_api_axum_policy_schema_mismatch",
    )
    decision.reject_if(
        observed_status not in {"pass", "fail"},
        "service_api_axum_policy_status_invalid",
    )
    decision.reject_if(
        observed_final_decision not in {"GO", "NO-GO"},
        "service_api_axum_policy_final_decision_invalid",
    )
    decision.reject_if(
        observed_final_decision != expected_final_decision,
        "service_api_axum_policy_final_decision_mismatch",
    )

    for field_name in REQUIRED_VERIFIED_FIELDS:
        decision.reject_if(
            report.get(field_name) != "verified",
            f"service_api_axum_policy_marker_missing:{field_name}",
        )

    decision.reject_if(
        report.get("fail_closed_reason_code") != EXPECTED_FAIL_CLOSED_REASON_CODE,
        "service_api_axum_policy_fail_closed_reason_code_mismatch",
    )
    decision.reject_if(
        not _is_positive_int(report.get("body_size_limit_bytes")),
        "service_api_axum_policy_body_size_limit_invalid",
    )
    decision.reject_if(
        report.get("body_size_limit_bytes") != EXPECTED_BODY_SIZE_LIMIT_BYTES,
        "service_api_axum_policy_body_size_limit_mismatch",
    )
    decision.reject_if(
        not _is_positive_int(report.get("api_max_requests_default")),
        "service_api_axum_policy_api_max_requests_default_invalid",
    )
    decision.reject_if(
        report.get("api_max_requests_default") != EXPECTED_API_MAX_REQUESTS_DEFAULT,
        "service_api_axum_policy_api_max_requests_default_mismatch",
    )
    decision.reject_if(
        not _is_positive_int(report.get("api_idle_timeout_default_ms")),
        "service_api_axum_policy_api_idle_timeout_default_invalid",
    )
    decision.reject_if(
        report.get("api_idle_timeout_default_ms") != EXPECTED_API_IDLE_TIMEOUT_DEFAULT_MS,
        "service_api_axum_policy_api_idle_timeout_default_mismatch",
    )
    decision.reject_if(
        not _is_positive_int(report.get("api_concurrency_limit_default")),
        "service_api_axum_policy_api_concurrency_limit_default_invalid",
    )
    decision.reject_if(
        report.get("api_concurrency_limit_default") != EXPECTED_API_CONCURRENCY_LIMIT_DEFAULT,
        "service_api_axum_policy_api_concurrency_limit_default_mismatch",
    )
    decision.reject_if(
        not _is_positive_int(report.get("api_rate_limit_per_second_default")),
        "service_api_axum_policy_api_rate_limit_per_second_default_invalid",
    )
    decision.reject_if(
        report.get("api_rate_limit_per_second_default")
        != EXPECTED_API_RATE_LIMIT_PER_SECOND_DEFAULT,
        "service_api_axum_policy_api_rate_limit_per_second_default_mismatch",
    )
    decision.reject_if(
        report.get("protocol_compliance_reason_taxonomy_version")
        != EXPECTED_PROTOCOL_COMPLIANCE_REASON_TAXONOMY_VERSION,
        "service_api_axum_policy_protocol_compliance_reason_taxonomy_version_mismatch",
    )
    decision.reject_if(
        report.get("protocol_compliance_reason_codes_csv")
        != EXPECTED_PROTOCOL_COMPLIANCE_REASON_CODES_CSV,
        "service_api_axum_policy_protocol_compliance_reason_codes_csv_mismatch",
    )
    decision.reject_if(
        report.get("ingress_resilience_reason_taxonomy_version")
        != EXPECTED_INGRESS_RESILIENCE_REASON_TAXONOMY_VERSION,
        "service_api_axum_policy_ingress_resilience_reason_taxonomy_version_mismatch",
    )
    decision.reject_if(
        report.get("ingress_resilience_reason_codes_csv")
        != EXPECTED_INGRESS_RESILIENCE_REASON_CODES_CSV,
        "service_api_axum_policy_ingress_resilience_reason_codes_csv_mismatch",
    )
    decision.reject_if(
        report.get("request_validation_reason_taxonomy_version")
        != EXPECTED_REQUEST_VALIDATION_REASON_TAXONOMY_VERSION,
        "service_api_axum_policy_request_validation_reason_taxonomy_version_mismatch",
    )
    decision.reject_if(
        report.get("request_validation_reason_codes_csv")
        != EXPECTED_REQUEST_VALIDATION_REASON_CODES_CSV,
        "service_api_axum_policy_request_validation_reason_codes_csv_mismatch",
    )
    decision.reject_if(
        report.get("error_envelope_reason_taxonomy_version")
        != EXPECTED_ERROR_ENVELOPE_REASON_TAXONOMY_VERSION,
        "service_api_axum_policy_error_envelope_reason_taxonomy_version_mismatch",
    )
    decision.reject_if(
        report.get("error_envelope_reason_codes_csv")
        != EXPECTED_ERROR_ENVELOPE_REASON_CODES_CSV,
        "service_api_axum_policy_error_envelope_reason_codes_csv_mismatch",
    )
    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "service_api_axum_policy_elapsed_seconds_invalid",
    )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report: dict[str, Any] = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "service_api_axum_ingress_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_status": observed_status,
        "observed_final_decision": observed_final_decision,
        "reason_codes": reason_codes,
        "ci_fast_gate": ci_fast_gate,
        "fail_closed_reason_code": report.get("fail_closed_reason_code"),
        "protocol_compliance_reason_taxonomy_version": (
            EXPECTED_PROTOCOL_COMPLIANCE_REASON_TAXONOMY_VERSION
        ),
        "protocol_compliance_reason_codes_csv": (
            EXPECTED_PROTOCOL_COMPLIANCE_REASON_CODES_CSV
        ),
        "ingress_resilience_reason_taxonomy_version": (
            EXPECTED_INGRESS_RESILIENCE_REASON_TAXONOMY_VERSION
        ),
        "ingress_resilience_reason_codes_csv": (
            EXPECTED_INGRESS_RESILIENCE_REASON_CODES_CSV
        ),
        "request_validation_reason_registry_status": (
            report.get("request_validation_reason_registry_status")
        ),
        "error_envelope_source_contract_status": (
            report.get("error_envelope_source_contract_status")
        ),
        "request_validation_reason_taxonomy_version": (
            EXPECTED_REQUEST_VALIDATION_REASON_TAXONOMY_VERSION
        ),
        "request_validation_reason_codes_csv": (
            EXPECTED_REQUEST_VALIDATION_REASON_CODES_CSV
        ),
        "error_envelope_reason_taxonomy_version": (
            EXPECTED_ERROR_ENVELOPE_REASON_TAXONOMY_VERSION
        ),
        "error_envelope_reason_codes_csv": (
            EXPECTED_ERROR_ENVELOPE_REASON_CODES_CSV
        ),
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
    print(f"service_api_axum_ingress_policy_status={policy_status}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"service api axum ingress live policy rejected: {reason_codes_csv}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Service API axum ingress live policy checker contract."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate service API axum ingress live report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to service API axum ingress live validation report JSON.",
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

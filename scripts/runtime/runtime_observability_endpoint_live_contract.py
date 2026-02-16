#!/usr/bin/env python3
"""Runtime observability endpoint live policy contracts."""

from __future__ import annotations

import argparse
import json
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

REPORT_SCHEMA = "kamn.runtime.observability-endpoint-live-validation.v1"
POLICY_SCHEMA = "kamn.runtime.observability-endpoint-live-policy-report.v1"
EXPECTED_FAIL_CLOSED_REASON_CODE = "observability_endpoint_not_found"
EXPECTED_FAIL_CLOSED_REASON_CODES_CSV = (
    "observability_endpoint_not_found,"
    "observability_endpoint_malformed_request,"
    "observability_endpoint_idle_timeout"
)
OBSERVABILITY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.observability-endpoint-reason-taxonomy.v1"
)
OBSERVABILITY_REASON_CODES_CSV = (
    "runtime_observability_endpoint_readiness_progress_stalled,"
    "runtime_observability_stream_parity_bypass_detected,"
    "ci_local_observability_endpoint_budget_boundary_exceeded"
)
OBSERVABILITY_TLS_NEGATIVE_MATRIX_REASON_CODES_CSV = (
    "observability_endpoint_tls_certificate_file_read_failed,"
    "observability_endpoint_tls_key_file_parse_failed,"
    "observability_endpoint_tls_mode_invalid,"
    "observability_endpoint_tls_plain_http_handshake_rejected"
)
CI_LOCAL_OBSERVABILITY_ENDPOINT_BUDGET_MAX_SECONDS = 240

REQUIRED_REPORT_FIELDS = [
    "schema_version",
    "status",
    "final_decision",
    "runtime_observability_stream_contract_status",
    "endpoint_readiness_status",
    "stream_parity_status",
    "unknown_path_contract_status",
    "malformed_input_contract_status",
    "timeout_contract_status",
    "observability_tls_route_contract_status",
    "observability_tls_negative_matrix_status",
    "reason_taxonomy_version",
    "reason_codes_csv",
    "observability_tls_negative_matrix_reason_codes_csv",
    "fail_closed_status",
    "docs_contract_status",
    "fail_closed_reason_code",
    "fail_closed_reason_codes_csv",
    "performance_budget_status",
    "elapsed_seconds",
    "max_seconds",
]

REQUIRED_VERIFIED_FIELDS = [
    "runtime_observability_stream_contract_status",
    "unknown_path_contract_status",
    "malformed_input_contract_status",
    "timeout_contract_status",
    "observability_tls_route_contract_status",
    "observability_tls_negative_matrix_status",
    "fail_closed_status",
    "docs_contract_status",
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
        "runtime_observability_policy_schema_mismatch",
    )
    decision.reject_if(
        observed_status not in {"pass", "fail"},
        "runtime_observability_policy_status_invalid",
    )
    decision.reject_if(
        observed_final_decision not in {"GO", "NO-GO"},
        "runtime_observability_policy_final_decision_invalid",
    )
    decision.reject_if(
        observed_final_decision != expected_final_decision,
        "runtime_observability_policy_final_decision_mismatch",
    )

    for field_name in REQUIRED_VERIFIED_FIELDS:
        decision.reject_if(
            report.get(field_name) != "verified",
            f"runtime_observability_policy_marker_missing:{field_name}",
        )
    decision.reject_if(
        report.get("endpoint_readiness_status") != "verified",
        "runtime_observability_endpoint_readiness_progress_stalled",
    )
    decision.reject_if(
        report.get("stream_parity_status") != "verified",
        "runtime_observability_stream_parity_bypass_detected",
    )
    decision.reject_if(
        report.get("reason_taxonomy_version") != OBSERVABILITY_REASON_TAXONOMY_VERSION,
        "runtime_observability_policy_reason_taxonomy_version_mismatch",
    )
    decision.reject_if(
        report.get("reason_codes_csv") != OBSERVABILITY_REASON_CODES_CSV,
        "runtime_observability_policy_reason_codes_csv_mismatch",
    )
    decision.reject_if(
        report.get("observability_tls_negative_matrix_reason_codes_csv")
        != OBSERVABILITY_TLS_NEGATIVE_MATRIX_REASON_CODES_CSV,
        "runtime_observability_policy_tls_negative_matrix_reason_codes_csv_mismatch",
    )

    decision.reject_if(
        report.get("fail_closed_reason_code") != EXPECTED_FAIL_CLOSED_REASON_CODE,
        "runtime_observability_policy_fail_closed_reason_code_mismatch",
    )
    decision.reject_if(
        report.get("fail_closed_reason_codes_csv") != EXPECTED_FAIL_CLOSED_REASON_CODES_CSV,
        "runtime_observability_policy_fail_closed_reason_codes_csv_mismatch",
    )
    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "runtime_observability_policy_elapsed_seconds_invalid",
    )
    report_max_seconds = report.get("max_seconds")
    decision.reject_if(
        not _is_non_negative_int(report_max_seconds) or report_max_seconds <= 0,
        "runtime_observability_policy_max_seconds_invalid",
    )
    if isinstance(report_max_seconds, int):
        decision.reject_if(
            report_max_seconds > CI_LOCAL_OBSERVABILITY_ENDPOINT_BUDGET_MAX_SECONDS,
            "ci_local_observability_endpoint_budget_boundary_exceeded",
        )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report: dict[str, Any] = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "runtime_observability_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_status": observed_status,
        "observed_final_decision": observed_final_decision,
        "reason_codes": reason_codes,
        "reason_taxonomy_version": OBSERVABILITY_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": OBSERVABILITY_REASON_CODES_CSV,
        "observability_tls_negative_matrix_reason_codes_csv": OBSERVABILITY_TLS_NEGATIVE_MATRIX_REASON_CODES_CSV,
        "ci_fast_gate": ci_fast_gate,
        "fail_closed_reason_code": report.get("fail_closed_reason_code"),
        "fail_closed_reason_codes_csv": report.get("fail_closed_reason_codes_csv"),
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
    print(f"runtime_observability_policy_status={policy_status}")
    print(f"reason_taxonomy_version={OBSERVABILITY_REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={OBSERVABILITY_REASON_CODES_CSV}")
    print(
        "observability_tls_negative_matrix_reason_codes_csv="
        f"{OBSERVABILITY_TLS_NEGATIVE_MATRIX_REASON_CODES_CSV}"
    )
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"runtime observability endpoint live policy rejected: {reason_codes_csv}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Runtime observability endpoint live policy checker contract."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate runtime observability endpoint live report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to runtime observability endpoint live validation report JSON.",
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

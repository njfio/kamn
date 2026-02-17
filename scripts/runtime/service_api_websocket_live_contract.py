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
LANE_REPORT_SCHEMA = "kamn.runtime.service-api-websocket-live-contract-lane-report.v1"
CONVERGENCE_SCHEMA = "kamn.runtime.service-api-websocket-live-convergence-report.v1"
EXPECTED_API_IDLE_TIMEOUT_DEFAULT_MS = 5_000
EXPECTED_WEBSOCKET_LIFECYCLE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1"
)
EXPECTED_WEBSOCKET_LIFECYCLE_REASON_CODES_CSV = (
    "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,"
    "service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,"
    "service_api_ws_key_header_missing"
)
EXPECTED_PROMOTION_DECISION_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-websocket-promotion-decision-reason-taxonomy.v1"
)
EXPECTED_PROMOTION_DECISION_REASON_CODES_CSV = (
    "service_api_websocket_policy_required_field_missing,"
    "service_api_websocket_policy_marker_missing,"
    "service_api_websocket_policy_reason_taxonomy_mismatch,"
    "service_api_websocket_policy_idle_timeout_contract_mismatch,"
    "ci_fast_gate_failed,"
    "service_api_websocket_policy_expected_decision_mismatch,"
    "service_api_websocket_policy_violation"
)
EXPECTED_EVIDENCE_CONVERGENCE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-websocket-evidence-convergence-reason-taxonomy.v1"
)
EXPECTED_EVIDENCE_CONVERGENCE_REASON_CODES_CSV = (
    "service_api_websocket_evidence_link_missing,"
    "service_api_websocket_evidence_payload_tamper_detected,"
    "service_api_websocket_promotion_decision_reason_mapping_mismatch"
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


def _is_non_empty_string_list(value: Any) -> bool:
    return (
        isinstance(value, list)
        and len(value) > 0
        and all(isinstance(item, str) and item for item in value)
    )


def _resolve_promotion_decision_reason_code(
    reason_codes: list[str],
    final_decision: str,
) -> str:
    if final_decision == "GO":
        return "none"

    if any(
        code.startswith("service_api_websocket_policy_required_field_missing:")
        for code in reason_codes
    ):
        return "service_api_websocket_policy_required_field_missing"
    if any(
        code.startswith("service_api_websocket_policy_marker_missing:")
        for code in reason_codes
    ):
        return "service_api_websocket_policy_marker_missing"
    if any(
        code
        in {
            "service_api_websocket_policy_websocket_lifecycle_reason_taxonomy_version_mismatch",
            "service_api_websocket_policy_websocket_lifecycle_reason_codes_csv_mismatch",
        }
        for code in reason_codes
    ):
        return "service_api_websocket_policy_reason_taxonomy_mismatch"
    if any(
        code
        in {
            "service_api_websocket_policy_api_idle_timeout_default_invalid",
            "service_api_websocket_policy_api_idle_timeout_default_mismatch",
        }
        for code in reason_codes
    ):
        return "service_api_websocket_policy_idle_timeout_contract_mismatch"
    if "ci_fast_gate_failed" in reason_codes:
        return "ci_fast_gate_failed"
    if "service_api_websocket_policy_final_decision_mismatch" in reason_codes:
        return "service_api_websocket_policy_expected_decision_mismatch"
    return "service_api_websocket_policy_violation"


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
    promotion_decision_reason_code = _resolve_promotion_decision_reason_code(
        reason_codes,
        final_decision,
    )

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
        "promotion_decision_reason_mapping_status": "verified",
        "promotion_decision_reason_taxonomy_version": (
            EXPECTED_PROMOTION_DECISION_REASON_TAXONOMY_VERSION
        ),
        "promotion_decision_reason_codes_csv": (
            EXPECTED_PROMOTION_DECISION_REASON_CODES_CSV
        ),
        "promotion_decision_reason_code": promotion_decision_reason_code,
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
    print("promotion_decision_reason_mapping_status=verified")
    print(
        "promotion_decision_reason_taxonomy_version="
        + EXPECTED_PROMOTION_DECISION_REASON_TAXONOMY_VERSION
    )
    print(
        "promotion_decision_reason_codes_csv="
        + EXPECTED_PROMOTION_DECISION_REASON_CODES_CSV
    )
    print(f"promotion_decision_reason_code={promotion_decision_reason_code}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"service api websocket live policy rejected: {reason_codes_value}")

    return 0


def _check_evidence_convergence(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    policy_file = Path(args.policy_file).resolve()

    if not report_file.is_file():
        fail(f"report file not found: {report_file}")
    if not policy_file.is_file():
        fail(f"policy file not found: {policy_file}")

    report = load_json(report_file)
    policy = load_json(policy_file)

    decision = DecisionAccumulator()

    decision.reject_if(
        report.get("schema_version") != LANE_REPORT_SCHEMA,
        "service_api_websocket_evidence_payload_tamper_detected:report_schema_version",
    )
    decision.reject_if(
        policy.get("schema_version") != POLICY_SCHEMA,
        "service_api_websocket_evidence_payload_tamper_detected:policy_schema_version",
    )

    report_final_decision = report.get("final_decision")
    policy_final_decision = policy.get("final_decision")
    decision.reject_if(
        report_final_decision not in {"GO", "NO-GO"},
        "service_api_websocket_evidence_payload_tamper_detected:final_decision",
    )
    decision.reject_if(
        policy_final_decision not in {"GO", "NO-GO"},
        "service_api_websocket_evidence_payload_tamper_detected:policy_final_decision",
    )
    decision.reject_if(
        (
            report_final_decision in {"GO", "NO-GO"}
            and policy_final_decision in {"GO", "NO-GO"}
            and report_final_decision != policy_final_decision
        ),
        "service_api_websocket_evidence_payload_tamper_detected:final_decision",
    )
    decision.reject_if(
        report.get("service_api_websocket_policy_status")
        != policy.get("service_api_websocket_policy_status"),
        "service_api_websocket_evidence_payload_tamper_detected:service_api_websocket_policy_status",
    )

    source_report_file = policy.get("source_report_file")
    source_report = None
    source_report_path: Path | None = None
    if not isinstance(source_report_file, str) or source_report_file.strip() == "":
        decision.reject_if(
            True,
            "service_api_websocket_evidence_link_missing:source_report_file",
        )
    else:
        source_report_path = Path(source_report_file).resolve()
        if source_report_path.is_file():
            try:
                source_report = load_json(source_report_path)
            except ContractError:
                decision.reject_if(
                    True,
                    "service_api_websocket_evidence_payload_tamper_detected:source_report_file",
                )

    if source_report is not None:
        decision.reject_if(
            source_report.get("schema_version") != REPORT_SCHEMA,
            "service_api_websocket_evidence_payload_tamper_detected:source_report_schema_version",
        )
        source_report_final_decision = source_report.get("final_decision")
        decision.reject_if(
            source_report_final_decision not in {"GO", "NO-GO"},
            "service_api_websocket_evidence_payload_tamper_detected:source_report_final_decision",
        )
        decision.reject_if(
            (
                source_report_final_decision in {"GO", "NO-GO"}
                and policy_final_decision in {"GO", "NO-GO"}
                and source_report_final_decision != policy_final_decision
            ),
            "service_api_websocket_evidence_payload_tamper_detected:source_report_final_decision",
        )

    policy_reason_codes = policy.get("reason_codes")
    policy_reason_codes_list: list[str] = []
    if _is_non_empty_string_list(policy_reason_codes):
        policy_reason_codes_list = list(policy_reason_codes)
    else:
        decision.reject_if(
            True,
            "service_api_websocket_evidence_payload_tamper_detected:reason_codes",
        )

    observed_reason_codes_value = policy.get("reason_codes_value")
    decision.reject_if(
        not isinstance(observed_reason_codes_value, str),
        "service_api_websocket_evidence_payload_tamper_detected:reason_codes_value",
    )
    if isinstance(observed_reason_codes_value, str) and policy_reason_codes_list:
        decision.reject_if(
            observed_reason_codes_value != ",".join(policy_reason_codes_list),
            "service_api_websocket_evidence_payload_tamper_detected:reason_codes_value",
        )

    if policy_final_decision == "GO" and policy_reason_codes_list:
        decision.reject_if(
            policy_reason_codes_list != ["none"],
            "service_api_websocket_evidence_payload_tamper_detected:reason_codes",
        )
    if policy_final_decision == "NO-GO" and policy_reason_codes_list:
        decision.reject_if(
            "none" in policy_reason_codes_list,
            "service_api_websocket_evidence_payload_tamper_detected:reason_codes",
        )

    expected_reason_code = _resolve_promotion_decision_reason_code(
        policy_reason_codes_list if policy_reason_codes_list else ["none"],
        policy_final_decision if policy_final_decision in {"GO", "NO-GO"} else "NO-GO",
    )

    decision.reject_if(
        policy.get("promotion_decision_reason_mapping_status") != "verified",
        "service_api_websocket_promotion_decision_reason_mapping_mismatch",
    )
    decision.reject_if(
        policy.get("promotion_decision_reason_taxonomy_version")
        != EXPECTED_PROMOTION_DECISION_REASON_TAXONOMY_VERSION,
        "service_api_websocket_promotion_decision_reason_mapping_mismatch",
    )
    decision.reject_if(
        policy.get("promotion_decision_reason_codes_csv")
        != EXPECTED_PROMOTION_DECISION_REASON_CODES_CSV,
        "service_api_websocket_promotion_decision_reason_mapping_mismatch",
    )
    decision.reject_if(
        policy.get("promotion_decision_reason_code") != expected_reason_code,
        "service_api_websocket_promotion_decision_reason_mapping_mismatch",
    )

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    evidence_convergence_status = "verified" if final_decision == "GO" else "failed"
    promotion_decision_reason_mapping_status = (
        "failed"
        if "service_api_websocket_promotion_decision_reason_mapping_mismatch" in reason_codes
        else "verified"
    )
    reason_codes_value = ",".join(reason_codes)

    convergence_report: dict[str, Any] = {
        "schema_version": CONVERGENCE_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "evidence_convergence_status": evidence_convergence_status,
        "promotion_decision_reason_mapping_status": (
            promotion_decision_reason_mapping_status
        ),
        "reason_taxonomy_version": (
            EXPECTED_EVIDENCE_CONVERGENCE_REASON_TAXONOMY_VERSION
        ),
        "reason_codes_csv": EXPECTED_EVIDENCE_CONVERGENCE_REASON_CODES_CSV,
        "reason_codes": reason_codes,
        "reason_codes_value": reason_codes_value,
        "promotion_decision_reason_taxonomy_version": (
            EXPECTED_PROMOTION_DECISION_REASON_TAXONOMY_VERSION
        ),
        "promotion_decision_reason_codes_csv": (
            EXPECTED_PROMOTION_DECISION_REASON_CODES_CSV
        ),
        "promotion_decision_reason_code": expected_reason_code,
        "observed_promotion_decision_reason_code": policy.get(
            "promotion_decision_reason_code"
        ),
        "report_file": str(report_file),
        "policy_file": str(policy_file),
        "source_report_file": (
            str(source_report_path) if source_report_path is not None else ""
        ),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, convergence_report)

    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(f"evidence_convergence_status={evidence_convergence_status}")
    print(
        "promotion_decision_reason_mapping_status="
        + promotion_decision_reason_mapping_status
    )
    print(
        "reason_taxonomy_version="
        + EXPECTED_EVIDENCE_CONVERGENCE_REASON_TAXONOMY_VERSION
    )
    print(f"reason_codes_csv={EXPECTED_EVIDENCE_CONVERGENCE_REASON_CODES_CSV}")
    print(f"reason_codes_value={reason_codes_value}")
    print(
        "promotion_decision_reason_taxonomy_version="
        + EXPECTED_PROMOTION_DECISION_REASON_TAXONOMY_VERSION
    )
    print(
        "promotion_decision_reason_codes_csv="
        + EXPECTED_PROMOTION_DECISION_REASON_CODES_CSV
    )
    print(f"promotion_decision_reason_code={expected_reason_code}")
    if output_json is not None:
        print(f"convergence_report_file={output_json}")

    if final_decision != "GO":
        fail(f"service api websocket evidence convergence rejected: {reason_codes_value}")

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

    check_evidence_parser = subparsers.add_parser(
        "check-evidence-convergence",
        help="Validate websocket evidence convergence across lane and policy artifacts.",
    )
    check_evidence_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to websocket contract-lane report JSON.",
    )
    check_evidence_parser.add_argument(
        "--policy-file",
        required=True,
        help="Path to websocket policy report JSON.",
    )
    check_evidence_parser.add_argument(
        "--output-json",
        help="Optional output path for convergence report JSON.",
    )

    args = parser.parse_args()

    try:
        if args.command == "check-policy":
            return _check_policy(args)
        if args.command == "check-evidence-convergence":
            return _check_evidence_convergence(args)
    except ContractError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    print(f"unknown command: {args.command}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

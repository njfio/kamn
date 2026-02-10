#!/usr/bin/env python3
"""Localhost signed integration evidence policy checker."""

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

    try:
        payload = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"report file is not valid JSON: {exc}")

    required_fields = (
        "schema_version",
        "status",
        "final_decision",
        "contract_key",
        "scenario_fixture_schema_version",
        "scenario_fixture_ids",
        "success_scenario_status",
        "signature_mismatch_scenario_status",
        "malformed_signature_scenario_status",
        "timeout_scenario_status",
        "session_expired_scenario_status",
        "replay_nonce_scenario_status",
        "admission_guards_scenario_status",
        "success_evidence_key",
        "signature_mismatch_evidence_key",
        "malformed_signature_evidence_key",
        "timeout_evidence_key",
        "session_expired_evidence_key",
        "replay_nonce_evidence_key",
        "admission_guards_evidence_key",
        "signature_mismatch_reason_code",
        "malformed_signature_reason_code",
        "timeout_reason_code",
        "session_expired_reason_code",
        "replay_nonce_reason_code",
        "admission_guards_reason_code",
        "success_reason_key",
        "signature_mismatch_reason_key",
        "malformed_signature_reason_key",
        "timeout_reason_key",
        "session_expired_reason_key",
        "replay_nonce_reason_key",
        "admission_guards_reason_key",
        "success_elapsed_seconds",
        "signature_mismatch_elapsed_seconds",
        "malformed_signature_elapsed_seconds",
        "timeout_elapsed_seconds",
        "session_expired_elapsed_seconds",
        "replay_nonce_elapsed_seconds",
        "admission_guards_elapsed_seconds",
        "expiry_guard_status",
        "replay_guard_status",
        "replay_rejected_nonce",
        "admission_guard_status",
        "admission_reason_codes",
    )
    for field_name in required_fields:
        if field_name not in payload:
            fail(f"missing report field: {field_name}")

    if payload["schema_version"] != "kamn.sdk.localhost-signed.integration-contract.v1":
        fail("unexpected schema_version for localhost signed integration contract report")

    status = payload["status"]
    if status not in {"pass", "fail"}:
        fail("status must be pass or fail")
    final_decision = payload["final_decision"]
    if final_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")

    if payload["contract_key"] != "localhost_signed_integration_contract:v1":
        fail("contract_key must be localhost_signed_integration_contract:v1")
    if (
        payload["scenario_fixture_schema_version"]
        != "kamn.sdk.localhost-signed.integration-fixtures.v1"
    ):
        fail(
            "scenario_fixture_schema_version must be "
            "kamn.sdk.localhost-signed.integration-fixtures.v1"
        )
    if payload["scenario_fixture_ids"] != [
        "success-v1",
        "signature-mismatch-v1",
        "timeout-v1",
    ]:
        fail(
            "scenario_fixture_ids must match deterministic fixture ids: "
            "['success-v1', 'signature-mismatch-v1', 'timeout-v1']"
        )

    for field_name in (
        "success_scenario_status",
        "signature_mismatch_scenario_status",
        "malformed_signature_scenario_status",
        "timeout_scenario_status",
        "session_expired_scenario_status",
        "replay_nonce_scenario_status",
        "admission_guards_scenario_status",
    ):
        if payload[field_name] not in {"pass", "fail"}:
            fail(f"{field_name} must be pass or fail")

    if payload["signature_mismatch_reason_code"] != "signature_mismatch_detected":
        fail("signature_mismatch_reason_code must be signature_mismatch_detected")
    if payload["malformed_signature_reason_code"] != "malformed_signature_detected":
        fail("malformed_signature_reason_code must be malformed_signature_detected")

    if payload["timeout_reason_code"] != "listener_timeout_detected":
        fail("timeout_reason_code must be listener_timeout_detected")
    if payload["session_expired_reason_code"] != "session_expired_detected":
        fail("session_expired_reason_code must be session_expired_detected")
    if payload["replay_nonce_reason_code"] != "replay_nonce_detected":
        fail("replay_nonce_reason_code must be replay_nonce_detected")
    if payload["admission_guards_reason_code"] != "session_admission_guards_detected":
        fail(
            "admission_guards_reason_code must be "
            "session_admission_guards_detected"
        )

    if payload["success_evidence_key"] != "localhost_signed_integration:success:v1":
        fail("success_evidence_key must be localhost_signed_integration:success:v1")

    if (
        payload["signature_mismatch_evidence_key"]
        != "localhost_signed_integration:signature-mismatch:v1"
    ):
        fail(
            "signature_mismatch_evidence_key must be "
            "localhost_signed_integration:signature-mismatch:v1"
        )
    if (
        payload["malformed_signature_evidence_key"]
        != "localhost_signed_integration:malformed-signature:v1"
    ):
        fail(
            "malformed_signature_evidence_key must be "
            "localhost_signed_integration:malformed-signature:v1"
        )

    if payload["timeout_evidence_key"] != "localhost_signed_integration:timeout:v1":
        fail("timeout_evidence_key must be localhost_signed_integration:timeout:v1")
    if (
        payload["session_expired_evidence_key"]
        != "localhost_signed_integration:session-expired:v1"
    ):
        fail(
            "session_expired_evidence_key must be "
            "localhost_signed_integration:session-expired:v1"
        )
    if payload["replay_nonce_evidence_key"] != "localhost_signed_integration:replay-nonce:v1":
        fail("replay_nonce_evidence_key must be localhost_signed_integration:replay-nonce:v1")
    if (
        payload["admission_guards_evidence_key"]
        != "localhost_signed_integration:admission-guards:v1"
    ):
        fail(
            "admission_guards_evidence_key must be "
            "localhost_signed_integration:admission-guards:v1"
        )

    if payload["success_reason_key"] != "localhost_signed_integration_reason:none:v1":
        fail("success_reason_key must be localhost_signed_integration_reason:none:v1")

    if (
        payload["signature_mismatch_reason_key"]
        != "localhost_signed_integration_reason:signature_mismatch_detected:v1"
    ):
        fail(
            "signature_mismatch_reason_key must be "
            "localhost_signed_integration_reason:signature_mismatch_detected:v1"
        )
    if (
        payload["malformed_signature_reason_key"]
        != "localhost_signed_integration_reason:malformed_signature_detected:v1"
    ):
        fail(
            "malformed_signature_reason_key must be "
            "localhost_signed_integration_reason:malformed_signature_detected:v1"
        )

    if (
        payload["timeout_reason_key"]
        != "localhost_signed_integration_reason:listener_timeout_detected:v1"
    ):
        fail(
            "timeout_reason_key must be "
            "localhost_signed_integration_reason:listener_timeout_detected:v1"
        )
    if (
        payload["session_expired_reason_key"]
        != "localhost_signed_integration_reason:session_expired_detected:v1"
    ):
        fail(
            "session_expired_reason_key must be "
            "localhost_signed_integration_reason:session_expired_detected:v1"
        )
    if (
        payload["replay_nonce_reason_key"]
        != "localhost_signed_integration_reason:replay_nonce_detected:v1"
    ):
        fail(
            "replay_nonce_reason_key must be "
            "localhost_signed_integration_reason:replay_nonce_detected:v1"
        )
    if (
        payload["admission_guards_reason_key"]
        != "localhost_signed_integration_reason:session_admission_guards_detected:v1"
    ):
        fail(
            "admission_guards_reason_key must be "
            "localhost_signed_integration_reason:session_admission_guards_detected:v1"
        )

    for field_name in (
        "success_elapsed_seconds",
        "signature_mismatch_elapsed_seconds",
        "malformed_signature_elapsed_seconds",
        "timeout_elapsed_seconds",
        "session_expired_elapsed_seconds",
        "replay_nonce_elapsed_seconds",
        "admission_guards_elapsed_seconds",
    ):
        value = payload[field_name]
        if not isinstance(value, int):
            fail(f"{field_name} must be an integer")
        if value < 0:
            fail(f"{field_name} must be non-negative")

    if payload["expiry_guard_status"] != "pass":
        fail("expiry_guard_status must be pass")
    if payload["replay_guard_status"] != "pass":
        fail("replay_guard_status must be pass")
    replay_rejected_nonce = payload["replay_rejected_nonce"]
    if not isinstance(replay_rejected_nonce, int) or replay_rejected_nonce <= 0:
        fail("replay_rejected_nonce must be a positive integer")

    if payload["admission_guard_status"] != "pass":
        fail("admission_guard_status must be pass")
    expected_admission_reason_codes = [
        "stale_session_detected",
        "unauthorized_sender_detected",
        "malformed_payload_detected",
    ]
    if payload["admission_reason_codes"] != expected_admission_reason_codes:
        fail(
            "admission_reason_codes must match deterministic sequence: "
            f"{expected_admission_reason_codes}"
        )

    expected_status = "pass"
    if payload["success_scenario_status"] != "pass":
        expected_status = "fail"
    if payload["signature_mismatch_scenario_status"] != "pass":
        expected_status = "fail"
    if payload["malformed_signature_scenario_status"] != "pass":
        expected_status = "fail"
    if payload["timeout_scenario_status"] != "pass":
        expected_status = "fail"
    if payload["session_expired_scenario_status"] != "pass":
        expected_status = "fail"
    if payload["replay_nonce_scenario_status"] != "pass":
        expected_status = "fail"
    if payload["admission_guards_scenario_status"] != "pass":
        expected_status = "fail"

    if status != expected_status:
        fail(
            "policy status mismatch: "
            f"expected status={expected_status}, found {status}"
        )
    expected_decision = "GO" if expected_status == "pass" else "NO-GO"
    if final_decision != expected_decision:
        fail(
            "final_decision mismatch: "
            f"expected final_decision={expected_decision}, found {final_decision}"
        )

    print("status=ok")
    print(f"report_file={report_path}")
    print(f"final_decision={final_decision}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Localhost signed integration evidence policy checker."
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

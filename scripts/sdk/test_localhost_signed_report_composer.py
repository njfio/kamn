#!/usr/bin/env python3
"""Unit tests for localhost signed report composition helpers."""

from __future__ import annotations

import unittest
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError
from localhost_signed_report_composer import (
    compose_localhost_signed_demo_contract_report,
    compose_localhost_signed_integration_contract_report,
)


class LocalhostSignedReportComposerTests(unittest.TestCase):
    def test_compose_demo_contract_report_emits_expected_payload(self) -> None:
        report = compose_localhost_signed_demo_contract_report(
            demo_artifact={
                "schema_version": "kamn.sdk.localhost-signed.demo-receipt-artifact.v1",
                "status": "pass",
            },
            integration_report={
                "schema_version": "kamn.sdk.localhost-signed.integration-contract.v1",
                "status": "pass",
            },
            elapsed_seconds=2,
            max_seconds=180,
        )
        self.assertEqual(report["schema_version"], "kamn.sdk.localhost-signed.demo-contract.v1")
        self.assertEqual(report["status"], "pass")
        self.assertEqual(report["reason_codes"], ["none"])
        self.assertEqual(report["budget_status"], "within_budget")
        self.assertEqual(report["elapsed_seconds"], 2)
        self.assertEqual(report["max_seconds"], 180)

    def test_compose_demo_contract_report_rejects_schema_drift(self) -> None:
        with self.assertRaises(ContractError):
            compose_localhost_signed_demo_contract_report(
                demo_artifact={"schema_version": "bad.schema.v1", "status": "pass"},
                integration_report={
                    "schema_version": "kamn.sdk.localhost-signed.integration-contract.v1",
                    "status": "pass",
                },
                elapsed_seconds=1,
                max_seconds=60,
            )

    def test_compose_integration_contract_report_maps_expected_fields(self) -> None:
        scenario_ids = ["success-v1", "signature-mismatch-v1", "timeout-v1"]
        success_payload = {
            "status": "pass",
            "evidence_key": "localhost_signed_integration:success:v1",
            "reason_key": "localhost_signed_integration_reason:none:v1",
            "elapsed_seconds": 1,
        }
        signature_payload = {
            "status": "pass",
            "evidence_key": "localhost_signed_integration:signature-mismatch:v1",
            "reason_code": "signature_mismatch_detected",
            "reason_key": "localhost_signed_integration_reason:signature_mismatch_detected:v1",
            "elapsed_seconds": 1,
        }
        malformed_signature_payload = {
            "status": "pass",
            "evidence_key": "localhost_signed_integration:malformed-signature:v1",
            "reason_code": "malformed_signature_detected",
            "reason_key": "localhost_signed_integration_reason:malformed_signature_detected:v1",
            "elapsed_seconds": 1,
        }
        timeout_payload = {
            "status": "pass",
            "evidence_key": "localhost_signed_integration:timeout:v1",
            "reason_code": "listener_timeout_detected",
            "reason_key": "localhost_signed_integration_reason:listener_timeout_detected:v1",
            "elapsed_seconds": 1,
        }
        session_expired_payload = {
            "status": "pass",
            "evidence_key": "localhost_signed_integration:session-expired:v1",
            "reason_code": "session_expired_detected",
            "reason_key": "localhost_signed_integration_reason:session_expired_detected:v1",
            "elapsed_seconds": 1,
            "expiry_guard_status": "pass",
        }
        replay_payload = {
            "status": "pass",
            "evidence_key": "localhost_signed_integration:replay-nonce:v1",
            "reason_code": "replay_nonce_detected",
            "reason_key": "localhost_signed_integration_reason:replay_nonce_detected:v1",
            "elapsed_seconds": 1,
            "replay_guard_status": "pass",
            "replay_rejected_nonce": 7,
        }
        admission_payload = {
            "status": "pass",
            "evidence_key": "localhost_signed_integration:admission-guards:v1",
            "reason_code": "session_admission_guards_detected",
            "reason_key": "localhost_signed_integration_reason:session_admission_guards_detected:v1",
            "elapsed_seconds": 1,
            "admission_guard_status": "pass",
            "admission_reason_codes": [
                "stale_session_detected",
                "unauthorized_sender_detected",
                "malformed_payload_detected",
            ],
        }

        report = compose_localhost_signed_integration_contract_report(
            fixture_schema_version="kamn.sdk.localhost-signed.integration-fixtures.v1",
            scenario_fixture_ids=scenario_ids,
            success_payload=success_payload,
            signature_payload=signature_payload,
            malformed_signature_payload=malformed_signature_payload,
            timeout_payload=timeout_payload,
            session_expired_payload=session_expired_payload,
            replay_payload=replay_payload,
            admission_payload=admission_payload,
        )

        self.assertEqual(
            report["schema_version"], "kamn.sdk.localhost-signed.integration-contract.v1"
        )
        self.assertEqual(report["final_decision"], "GO")
        self.assertEqual(report["contract_key"], "localhost_signed_integration_contract:v1")
        self.assertEqual(report["scenario_fixture_ids"], scenario_ids)
        self.assertEqual(
            report["signature_mismatch_reason_code"], "signature_mismatch_detected"
        )
        self.assertEqual(report["replay_rejected_nonce"], 7)
        self.assertEqual(
            report["admission_reason_codes"],
            [
                "stale_session_detected",
                "unauthorized_sender_detected",
                "malformed_payload_detected",
            ],
        )

    def test_compose_integration_contract_report_rejects_missing_reason_key(self) -> None:
        with self.assertRaises(ContractError):
            compose_localhost_signed_integration_contract_report(
                fixture_schema_version="kamn.sdk.localhost-signed.integration-fixtures.v1",
                scenario_fixture_ids=["success-v1", "signature-mismatch-v1", "timeout-v1"],
                success_payload={
                    "status": "pass",
                    "evidence_key": "localhost_signed_integration:success:v1",
                    "elapsed_seconds": 1,
                },
                signature_payload={
                    "status": "pass",
                    "evidence_key": "localhost_signed_integration:signature-mismatch:v1",
                    "reason_code": "signature_mismatch_detected",
                    "reason_key": "localhost_signed_integration_reason:signature_mismatch_detected:v1",
                    "elapsed_seconds": 1,
                },
                malformed_signature_payload={
                    "status": "pass",
                    "evidence_key": "localhost_signed_integration:malformed-signature:v1",
                    "reason_code": "malformed_signature_detected",
                    "reason_key": "localhost_signed_integration_reason:malformed_signature_detected:v1",
                    "elapsed_seconds": 1,
                },
                timeout_payload={
                    "status": "pass",
                    "evidence_key": "localhost_signed_integration:timeout:v1",
                    "reason_code": "listener_timeout_detected",
                    "reason_key": "localhost_signed_integration_reason:listener_timeout_detected:v1",
                    "elapsed_seconds": 1,
                },
                session_expired_payload={
                    "status": "pass",
                    "evidence_key": "localhost_signed_integration:session-expired:v1",
                    "reason_code": "session_expired_detected",
                    "reason_key": "localhost_signed_integration_reason:session_expired_detected:v1",
                    "elapsed_seconds": 1,
                    "expiry_guard_status": "pass",
                },
                replay_payload={
                    "status": "pass",
                    "evidence_key": "localhost_signed_integration:replay-nonce:v1",
                    "reason_code": "replay_nonce_detected",
                    "reason_key": "localhost_signed_integration_reason:replay_nonce_detected:v1",
                    "elapsed_seconds": 1,
                    "replay_guard_status": "pass",
                    "replay_rejected_nonce": 7,
                },
                admission_payload={
                    "status": "pass",
                    "evidence_key": "localhost_signed_integration:admission-guards:v1",
                    "reason_code": "session_admission_guards_detected",
                    "reason_key": "localhost_signed_integration_reason:session_admission_guards_detected:v1",
                    "elapsed_seconds": 1,
                    "admission_guard_status": "pass",
                    "admission_reason_codes": [
                        "stale_session_detected",
                        "unauthorized_sender_detected",
                        "malformed_payload_detected",
                    ],
                },
            )


if __name__ == "__main__":
    unittest.main()

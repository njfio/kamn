#!/usr/bin/env python3
"""Shared localhost_signed_report_composer helpers for SDK contract lane reports."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json  # noqa: E402

DEMO_ARTIFACT_SCHEMA = "kamn.sdk.localhost-signed.demo-receipt-artifact.v1"
INTEGRATION_REPORT_SCHEMA = "kamn.sdk.localhost-signed.integration-contract.v1"
DEMO_CONTRACT_SCHEMA = "kamn.sdk.localhost-signed.demo-contract.v1"
INTEGRATION_CONTRACT_SCHEMA = "kamn.sdk.localhost-signed.integration-contract.v1"


def _require_string(payload: dict[str, Any], field: str, context: str) -> str:
    value = payload.get(field)
    if not isinstance(value, str) or not value:
        fail(f"{context}.{field} must be a non-empty string")
    return value


def _require_int(payload: dict[str, Any], field: str, context: str) -> int:
    value = payload.get(field)
    if not isinstance(value, int):
        fail(f"{context}.{field} must be an integer")
    return value


def _require_string_list(payload: dict[str, Any], field: str, context: str) -> list[str]:
    value = payload.get(field)
    if not isinstance(value, list) or not all(isinstance(item, str) and item for item in value):
        fail(f"{context}.{field} must be a list of non-empty strings")
    return value


def compose_localhost_signed_demo_contract_report(
    *,
    demo_artifact: dict[str, Any],
    integration_report: dict[str, Any],
    elapsed_seconds: int,
    max_seconds: int,
) -> dict[str, Any]:
    """Compose stable localhost signed demo contract lane report payload."""
    if demo_artifact.get("schema_version") != DEMO_ARTIFACT_SCHEMA:
        fail("unexpected localhost signed demo artifact schema")
    if demo_artifact.get("status") != "pass":
        fail("expected localhost signed demo artifact status=pass")

    if integration_report.get("schema_version") != INTEGRATION_REPORT_SCHEMA:
        fail("unexpected localhost signed integration report schema")
    if integration_report.get("status") != "pass":
        fail("expected localhost signed integration report status=pass")

    if elapsed_seconds < 0:
        fail("elapsed_seconds must be non-negative")
    if max_seconds <= 0:
        fail("max_seconds must be positive")
    if elapsed_seconds > max_seconds:
        fail("elapsed_seconds exceeds max_seconds")

    return {
        "schema_version": DEMO_CONTRACT_SCHEMA,
        "status": "pass",
        "suite": "localhost_signed_demo_contract_lane",
        "demo_artifact_schema": DEMO_ARTIFACT_SCHEMA,
        "integration_report_schema": INTEGRATION_REPORT_SCHEMA,
        "demo_status": "pass",
        "integration_status": "pass",
        "demo_success_marker": "localhost signed message demo completed.",
        "integration_success_marker": "localhost signed integration contract lane tests passed.",
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "budget_status": "within_budget",
        "reason_codes": ["none"],
    }


def compose_localhost_signed_integration_contract_report(
    *,
    fixture_schema_version: str,
    scenario_fixture_ids: list[str],
    success_payload: dict[str, Any],
    signature_payload: dict[str, Any],
    malformed_signature_payload: dict[str, Any],
    timeout_payload: dict[str, Any],
    session_expired_payload: dict[str, Any],
    replay_payload: dict[str, Any],
    admission_payload: dict[str, Any],
) -> dict[str, Any]:
    """Compose stable localhost signed integration contract lane report payload."""
    if not isinstance(fixture_schema_version, str) or not fixture_schema_version:
        fail("fixture_schema_version must be a non-empty string")
    if not isinstance(scenario_fixture_ids, list) or not all(
        isinstance(item, str) and item for item in scenario_fixture_ids
    ):
        fail("scenario_fixture_ids must be a list of non-empty strings")

    for context, payload in (
        ("success_payload", success_payload),
        ("signature_payload", signature_payload),
        ("malformed_signature_payload", malformed_signature_payload),
        ("timeout_payload", timeout_payload),
        ("session_expired_payload", session_expired_payload),
        ("replay_payload", replay_payload),
        ("admission_payload", admission_payload),
    ):
        if not isinstance(payload, dict):
            fail(f"{context} must be an object")
        _require_string(payload, "status", context)
        _require_string(payload, "evidence_key", context)
        _require_string(payload, "reason_key", context)
        _require_int(payload, "elapsed_seconds", context)

    _require_string(signature_payload, "reason_code", "signature_payload")
    _require_string(malformed_signature_payload, "reason_code", "malformed_signature_payload")
    _require_string(timeout_payload, "reason_code", "timeout_payload")
    _require_string(session_expired_payload, "reason_code", "session_expired_payload")
    _require_string(replay_payload, "reason_code", "replay_payload")
    _require_string(admission_payload, "reason_code", "admission_payload")
    _require_string(session_expired_payload, "expiry_guard_status", "session_expired_payload")
    _require_string(replay_payload, "replay_guard_status", "replay_payload")
    replay_rejected_nonce = _require_int(replay_payload, "replay_rejected_nonce", "replay_payload")
    _require_string(admission_payload, "admission_guard_status", "admission_payload")
    admission_reason_codes = _require_string_list(
        admission_payload,
        "admission_reason_codes",
        "admission_payload",
    )

    return {
        "schema_version": INTEGRATION_CONTRACT_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "contract_key": "localhost_signed_integration_contract:v1",
        "scenario_fixture_schema_version": fixture_schema_version,
        "scenario_fixture_ids": scenario_fixture_ids,
        "success_scenario_status": success_payload["status"],
        "signature_mismatch_scenario_status": signature_payload["status"],
        "malformed_signature_scenario_status": malformed_signature_payload["status"],
        "timeout_scenario_status": timeout_payload["status"],
        "session_expired_scenario_status": session_expired_payload["status"],
        "replay_nonce_scenario_status": replay_payload["status"],
        "admission_guards_scenario_status": admission_payload["status"],
        "success_evidence_key": success_payload["evidence_key"],
        "signature_mismatch_evidence_key": signature_payload["evidence_key"],
        "malformed_signature_evidence_key": malformed_signature_payload["evidence_key"],
        "timeout_evidence_key": timeout_payload["evidence_key"],
        "session_expired_evidence_key": session_expired_payload["evidence_key"],
        "replay_nonce_evidence_key": replay_payload["evidence_key"],
        "admission_guards_evidence_key": admission_payload["evidence_key"],
        "signature_mismatch_reason_code": signature_payload["reason_code"],
        "malformed_signature_reason_code": malformed_signature_payload["reason_code"],
        "timeout_reason_code": timeout_payload["reason_code"],
        "session_expired_reason_code": session_expired_payload["reason_code"],
        "replay_nonce_reason_code": replay_payload["reason_code"],
        "admission_guards_reason_code": admission_payload["reason_code"],
        "success_reason_key": success_payload["reason_key"],
        "signature_mismatch_reason_key": signature_payload["reason_key"],
        "malformed_signature_reason_key": malformed_signature_payload["reason_key"],
        "timeout_reason_key": timeout_payload["reason_key"],
        "session_expired_reason_key": session_expired_payload["reason_key"],
        "replay_nonce_reason_key": replay_payload["reason_key"],
        "admission_guards_reason_key": admission_payload["reason_key"],
        "success_elapsed_seconds": success_payload["elapsed_seconds"],
        "signature_mismatch_elapsed_seconds": signature_payload["elapsed_seconds"],
        "malformed_signature_elapsed_seconds": malformed_signature_payload["elapsed_seconds"],
        "timeout_elapsed_seconds": timeout_payload["elapsed_seconds"],
        "session_expired_elapsed_seconds": session_expired_payload["elapsed_seconds"],
        "replay_nonce_elapsed_seconds": replay_payload["elapsed_seconds"],
        "admission_guards_elapsed_seconds": admission_payload["elapsed_seconds"],
        "expiry_guard_status": session_expired_payload["expiry_guard_status"],
        "replay_guard_status": replay_payload["replay_guard_status"],
        "replay_rejected_nonce": replay_rejected_nonce,
        "admission_guard_status": admission_payload["admission_guard_status"],
        "admission_reason_codes": admission_reason_codes,
    }


def _handle_compose_demo(args: argparse.Namespace) -> int:
    if args.elapsed_seconds < 0:
        fail("--elapsed-seconds must be non-negative")
    if args.max_seconds <= 0:
        fail("--max-seconds must be positive")

    demo_artifact = load_json(Path(args.demo_artifact))
    integration_report = load_json(Path(args.integration_report))
    summary = compose_localhost_signed_demo_contract_report(
        demo_artifact=demo_artifact,
        integration_report=integration_report,
        elapsed_seconds=args.elapsed_seconds,
        max_seconds=args.max_seconds,
    )

    output_path = Path(args.output_json)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(summary, separators=(",", ":")), encoding="utf-8")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Compose localhost signed SDK contract lane reports."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    compose_demo = subparsers.add_parser(
        "compose-demo",
        help="Compose localhost signed demo contract lane summary report.",
    )
    compose_demo.add_argument("--demo-artifact", required=True)
    compose_demo.add_argument("--integration-report", required=True)
    compose_demo.add_argument("--output-json", required=True)
    compose_demo.add_argument("--elapsed-seconds", required=True, type=int)
    compose_demo.add_argument("--max-seconds", required=True, type=int)
    compose_demo.set_defaults(handler=_handle_compose_demo)

    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    handler = getattr(args, "handler", None)
    if handler is None:
        fail("no command selected")
    return int(handler(args))


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

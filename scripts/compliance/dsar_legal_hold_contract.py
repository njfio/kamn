#!/usr/bin/env python3
"""DSAR legal-hold evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any, Mapping

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_keys,
    require_object,
    write_json,
)

SCHEMA_VERSION = "kamn.compliance.dsar-legal-hold.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_bool(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("boolean fields must be true or false")


def _parse_request_type(raw_value: str) -> str:
    if raw_value in {"ACCESS", "EXPORT", "ERASURE"}:
        return raw_value
    fail("request-type must be ACCESS, EXPORT, or ERASURE")


def _parse_gate_status(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("tamper-check and ci-fast-gate must be PASS or FAIL")


def _compute_policy_checks(
    *, request_type: str, legal_hold_active: bool, retention_expired: bool
) -> Mapping[str, bool]:
    legal_hold_blocks_erasure = request_type == "ERASURE" and legal_hold_active
    retention_allows_erasure = request_type != "ERASURE" or retention_expired
    return {
        "legal_hold_blocks_erasure": legal_hold_blocks_erasure,
        "retention_allows_erasure": retention_allows_erasure,
    }


def _compute_reason_codes(
    *,
    tamper_check: str,
    ci_fast_gate: str,
    evidence_complete: bool,
    approval_recorded: bool,
    legal_hold_blocks_erasure: bool,
    retention_allows_erasure: bool,
) -> list[str]:
    reason_codes: list[str] = []
    if tamper_check != "PASS":
        reason_codes.append("tamper_check_failed")
    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")
    if not evidence_complete:
        reason_codes.append("evidence_incomplete")
    if not approval_recorded:
        reason_codes.append("approval_missing")
    if legal_hold_blocks_erasure:
        reason_codes.append("legal_hold_precedence_block")
    if not retention_allows_erasure:
        reason_codes.append("retention_window_not_expired")
    return reason_codes


def generate_bundle(args: argparse.Namespace) -> int:
    request_type = _parse_request_type(args.request_type)
    legal_hold_active = _parse_bool(args.legal_hold_active)
    retention_expired = _parse_bool(args.retention_expired)
    evidence_complete = _parse_bool(args.evidence_complete)
    approval_recorded = _parse_bool(args.approval_recorded)
    tamper_check = _parse_gate_status(args.tamper_check)
    ci_fast_gate = _parse_gate_status(args.ci_fast_gate)

    policy_checks = _compute_policy_checks(
        request_type=request_type,
        legal_hold_active=legal_hold_active,
        retention_expired=retention_expired,
    )
    legal_hold_blocks_erasure = policy_checks["legal_hold_blocks_erasure"]
    retention_allows_erasure = policy_checks["retention_allows_erasure"]

    is_go = (
        tamper_check == "PASS"
        and ci_fast_gate == "PASS"
        and evidence_complete
        and approval_recorded
        and not legal_hold_blocks_erasure
        and retention_allows_erasure
    )
    final_decision = GO_DECISION if is_go else NO_GO_DECISION

    reason_codes = _compute_reason_codes(
        tamper_check=tamper_check,
        ci_fast_gate=ci_fast_gate,
        evidence_complete=evidence_complete,
        approval_recorded=approval_recorded,
        legal_hold_blocks_erasure=legal_hold_blocks_erasure,
        retention_allows_erasure=retention_allows_erasure,
    )

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "request_id": args.request_id,
        "subject_did": args.subject_did,
        "request_type": request_type,
        "legal_hold_active": legal_hold_active,
        "retention_expired": retention_expired,
        "evidence_complete": evidence_complete,
        "approval_recorded": approval_recorded,
        "tamper_check": tamper_check,
        "ci_fast_gate": ci_fast_gate,
        "policy_checks": policy_checks,
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    return 0


def _require_bool(payload: Mapping[str, Any], field_name: str) -> bool:
    value = payload.get(field_name)
    if not isinstance(value, bool):
        fail(f"{field_name} must be boolean")
    return value


def check_bundle(args: argparse.Namespace) -> int:
    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        fail(f"bundle file not found: {bundle_path}")

    payload = load_json(bundle_path)
    require_keys(
        payload,
        (
            "schema_version",
            "generated_at",
            "request_id",
            "subject_did",
            "request_type",
            "legal_hold_active",
            "retention_expired",
            "evidence_complete",
            "approval_recorded",
            "tamper_check",
            "ci_fast_gate",
            "policy_checks",
            "reason_codes",
            "final_decision",
        ),
    )

    request_type = payload.get("request_type")
    if request_type not in {"ACCESS", "EXPORT", "ERASURE"}:
        fail("request_type must be ACCESS, EXPORT, or ERASURE")

    legal_hold_active = _require_bool(payload, "legal_hold_active")
    retention_expired = _require_bool(payload, "retention_expired")
    evidence_complete = _require_bool(payload, "evidence_complete")
    approval_recorded = _require_bool(payload, "approval_recorded")

    tamper_check = payload.get("tamper_check")
    if tamper_check not in {"PASS", "FAIL"}:
        fail("tamper_check must be PASS or FAIL")

    ci_fast_gate = payload.get("ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    policy_checks = require_object(payload, "policy_checks")
    if "legal_hold_blocks_erasure" not in policy_checks:
        fail("missing policy_checks field: legal_hold_blocks_erasure")
    if "retention_allows_erasure" not in policy_checks:
        fail("missing policy_checks field: retention_allows_erasure")

    for field_name in ("legal_hold_blocks_erasure", "retention_allows_erasure"):
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

    derived_checks = _compute_policy_checks(
        request_type=request_type,
        legal_hold_active=legal_hold_active,
        retention_expired=retention_expired,
    )
    legal_hold_blocks_erasure = derived_checks["legal_hold_blocks_erasure"]
    retention_allows_erasure = derived_checks["retention_allows_erasure"]

    if policy_checks["legal_hold_blocks_erasure"] != legal_hold_blocks_erasure:
        fail("policy_checks.legal_hold_blocks_erasure does not match derived policy")
    if policy_checks["retention_allows_erasure"] != retention_allows_erasure:
        fail("policy_checks.retention_allows_erasure does not match derived policy")

    expected_go = (
        tamper_check == "PASS"
        and ci_fast_gate == "PASS"
        and evidence_complete
        and approval_recorded
        and not legal_hold_blocks_erasure
        and retention_allows_erasure
    )
    expected_decision = GO_DECISION if expected_go else NO_GO_DECISION

    actual_decision = payload.get("final_decision")
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    failed_checks = _compute_reason_codes(
        tamper_check=tamper_check,
        ci_fast_gate=ci_fast_gate,
        evidence_complete=evidence_complete,
        approval_recorded=approval_recorded,
        legal_hold_blocks_erasure=legal_hold_blocks_erasure,
        retention_allows_erasure=retention_allows_erasure,
    )
    failed_checks_value = ",".join(failed_checks) if failed_checks else "none"

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "DSAR legal-hold evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--request-id", required=True)
    generate.add_argument("--subject-did", required=True)
    generate.add_argument("--request-type", required=True)
    generate.add_argument("--legal-hold-active", required=True)
    generate.add_argument("--retention-expired", required=True)
    generate.add_argument("--evidence-complete", required=True)
    generate.add_argument("--approval-recorded", required=True)
    generate.add_argument("--tamper-check", required=True)
    generate.add_argument("--ci-fast-gate", required=True)
    generate.set_defaults(handler=generate_bundle)

    check = subparsers.add_parser("check")
    check.add_argument("--bundle-file", required=True)
    check.set_defaults(handler=check_bundle)

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

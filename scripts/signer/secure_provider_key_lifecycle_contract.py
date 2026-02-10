#!/usr/bin/env python3
"""Secure-provider signer key-lifecycle evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import re
import sys
from typing import Mapping

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    load_json,
    require_enum,
    require_int,
    require_keys,
    require_non_negative_int,
    require_object,
    require_pattern,
    require_string,
    write_json,
)

SCHEMA_VERSION = "kamn.signer.secure-provider-key-lifecycle.v1"
SUPPORTED_PROVIDER = "aws-kms"
ALLOWED_KEY_ROLES = ("operator", "admin", "treasury", "auditor")
ALLOWED_LIFECYCLE_ACTIONS = ("rotate", "revoke")
ALLOWED_REVOCATION_REASON_CODES = (
    "compromised-key",
    "operator-requested",
    "policy-violation",
    "incident-response",
)


def _is_valid_sha256_digest(value: str) -> bool:
    return value.startswith("sha256:") and len(value) > len("sha256:")


def _is_valid_incident_ticket(value: str) -> bool:
    return re.fullmatch(r"INC-[0-9]{4,}", value or "") is not None


def _compute_policy_checks(
    secure_key_reference: str,
    provider: str,
    key_role: str,
    lifecycle_action: str,
    previous_version: int,
    target_version: int,
    incident_ticket: str,
    revocation_reason_code: str,
    required_approvals: int,
    received_approvals: int,
    custody_attestation_hash: str,
    approval_quorum_hash: str,
    ci_fast_gate: str,
) -> Mapping[str, bool]:
    rotate_transition_valid = (
        lifecycle_action == "rotate"
        and previous_version >= 1
        and target_version == previous_version + 1
    )
    revoke_transition_valid = (
        lifecycle_action == "revoke"
        and previous_version >= 1
        and target_version == previous_version
    )

    return {
        "provider_supported": provider == SUPPORTED_PROVIDER,
        "key_reference_scoped": secure_key_reference.startswith(
            f"secure:{provider}:role-{key_role}/"
        ),
        "role_supported": key_role in ALLOWED_KEY_ROLES,
        "approval_quorum_satisfied": required_approvals > 0
        and received_approvals >= required_approvals,
        "custody_attestation_valid": _is_valid_sha256_digest(custody_attestation_hash),
        "approval_quorum_hash_valid": _is_valid_sha256_digest(approval_quorum_hash),
        "lifecycle_transition_valid": rotate_transition_valid or revoke_transition_valid,
        "revoke_reason_present": lifecycle_action != "revoke"
        or revocation_reason_code in ALLOWED_REVOCATION_REASON_CODES,
        "incident_ticket_present": _is_valid_incident_ticket(incident_ticket),
        "ci_fast_gate_passed": ci_fast_gate == "PASS",
    }


def _failed_checks(policy_checks: Mapping[str, bool]) -> list[str]:
    failed = [name for name, passed in policy_checks.items() if not passed]
    return sorted(failed)


def _reason_messages() -> Mapping[str, str]:
    return {
        "provider_supported": f"secure provider must be {SUPPORTED_PROVIDER}",
        "key_reference_scoped": "secure key reference must be scoped to provider and role",
        "role_supported": "secure key role is unsupported",
        "approval_quorum_satisfied": "received approvals are below required approvals",
        "custody_attestation_valid": "custody attestation hash must be a non-empty sha256 digest",
        "approval_quorum_hash_valid": "approval quorum hash must be a non-empty sha256 digest",
        "lifecycle_transition_valid": "lifecycle transition must satisfy rotate/revoke version invariants",
        "revoke_reason_present": "revoke action requires a supported revocation reason code",
        "incident_ticket_present": "incident ticket must follow INC-<digits> format",
        "ci_fast_gate_passed": "ci-fast-gate-failed",
    }


def _require_policy_check_map(payload: Mapping[str, object]) -> Mapping[str, bool]:
    policy_checks_raw = payload.get("policy_checks")
    if not isinstance(policy_checks_raw, dict):
        fail("policy_checks must be an object")

    policy_checks: dict[str, bool] = {}
    for key_name in _reason_messages().keys():
        if key_name not in policy_checks_raw:
            fail(f"missing policy_checks field: {key_name}")
        value = policy_checks_raw[key_name]
        if not isinstance(value, bool):
            fail(f"policy_checks.{key_name} must be boolean")
        policy_checks[key_name] = value

    return policy_checks


def _reason_key(decision: str) -> str:
    return f"secure_provider_key_lifecycle_reason_codes:{decision}:v1"


def generate_bundle(args: argparse.Namespace) -> int:
    require_pattern(
        "secure-key-reference",
        args.secure_key_reference,
        r"secure:[a-z0-9-]+:role-[a-z-]+/[A-Za-z0-9._:-]+",
        "secure-key-reference must follow secure:<provider>:role-<role>/<key-id>",
    )
    require_pattern(
        "provider",
        args.provider,
        r"[a-z0-9-]+",
        "provider must be lowercase alphanumeric with hyphens",
    )
    require_pattern(
        "key-role",
        args.key_role,
        r"[a-z-]+",
        "key-role must be lowercase with optional hyphens",
    )
    lifecycle_action = require_enum(
        "lifecycle-action", args.lifecycle_action, ALLOWED_LIFECYCLE_ACTIONS
    )
    previous_version = require_non_negative_int(
        "previous-version", str(args.previous_version)
    )
    target_version = require_non_negative_int("target-version", str(args.target_version))
    require_pattern(
        "incident-ticket",
        args.incident_ticket,
        r"[A-Za-z0-9-]+",
        "incident-ticket must be non-empty alphanumeric with hyphens",
    )
    require_pattern(
        "revocation-reason-code",
        args.revocation_reason_code,
        r"[a-z-]+",
        "revocation-reason-code must be lowercase with optional hyphens",
    )
    required_approvals = require_non_negative_int(
        "required-approvals", str(args.required_approvals)
    )
    received_approvals = require_non_negative_int(
        "received-approvals", str(args.received_approvals)
    )
    require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    policy_checks = _compute_policy_checks(
        secure_key_reference=args.secure_key_reference,
        provider=args.provider,
        key_role=args.key_role,
        lifecycle_action=lifecycle_action,
        previous_version=previous_version,
        target_version=target_version,
        incident_ticket=args.incident_ticket,
        revocation_reason_code=args.revocation_reason_code,
        required_approvals=required_approvals,
        received_approvals=received_approvals,
        custody_attestation_hash=args.custody_attestation_hash,
        approval_quorum_hash=args.approval_quorum_hash,
        ci_fast_gate=args.ci_fast_gate,
    )

    decision = DecisionAccumulator()
    reason_messages = _reason_messages()
    for check_name in reason_messages:
        decision.reject_if(not policy_checks[check_name], reason_messages[check_name])

    final_decision, decision_reasons = decision.finalize(
        "all secure-provider key-lifecycle policy checks satisfied"
    )
    reason_key = _reason_key(final_decision)
    failed_checks = _failed_checks(policy_checks)
    failed_checks_value = ",".join(failed_checks) if failed_checks else "none"

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "key_lifecycle": {
            "secure_key_reference": args.secure_key_reference,
            "provider": args.provider,
            "key_role": args.key_role,
            "lifecycle_action": lifecycle_action,
            "previous_version": previous_version,
            "target_version": target_version,
            "incident_ticket": args.incident_ticket,
            "revocation_reason_code": args.revocation_reason_code,
        },
        "approvals": {
            "required": required_approvals,
            "received": received_approvals,
            "approval_quorum_hash": args.approval_quorum_hash,
        },
        "custody_attestation_hash": args.custody_attestation_hash,
        "ci_fast_gate": args.ci_fast_gate,
        "policy_checks": policy_checks,
        "failed_checks": failed_checks,
        "decision_reasons": decision_reasons,
        "reason_key": reason_key,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    print(f"reason_key={reason_key}")
    print(f"failed_checks={failed_checks_value}")
    return 0


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
            "key_lifecycle",
            "approvals",
            "custody_attestation_hash",
            "ci_fast_gate",
            "policy_checks",
            "failed_checks",
            "decision_reasons",
            "reason_key",
            "final_decision",
        ),
    )

    schema_version = require_string(payload, "schema_version")
    if schema_version != SCHEMA_VERSION:
        fail("unexpected schema_version for secure-provider key-lifecycle evidence bundle")

    lifecycle = require_object(payload, "key_lifecycle")
    secure_key_reference = require_string(lifecycle, "secure_key_reference")
    provider = require_string(lifecycle, "provider")
    key_role = require_string(lifecycle, "key_role")
    lifecycle_action = require_string(lifecycle, "lifecycle_action")
    if lifecycle_action not in ALLOWED_LIFECYCLE_ACTIONS:
        fail("key_lifecycle.lifecycle_action must be rotate or revoke")
    previous_version = require_int(lifecycle, "previous_version", min_value=0)
    target_version = require_int(lifecycle, "target_version", min_value=0)
    incident_ticket = require_string(lifecycle, "incident_ticket")
    revocation_reason_code = require_string(lifecycle, "revocation_reason_code")

    approvals = require_object(payload, "approvals")
    required_approvals = require_int(approvals, "required", min_value=0)
    received_approvals = require_int(approvals, "received", min_value=0)
    approval_quorum_hash = require_string(approvals, "approval_quorum_hash")
    custody_attestation_hash = require_string(payload, "custody_attestation_hash")
    ci_fast_gate = require_string(payload, "ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    derived_checks = _compute_policy_checks(
        secure_key_reference=secure_key_reference,
        provider=provider,
        key_role=key_role,
        lifecycle_action=lifecycle_action,
        previous_version=previous_version,
        target_version=target_version,
        incident_ticket=incident_ticket,
        revocation_reason_code=revocation_reason_code,
        required_approvals=required_approvals,
        received_approvals=received_approvals,
        custody_attestation_hash=custody_attestation_hash,
        approval_quorum_hash=approval_quorum_hash,
        ci_fast_gate=ci_fast_gate,
    )

    policy_checks = _require_policy_check_map(payload)
    for check_name, expected_value in derived_checks.items():
        if policy_checks[check_name] != expected_value:
            fail(f"policy_checks.{check_name} does not match derived policy")

    failed_checks = payload.get("failed_checks")
    if not isinstance(failed_checks, list):
        fail("failed_checks must be an array")
    if not all(isinstance(item, str) and item for item in failed_checks):
        fail("failed_checks must contain non-empty strings")
    if failed_checks != sorted(failed_checks):
        fail("failed_checks must be sorted and deterministic")

    expected_failed_checks = _failed_checks(derived_checks)
    if failed_checks != expected_failed_checks:
        fail(
            "failed_checks mismatch: "
            f"expected failed_checks={expected_failed_checks}, found {failed_checks}"
        )

    decision_reasons = payload.get("decision_reasons")
    if not isinstance(decision_reasons, list):
        fail("decision_reasons must be an array")
    if not all(isinstance(item, str) and item for item in decision_reasons):
        fail("decision_reasons must contain non-empty strings")

    expected_decision = "NO-GO" if expected_failed_checks else "GO"
    actual_decision = require_string(payload, "final_decision")
    if actual_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    reason_key = require_string(payload, "reason_key")
    expected_reason_key = _reason_key(expected_decision)
    if reason_key != expected_reason_key:
        fail(
            "reason key mismatch: "
            f"expected reason_key={expected_reason_key}, found {reason_key}"
        )

    failed_checks_value = ",".join(expected_failed_checks) if expected_failed_checks else "none"
    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"reason_key={reason_key}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Secure-provider signer key-lifecycle evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--secure-key-reference", required=True)
    generate.add_argument("--provider", required=True)
    generate.add_argument("--key-role", required=True)
    generate.add_argument("--lifecycle-action", required=True)
    generate.add_argument("--previous-version", required=True)
    generate.add_argument("--target-version", required=True)
    generate.add_argument("--incident-ticket", required=True)
    generate.add_argument("--revocation-reason-code", required=True)
    generate.add_argument("--required-approvals", required=True)
    generate.add_argument("--received-approvals", required=True)
    generate.add_argument("--custody-attestation-hash", required=True)
    generate.add_argument("--approval-quorum-hash", required=True)
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

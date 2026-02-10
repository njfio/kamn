#!/usr/bin/env python3
"""DID lifecycle operator-binding evidence generator and policy checker."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
import re
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    parse_int,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.did.lifecycle-operator-binding.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _require_non_negative_int(name: str, raw_value: str) -> int:
    value = parse_int(name, raw_value)
    if value < 0:
        fail(f"{name} must be a non-negative integer")
    return value


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.did,
        args.actor_did,
        args.required_operator_did,
        args.mutation_action,
        args.mutation_nonce,
        args.mutation_reason_code,
        args.audit_export_id,
        args.audit_record_count,
        args.audit_digest,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all lifecycle operator-binding bundle arguments are required")

    mutation_action = args.mutation_action
    if mutation_action not in {"rotate", "revoke", "recover"}:
        fail("mutation-action must be rotate, revoke, or recover")

    ci_fast_gate = args.ci_fast_gate
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci-fast-gate must be PASS or FAIL")

    mutation_nonce = _require_non_negative_int("mutation-nonce", args.mutation_nonce)
    if mutation_nonce == 0:
        fail("mutation-nonce must be greater than zero")
    audit_record_count = _require_non_negative_int(
        "audit-record-count", args.audit_record_count
    )

    did = args.did
    actor_did = args.actor_did
    required_operator_did = args.required_operator_did
    mutation_reason_code = args.mutation_reason_code
    audit_export_id = args.audit_export_id
    audit_digest = args.audit_digest

    supported_actions = {"rotate", "revoke", "recover"}
    known_reason_codes = {
        "did_lifecycle_mutation_allowed",
        "did_lifecycle_mutation_nonce_invalid",
        "did_lifecycle_mutation_nonce_replay",
        "did_lifecycle_mutation_unauthorized_actor",
        "did_lifecycle_mutation_invalid_transition",
    }
    hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")

    operator_binding_satisfied = actor_did == required_operator_did
    mutation_action_supported = mutation_action in supported_actions
    mutation_reason_code_valid = mutation_reason_code in known_reason_codes
    authorization_granted = (
        operator_binding_satisfied
        and mutation_reason_code == "did_lifecycle_mutation_allowed"
    )
    authorization_evidence_consistent = (
        (
            operator_binding_satisfied
            and mutation_reason_code == "did_lifecycle_mutation_allowed"
        )
        or (
            (not operator_binding_satisfied)
            and mutation_reason_code == "did_lifecycle_mutation_unauthorized_actor"
        )
    )
    audit_export_id_present = bool(audit_export_id.strip())
    audit_record_count_positive = audit_record_count > 0
    audit_digest_valid = bool(hash_pattern.match(audit_digest))
    ci_fast_gate_passed = ci_fast_gate == "PASS"

    policy_checks = {
        "operator_binding_satisfied": operator_binding_satisfied,
        "mutation_action_supported": mutation_action_supported,
        "mutation_reason_code_valid": mutation_reason_code_valid,
        "authorization_granted": authorization_granted,
        "authorization_evidence_consistent": authorization_evidence_consistent,
        "audit_export_id_present": audit_export_id_present,
        "audit_record_count_positive": audit_record_count_positive,
        "audit_digest_valid": audit_digest_valid,
        "ci_fast_gate_passed": ci_fast_gate_passed,
    }

    reason_codes: list[str] = []
    if not operator_binding_satisfied:
        reason_codes.append("operator_binding_mismatch")
    if not mutation_action_supported:
        reason_codes.append("mutation_action_unsupported")
    if not mutation_reason_code_valid:
        reason_codes.append("mutation_reason_code_invalid")
    if not authorization_granted:
        reason_codes.append("mutation_not_authorized")
    if not authorization_evidence_consistent:
        reason_codes.append("authorization_evidence_inconsistent")
    if not audit_export_id_present:
        reason_codes.append("audit_export_id_missing")
    if not audit_record_count_positive:
        reason_codes.append("audit_record_count_zero")
    if not audit_digest_valid:
        reason_codes.append("audit_digest_invalid")
    if not ci_fast_gate_passed:
        reason_codes.append("ci_fast_gate_failed")
    reason_codes = sorted(reason_codes)

    is_go = all(policy_checks.values())
    final_decision = GO_DECISION if is_go else NO_GO_DECISION
    reason_key = f"did_lifecycle_operator_binding_reason_codes:{final_decision}:v1"

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "did": did,
        "actor_did": actor_did,
        "required_operator_did": required_operator_did,
        "mutation_action": mutation_action,
        "mutation_nonce": mutation_nonce,
        "mutation_reason_code": mutation_reason_code,
        "audit_export": {
            "export_id": audit_export_id,
            "record_count": audit_record_count,
            "digest": audit_digest,
        },
        "ci_fast_gate": ci_fast_gate,
        "reason_key": reason_key,
        "policy_checks": policy_checks,
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"reason_key={reason_key}")
    print(f"final_decision={final_decision}")
    return 0


def check_bundle(args: argparse.Namespace) -> int:
    if not args.bundle_file:
        fail("--bundle-file is required")

    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        fail(f"bundle file not found: {bundle_path}")

    payload = load_json(bundle_path)
    require_keys(
        payload,
        (
            "schema_version",
            "generated_at",
            "did",
            "actor_did",
            "required_operator_did",
            "mutation_action",
            "mutation_nonce",
            "mutation_reason_code",
            "audit_export",
            "ci_fast_gate",
            "reason_key",
            "policy_checks",
            "reason_codes",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unsupported schema_version for lifecycle operator-binding evidence bundle")

    mutation_nonce = payload["mutation_nonce"]
    if not isinstance(mutation_nonce, int):
        fail("mutation_nonce must be an integer")
    if mutation_nonce <= 0:
        fail("mutation_nonce must be greater than zero")

    mutation_action = payload["mutation_action"]
    if mutation_action not in {"rotate", "revoke", "recover"}:
        fail("mutation_action must be rotate, revoke, or recover")

    ci_fast_gate = payload["ci_fast_gate"]
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    known_reason_codes = {
        "did_lifecycle_mutation_allowed",
        "did_lifecycle_mutation_nonce_invalid",
        "did_lifecycle_mutation_nonce_replay",
        "did_lifecycle_mutation_unauthorized_actor",
        "did_lifecycle_mutation_invalid_transition",
    }
    mutation_reason_code = payload["mutation_reason_code"]
    if not isinstance(mutation_reason_code, str):
        fail("mutation_reason_code must be a string")

    audit_export = payload["audit_export"]
    if not isinstance(audit_export, dict):
        fail("audit_export must be an object")
    for field_name in ("export_id", "record_count", "digest"):
        if field_name not in audit_export:
            fail(f"missing audit_export field: {field_name}")

    audit_record_count = audit_export["record_count"]
    if not isinstance(audit_record_count, int):
        fail("audit_export.record_count must be an integer")
    if audit_record_count < 0:
        fail("audit_export.record_count must be non-negative")

    hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")

    actor_did = payload["actor_did"]
    required_operator_did = payload["required_operator_did"]

    operator_binding_satisfied = actor_did == required_operator_did
    mutation_action_supported = mutation_action in {"rotate", "revoke", "recover"}
    mutation_reason_code_valid = mutation_reason_code in known_reason_codes
    authorization_granted = (
        operator_binding_satisfied
        and mutation_reason_code == "did_lifecycle_mutation_allowed"
    )
    authorization_evidence_consistent = (
        (
            operator_binding_satisfied
            and mutation_reason_code == "did_lifecycle_mutation_allowed"
        )
        or (
            (not operator_binding_satisfied)
            and mutation_reason_code == "did_lifecycle_mutation_unauthorized_actor"
        )
    )
    audit_export_id_present = bool(str(audit_export["export_id"]).strip())
    audit_record_count_positive = audit_record_count > 0
    audit_digest_valid = bool(hash_pattern.match(str(audit_export["digest"])))
    ci_fast_gate_passed = ci_fast_gate == "PASS"

    derived_checks = {
        "operator_binding_satisfied": operator_binding_satisfied,
        "mutation_action_supported": mutation_action_supported,
        "mutation_reason_code_valid": mutation_reason_code_valid,
        "authorization_granted": authorization_granted,
        "authorization_evidence_consistent": authorization_evidence_consistent,
        "audit_export_id_present": audit_export_id_present,
        "audit_record_count_positive": audit_record_count_positive,
        "audit_digest_valid": audit_digest_valid,
        "ci_fast_gate_passed": ci_fast_gate_passed,
    }

    policy_checks = payload["policy_checks"]
    if not isinstance(policy_checks, dict):
        fail("policy_checks must be an object")
    for field_name in derived_checks:
        if field_name not in policy_checks:
            fail(f"missing policy_checks field: {field_name}")
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")
        if policy_checks[field_name] != derived_checks[field_name]:
            fail(f"policy_checks.{field_name} does not match derived policy")

    expected_decision = GO_DECISION if all(derived_checks.values()) else NO_GO_DECISION
    actual_decision = payload["final_decision"]
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    reason_key = payload["reason_key"]
    if not isinstance(reason_key, str) or not reason_key:
        fail("reason_key must be a non-empty string")
    expected_reason_key = (
        f"did_lifecycle_operator_binding_reason_codes:{actual_decision}:v1"
    )
    if reason_key != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {reason_key}"
        )

    failed_checks: list[str] = []
    if not operator_binding_satisfied:
        failed_checks.append("operator_binding_mismatch")
    if not mutation_action_supported:
        failed_checks.append("mutation_action_unsupported")
    if not mutation_reason_code_valid:
        failed_checks.append("mutation_reason_code_invalid")
    if not authorization_granted:
        failed_checks.append("mutation_not_authorized")
    if not authorization_evidence_consistent:
        failed_checks.append("authorization_evidence_inconsistent")
    if not audit_export_id_present:
        failed_checks.append("audit_export_id_missing")
    if not audit_record_count_positive:
        failed_checks.append("audit_record_count_zero")
    if not audit_digest_valid:
        failed_checks.append("audit_digest_invalid")
    if not ci_fast_gate_passed:
        failed_checks.append("ci_fast_gate_failed")
    failed_checks = sorted(failed_checks)

    reason_codes = payload["reason_codes"]
    if not isinstance(reason_codes, list):
        fail("reason_codes must be an array")
    if not all(isinstance(item, str) and item for item in reason_codes):
        fail("reason_codes must contain non-empty strings")
    if reason_codes != sorted(reason_codes):
        fail("reason_codes must be sorted and deterministic")
    if reason_codes != failed_checks:
        fail(
            "reason_codes mismatch: "
            f"expected reason_codes={failed_checks}, found {reason_codes}"
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
            "DID lifecycle operator-binding evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--did")
    generate.add_argument("--actor-did")
    generate.add_argument("--required-operator-did")
    generate.add_argument("--mutation-action")
    generate.add_argument("--mutation-nonce")
    generate.add_argument("--mutation-reason-code")
    generate.add_argument("--audit-export-id")
    generate.add_argument("--audit-record-count")
    generate.add_argument("--audit-digest")
    generate.add_argument("--ci-fast-gate")
    generate.set_defaults(handler=generate_bundle)

    check = subparsers.add_parser("check")
    check.add_argument("--bundle-file")
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

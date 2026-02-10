#!/usr/bin/env python3
"""Federated DID handshake evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
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

SCHEMA_VERSION = "kamn.did.federated-handshake.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_bool(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("boolean fields must be true or false")


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.handshake_id,
        args.subject_did,
        args.local_network,
        args.remote_network,
        args.resolver_cache_hit,
        args.resolver_version,
        args.signature_policy,
        args.nonce_monotonic,
        args.downgrade_detected,
        args.partition_sequence_monotonic,
        args.required_quorum,
        args.received_quorum,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all handshake bundle arguments are required")

    signature_policy = args.signature_policy
    ci_fast_gate = args.ci_fast_gate
    for status in (signature_policy, ci_fast_gate):
        if status not in {"PASS", "FAIL"}:
            fail("signature-policy and ci-fast-gate must be PASS or FAIL")

    required_quorum = parse_int("required-quorum", args.required_quorum)
    received_quorum = parse_int("received-quorum", args.received_quorum)
    if required_quorum < 0:
        fail("required-quorum must be a non-negative integer")
    if received_quorum < 0:
        fail("received-quorum must be a non-negative integer")
    if required_quorum == 0:
        fail("required-quorum must be greater than zero")

    resolver_cache_hit = _parse_bool(args.resolver_cache_hit)
    nonce_monotonic = _parse_bool(args.nonce_monotonic)
    downgrade_detected = _parse_bool(args.downgrade_detected)
    partition_sequence_monotonic = _parse_bool(args.partition_sequence_monotonic)

    handshake_id = args.handshake_id
    subject_did = args.subject_did
    local_network = args.local_network
    remote_network = args.remote_network
    resolver_version = args.resolver_version

    resolver_version_present = bool(resolver_version.strip())
    signature_policy_passed = signature_policy == "PASS"
    quorum_satisfied = received_quorum >= required_quorum
    replay_guard_passed = nonce_monotonic and partition_sequence_monotonic
    downgrade_guard_passed = not downgrade_detected

    is_go = (
        resolver_version_present
        and signature_policy_passed
        and quorum_satisfied
        and replay_guard_passed
        and downgrade_guard_passed
        and ci_fast_gate == "PASS"
    )

    reason_codes: list[str] = []
    if not resolver_version_present:
        reason_codes.append("resolver_version_missing")
    if not signature_policy_passed:
        reason_codes.append("signature_policy_failed")
    if not quorum_satisfied:
        reason_codes.append("quorum_shortfall")
    if not nonce_monotonic:
        reason_codes.append("nonce_replay_detected")
    if not partition_sequence_monotonic:
        reason_codes.append("partition_sequence_replayed")
    if downgrade_detected:
        reason_codes.append("downgrade_attack_detected")
    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    final_decision = GO_DECISION if is_go else NO_GO_DECISION
    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "handshake_id": handshake_id,
        "subject_did": subject_did,
        "local_network": local_network,
        "remote_network": remote_network,
        "resolver_cache_hit": resolver_cache_hit,
        "resolver_version": resolver_version,
        "signature_policy": signature_policy,
        "nonce_monotonic": nonce_monotonic,
        "downgrade_detected": downgrade_detected,
        "partition_sequence_monotonic": partition_sequence_monotonic,
        "required_quorum": required_quorum,
        "received_quorum": received_quorum,
        "ci_fast_gate": ci_fast_gate,
        "policy_checks": {
            "resolver_version_present": resolver_version_present,
            "signature_policy_passed": signature_policy_passed,
            "quorum_satisfied": quorum_satisfied,
            "replay_guard_passed": replay_guard_passed,
            "downgrade_guard_passed": downgrade_guard_passed,
        },
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
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
            "handshake_id",
            "subject_did",
            "local_network",
            "remote_network",
            "resolver_cache_hit",
            "resolver_version",
            "signature_policy",
            "nonce_monotonic",
            "downgrade_detected",
            "partition_sequence_monotonic",
            "required_quorum",
            "received_quorum",
            "ci_fast_gate",
            "policy_checks",
            "reason_codes",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unsupported schema_version for federated DID handshake bundle")

    for field_name in (
        "resolver_cache_hit",
        "nonce_monotonic",
        "downgrade_detected",
        "partition_sequence_monotonic",
    ):
        if not isinstance(payload[field_name], bool):
            fail(f"{field_name} must be boolean")

    for field_name in ("required_quorum", "received_quorum"):
        field_value = payload[field_name]
        if not isinstance(field_value, int):
            fail(f"{field_name} must be integer")
        if field_value < 0:
            fail(f"{field_name} must be non-negative")

    if payload["required_quorum"] <= 0:
        fail("required_quorum must be greater than zero")

    for field_name in ("signature_policy", "ci_fast_gate"):
        if payload[field_name] not in {"PASS", "FAIL"}:
            fail(f"{field_name} must be PASS or FAIL")

    policy_checks = payload["policy_checks"]
    if not isinstance(policy_checks, dict):
        fail("policy_checks must be an object")

    required_policy_fields = (
        "resolver_version_present",
        "signature_policy_passed",
        "quorum_satisfied",
        "replay_guard_passed",
        "downgrade_guard_passed",
    )
    for field_name in required_policy_fields:
        if field_name not in policy_checks:
            fail(f"missing policy_checks field: {field_name}")
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

    resolver_version_present = bool(str(payload["resolver_version"]).strip())
    signature_policy_passed = payload["signature_policy"] == "PASS"
    quorum_satisfied = payload["received_quorum"] >= payload["required_quorum"]
    replay_guard_passed = payload["nonce_monotonic"] and payload["partition_sequence_monotonic"]
    downgrade_guard_passed = not payload["downgrade_detected"]

    expected_checks = {
        "resolver_version_present": resolver_version_present,
        "signature_policy_passed": signature_policy_passed,
        "quorum_satisfied": quorum_satisfied,
        "replay_guard_passed": replay_guard_passed,
        "downgrade_guard_passed": downgrade_guard_passed,
    }
    for key, expected_value in expected_checks.items():
        if policy_checks[key] != expected_value:
            fail(f"policy_checks.{key} does not match derived policy")

    expected_go = (
        resolver_version_present
        and signature_policy_passed
        and quorum_satisfied
        and replay_guard_passed
        and downgrade_guard_passed
        and payload["ci_fast_gate"] == "PASS"
    )
    expected_decision = GO_DECISION if expected_go else NO_GO_DECISION
    actual_decision = payload["final_decision"]

    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    failed_checks: list[str] = []
    if not resolver_version_present:
        failed_checks.append("resolver_version_missing")
    if not signature_policy_passed:
        failed_checks.append("signature_policy_failed")
    if not quorum_satisfied:
        failed_checks.append("quorum_shortfall")
    if not payload["nonce_monotonic"]:
        failed_checks.append("nonce_replay_detected")
    if not payload["partition_sequence_monotonic"]:
        failed_checks.append("partition_sequence_replayed")
    if payload["downgrade_detected"]:
        failed_checks.append("downgrade_attack_detected")
    if payload["ci_fast_gate"] != "PASS":
        failed_checks.append("ci_fast_gate_failed")

    failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Federated DID handshake evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--handshake-id")
    generate.add_argument("--subject-did")
    generate.add_argument("--local-network")
    generate.add_argument("--remote-network")
    generate.add_argument("--resolver-cache-hit")
    generate.add_argument("--resolver-version")
    generate.add_argument("--signature-policy")
    generate.add_argument("--nonce-monotonic")
    generate.add_argument("--downgrade-detected")
    generate.add_argument("--partition-sequence-monotonic")
    generate.add_argument("--required-quorum")
    generate.add_argument("--received-quorum")
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

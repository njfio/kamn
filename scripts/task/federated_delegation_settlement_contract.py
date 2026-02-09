#!/usr/bin/env python3
"""Federated delegation settlement evidence generator and policy checker."""

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
    require_non_negative_int,
    write_json,
)

SCHEMA_VERSION = "kamn.task.federated-delegation-settlement.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_bool(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("boolean fields must be true or false")


def _parse_finality(raw_value: str) -> str:
    if raw_value in {"FINAL", "PENDING", "FAILED"}:
        return raw_value
    fail("settlement-receipt-finality must be FINAL, PENDING, or FAILED")


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("ci-fast-gate must be PASS or FAIL")


def _as_non_negative_int(raw_value: str, field_name: str) -> int:
    try:
        return require_non_negative_int(field_name, raw_value)
    except ContractError:
        fail(f"{field_name} must be a non-negative integer")
    raise AssertionError("unreachable")


def _require_bool(payload: Mapping[str, Any], field_name: str) -> bool:
    value = payload.get(field_name)
    if not isinstance(value, bool):
        fail(f"{field_name} must be boolean")
    return value


def _require_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str):
        fail(f"{field_name} must be string")
    return value


def _require_int(payload: Mapping[str, Any], field_name: str) -> int:
    value = payload.get(field_name)
    if not isinstance(value, int):
        fail(f"{field_name} must be integer")
    return value


def _compute_policy_checks(
    *,
    delegation_id: str,
    task_id: str,
    delegator_did: str,
    delegatee_did: str,
    source_network: str,
    destination_network: str,
    settlement_reference_id: str,
    expected_settlement_reference_id: str,
    settlement_receipt_finality: str,
    nonce_monotonic: bool,
    replay_detected: bool,
    partition_sequence_monotonic: bool,
    required_attestors: int,
    received_attestors: int,
) -> Mapping[str, bool]:
    delegation_context_present = all(
        bool(value.strip())
        for value in (delegation_id, task_id, delegator_did, delegatee_did)
    )
    settlement_reference_present = bool(settlement_reference_id.strip()) and bool(
        expected_settlement_reference_id.strip()
    )
    settlement_reference_match = (
        settlement_reference_id == expected_settlement_reference_id
    )
    receipt_finality_final = settlement_receipt_finality == "FINAL"
    replay_guard_passed = (
        nonce_monotonic and not replay_detected and partition_sequence_monotonic
    )
    quorum_satisfied = received_attestors >= required_attestors
    cross_network_delegation = (
        bool(source_network.strip())
        and bool(destination_network.strip())
        and source_network != destination_network
    )

    return {
        "delegation_context_present": delegation_context_present,
        "settlement_reference_present": settlement_reference_present,
        "settlement_reference_match": settlement_reference_match,
        "receipt_finality_final": receipt_finality_final,
        "replay_guard_passed": replay_guard_passed,
        "quorum_satisfied": quorum_satisfied,
        "cross_network_delegation": cross_network_delegation,
    }


def _reason_codes(
    *,
    delegation_context_present: bool,
    settlement_reference_present: bool,
    settlement_reference_match: bool,
    receipt_finality_final: bool,
    nonce_monotonic: bool,
    replay_detected: bool,
    partition_sequence_monotonic: bool,
    quorum_satisfied: bool,
    cross_network_delegation: bool,
    ci_fast_gate: str,
) -> list[str]:
    reason_codes: list[str] = []
    if not delegation_context_present:
        reason_codes.append("delegation_context_missing")
    if not settlement_reference_present:
        reason_codes.append("settlement_reference_missing")
    if settlement_reference_present and not settlement_reference_match:
        reason_codes.append("settlement_reference_drift")
    if not receipt_finality_final:
        reason_codes.append("settlement_receipt_not_final")
    if not nonce_monotonic:
        reason_codes.append("nonce_not_monotonic")
    if replay_detected:
        reason_codes.append("replay_detected")
    if not partition_sequence_monotonic:
        reason_codes.append("partition_sequence_replayed")
    if not quorum_satisfied:
        reason_codes.append("attestor_quorum_shortfall")
    if not cross_network_delegation:
        reason_codes.append("non_federated_network_pair")
    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")
    return reason_codes


def generate_bundle(args: argparse.Namespace) -> int:
    settlement_receipt_finality = _parse_finality(args.settlement_receipt_finality)
    nonce_monotonic = _parse_bool(args.nonce_monotonic)
    replay_detected = _parse_bool(args.replay_detected)
    partition_sequence_monotonic = _parse_bool(args.partition_sequence_monotonic)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)

    required_attestors = _as_non_negative_int(
        str(args.required_attestors), "required-attestors"
    )
    received_attestors = _as_non_negative_int(
        str(args.received_attestors), "received-attestors"
    )
    if required_attestors == 0:
        fail("required-attestors must be greater than zero")

    checks = _compute_policy_checks(
        delegation_id=args.delegation_id,
        task_id=args.task_id,
        delegator_did=args.delegator_did,
        delegatee_did=args.delegatee_did,
        source_network=args.source_network,
        destination_network=args.destination_network,
        settlement_reference_id=args.settlement_reference_id,
        expected_settlement_reference_id=args.expected_settlement_reference_id,
        settlement_receipt_finality=settlement_receipt_finality,
        nonce_monotonic=nonce_monotonic,
        replay_detected=replay_detected,
        partition_sequence_monotonic=partition_sequence_monotonic,
        required_attestors=required_attestors,
        received_attestors=received_attestors,
    )

    is_go = (
        checks["delegation_context_present"]
        and checks["settlement_reference_present"]
        and checks["settlement_reference_match"]
        and checks["receipt_finality_final"]
        and checks["replay_guard_passed"]
        and checks["quorum_satisfied"]
        and checks["cross_network_delegation"]
        and ci_fast_gate == "PASS"
    )

    reason_codes = _reason_codes(
        delegation_context_present=checks["delegation_context_present"],
        settlement_reference_present=checks["settlement_reference_present"],
        settlement_reference_match=checks["settlement_reference_match"],
        receipt_finality_final=checks["receipt_finality_final"],
        nonce_monotonic=nonce_monotonic,
        replay_detected=replay_detected,
        partition_sequence_monotonic=partition_sequence_monotonic,
        quorum_satisfied=checks["quorum_satisfied"],
        cross_network_delegation=checks["cross_network_delegation"],
        ci_fast_gate=ci_fast_gate,
    )

    final_decision = GO_DECISION if is_go else NO_GO_DECISION
    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "delegation_id": args.delegation_id,
        "task_id": args.task_id,
        "delegator_did": args.delegator_did,
        "delegatee_did": args.delegatee_did,
        "source_network": args.source_network,
        "destination_network": args.destination_network,
        "settlement_reference_id": args.settlement_reference_id,
        "expected_settlement_reference_id": args.expected_settlement_reference_id,
        "settlement_receipt_finality": settlement_receipt_finality,
        "nonce_monotonic": nonce_monotonic,
        "replay_detected": replay_detected,
        "partition_sequence_monotonic": partition_sequence_monotonic,
        "required_attestors": required_attestors,
        "received_attestors": received_attestors,
        "ci_fast_gate": ci_fast_gate,
        "policy_checks": checks,
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
    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        fail(f"bundle file not found: {bundle_path}")

    payload = load_json(bundle_path)
    required_fields = (
        "schema_version",
        "generated_at",
        "delegation_id",
        "task_id",
        "delegator_did",
        "delegatee_did",
        "source_network",
        "destination_network",
        "settlement_reference_id",
        "expected_settlement_reference_id",
        "settlement_receipt_finality",
        "nonce_monotonic",
        "replay_detected",
        "partition_sequence_monotonic",
        "required_attestors",
        "received_attestors",
        "ci_fast_gate",
        "policy_checks",
        "reason_codes",
        "final_decision",
    )
    require_keys(payload, required_fields)

    if payload.get("schema_version") != SCHEMA_VERSION:
        fail("unsupported schema_version for federated delegation settlement bundle")

    settlement_receipt_finality = payload.get("settlement_receipt_finality")
    if settlement_receipt_finality not in {"FINAL", "PENDING", "FAILED"}:
        fail("settlement_receipt_finality must be FINAL|PENDING|FAILED")

    ci_fast_gate = payload.get("ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    nonce_monotonic = _require_bool(payload, "nonce_monotonic")
    replay_detected = _require_bool(payload, "replay_detected")
    partition_sequence_monotonic = _require_bool(
        payload, "partition_sequence_monotonic"
    )

    required_attestors = _require_int(payload, "required_attestors")
    received_attestors = _require_int(payload, "received_attestors")
    if required_attestors < 0:
        fail("required_attestors must be non-negative")
    if received_attestors < 0:
        fail("received_attestors must be non-negative")
    if required_attestors <= 0:
        fail("required_attestors must be greater than zero")

    policy_checks = payload.get("policy_checks")
    if not isinstance(policy_checks, dict):
        fail("policy_checks must be an object")

    required_policy_fields = (
        "delegation_context_present",
        "settlement_reference_present",
        "settlement_reference_match",
        "receipt_finality_final",
        "replay_guard_passed",
        "quorum_satisfied",
        "cross_network_delegation",
    )
    for field_name in required_policy_fields:
        if field_name not in policy_checks:
            fail(f"missing policy_checks field: {field_name}")
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

    checks = _compute_policy_checks(
        delegation_id=_require_string(payload, "delegation_id"),
        task_id=_require_string(payload, "task_id"),
        delegator_did=_require_string(payload, "delegator_did"),
        delegatee_did=_require_string(payload, "delegatee_did"),
        source_network=_require_string(payload, "source_network"),
        destination_network=_require_string(payload, "destination_network"),
        settlement_reference_id=_require_string(payload, "settlement_reference_id"),
        expected_settlement_reference_id=_require_string(
            payload, "expected_settlement_reference_id"
        ),
        settlement_receipt_finality=settlement_receipt_finality,
        nonce_monotonic=nonce_monotonic,
        replay_detected=replay_detected,
        partition_sequence_monotonic=partition_sequence_monotonic,
        required_attestors=required_attestors,
        received_attestors=received_attestors,
    )

    for field_name, expected_value in checks.items():
        if policy_checks[field_name] != expected_value:
            fail(f"policy_checks.{field_name} does not match derived policy")

    expected_go = (
        checks["delegation_context_present"]
        and checks["settlement_reference_present"]
        and checks["settlement_reference_match"]
        and checks["receipt_finality_final"]
        and checks["replay_guard_passed"]
        and checks["quorum_satisfied"]
        and checks["cross_network_delegation"]
        and ci_fast_gate == "PASS"
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

    failed_checks = _reason_codes(
        delegation_context_present=checks["delegation_context_present"],
        settlement_reference_present=checks["settlement_reference_present"],
        settlement_reference_match=checks["settlement_reference_match"],
        receipt_finality_final=checks["receipt_finality_final"],
        nonce_monotonic=nonce_monotonic,
        replay_detected=replay_detected,
        partition_sequence_monotonic=partition_sequence_monotonic,
        quorum_satisfied=checks["quorum_satisfied"],
        cross_network_delegation=checks["cross_network_delegation"],
        ci_fast_gate=ci_fast_gate,
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
            "Federated delegation settlement evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--delegation-id", required=True)
    generate.add_argument("--task-id", required=True)
    generate.add_argument("--delegator-did", required=True)
    generate.add_argument("--delegatee-did", required=True)
    generate.add_argument("--source-network", required=True)
    generate.add_argument("--destination-network", required=True)
    generate.add_argument("--settlement-reference-id", required=True)
    generate.add_argument("--expected-settlement-reference-id", required=True)
    generate.add_argument("--settlement-receipt-finality", required=True)
    generate.add_argument("--nonce-monotonic", required=True)
    generate.add_argument("--replay-detected", required=True)
    generate.add_argument("--partition-sequence-monotonic", required=True)
    generate.add_argument("--required-attestors", required=True)
    generate.add_argument("--received-attestors", required=True)
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

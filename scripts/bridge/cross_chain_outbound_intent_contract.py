#!/usr/bin/env python3
"""Cross-chain outbound intent evidence generator and policy checker."""

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

SCHEMA_VERSION = "kamn.bridge.cross-chain-outbound-intent.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_chain(raw_value: str) -> str:
    if raw_value in {"ethereum", "near"}:
        return raw_value
    fail("chain must be ethereum or near")


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("ci-fast-gate must be PASS or FAIL")


def _parse_duplicate_request(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("duplicate-request must be true or false")


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.chain,
        args.request_id,
        args.destination_channel,
        args.required_approvals,
        args.received_approvals,
        args.approval_quorum_hash,
        args.idempotency_key,
        args.attempt_number,
        args.payload_hash,
        args.previous_payload_hash,
        args.duplicate_request,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all cross-chain outbound intent evidence arguments are required")

    chain = _parse_chain(args.chain)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)
    duplicate_request = _parse_duplicate_request(args.duplicate_request)

    required_approvals = parse_int("required-approvals", args.required_approvals)
    received_approvals = parse_int("received-approvals", args.received_approvals)
    attempt_number = parse_int("attempt-number", args.attempt_number)

    request_id = args.request_id
    destination_channel = args.destination_channel
    approval_quorum_hash = args.approval_quorum_hash
    idempotency_key = args.idempotency_key
    payload_hash = args.payload_hash
    previous_payload_hash = args.previous_payload_hash

    decision_reasons: list[str] = []
    if not request_id.strip():
        decision_reasons.append("request_id must not be empty")
    if not destination_channel.startswith(f"{chain}:"):
        decision_reasons.append("destination channel must match selected chain prefix")
    if required_approvals <= 0:
        decision_reasons.append("required approvals must be greater than zero")
    if received_approvals < required_approvals:
        decision_reasons.append("received approvals are below required approvals")
    if not approval_quorum_hash.startswith("sha256:") or len(approval_quorum_hash) <= len(
        "sha256:"
    ):
        decision_reasons.append("approval quorum hash must be a non-empty sha256 digest")
    if not idempotency_key.startswith("idemp:") or len(idempotency_key) <= len("idemp:"):
        decision_reasons.append("idempotency key must use idemp:<value> format")
    if attempt_number < 1:
        decision_reasons.append("attempt number must be at least 1")

    for field_name, field_value in (
        ("payload_hash", payload_hash),
        ("previous_payload_hash", previous_payload_hash),
    ):
        if not field_value.startswith("sha256:") or len(field_value) <= len("sha256:"):
            decision_reasons.append(f"{field_name} must be a non-empty sha256 digest")

    if attempt_number > 1 and payload_hash != previous_payload_hash:
        decision_reasons.append("retry payload hash drift detected")
    if duplicate_request:
        decision_reasons.append("duplicate request replay detected")
    if ci_fast_gate != "PASS":
        decision_reasons.append("ci-fast-gate-failed")

    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    if not decision_reasons:
        decision_reasons.append("all outbound intent approval/idempotency gates satisfied")

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "chain": chain,
        "request_id": request_id,
        "destination_channel": destination_channel,
        "approvals": {
            "required": required_approvals,
            "received": received_approvals,
            "approval_quorum_hash": approval_quorum_hash,
        },
        "retry": {
            "idempotency_key": idempotency_key,
            "attempt_number": attempt_number,
            "payload_hash": payload_hash,
            "previous_payload_hash": previous_payload_hash,
            "duplicate_request": duplicate_request,
        },
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
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
            "chain",
            "request_id",
            "destination_channel",
            "approvals",
            "retry",
            "ci_fast_gate",
            "decision_reasons",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unexpected schema_version for cross-chain outbound intent evidence bundle")

    chain = payload["chain"]
    if chain not in {"ethereum", "near"}:
        fail("chain must be ethereum or near")

    request_id = payload["request_id"]
    if not isinstance(request_id, str) or not request_id.strip():
        fail("request_id must be a non-empty string")

    destination_channel = payload["destination_channel"]
    if not isinstance(destination_channel, str):
        fail("destination_channel must be a string")
    if not destination_channel.startswith(f"{chain}:"):
        fail("destination channel must match selected chain prefix")

    approvals = payload["approvals"]
    if not isinstance(approvals, dict):
        fail("approvals must be an object")
    for key in ("required", "received", "approval_quorum_hash"):
        if key not in approvals:
            fail(f"approvals missing field: {key}")
    if not isinstance(approvals["required"], int) or approvals["required"] < 0:
        fail("approvals.required must be a non-negative integer")
    if not isinstance(approvals["received"], int) or approvals["received"] < 0:
        fail("approvals.received must be a non-negative integer")
    if not isinstance(approvals["approval_quorum_hash"], str):
        fail("approvals.approval_quorum_hash must be a string")

    retry = payload["retry"]
    if not isinstance(retry, dict):
        fail("retry must be an object")
    for key in (
        "idempotency_key",
        "attempt_number",
        "payload_hash",
        "previous_payload_hash",
        "duplicate_request",
    ):
        if key not in retry:
            fail(f"retry missing field: {key}")
    if not isinstance(retry["idempotency_key"], str):
        fail("retry.idempotency_key must be a string")
    if not isinstance(retry["attempt_number"], int):
        fail("retry.attempt_number must be an integer")
    if not isinstance(retry["payload_hash"], str):
        fail("retry.payload_hash must be a string")
    if not isinstance(retry["previous_payload_hash"], str):
        fail("retry.previous_payload_hash must be a string")
    if not isinstance(retry["duplicate_request"], bool):
        fail("retry.duplicate_request must be a boolean")

    ci_fast_gate = payload["ci_fast_gate"]
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    decision_reasons = payload["decision_reasons"]
    if not isinstance(decision_reasons, list) or not all(
        isinstance(item, str) for item in decision_reasons
    ):
        fail("decision_reasons must be an array of strings")

    expected_go = True
    if approvals["required"] <= 0:
        expected_go = False
    if approvals["received"] < approvals["required"]:
        expected_go = False
    if not approvals["approval_quorum_hash"].startswith("sha256:") or len(
        approvals["approval_quorum_hash"]
    ) <= len("sha256:"):
        expected_go = False
    if not retry["idempotency_key"].startswith("idemp:") or len(
        retry["idempotency_key"]
    ) <= len("idemp:"):
        expected_go = False
    if retry["attempt_number"] < 1:
        expected_go = False
    for hash_field in ("payload_hash", "previous_payload_hash"):
        hash_value = retry[hash_field]
        if not hash_value.startswith("sha256:") or len(hash_value) <= len("sha256:"):
            expected_go = False
    if (
        retry["attempt_number"] > 1
        and retry["payload_hash"] != retry["previous_payload_hash"]
    ):
        expected_go = False
    if retry["duplicate_request"]:
        expected_go = False
    if ci_fast_gate != "PASS":
        expected_go = False

    expected_decision = GO_DECISION if expected_go else NO_GO_DECISION
    actual_decision = payload["final_decision"]
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Cross-chain outbound intent evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--chain")
    generate.add_argument("--request-id")
    generate.add_argument("--destination-channel")
    generate.add_argument("--required-approvals")
    generate.add_argument("--received-approvals")
    generate.add_argument("--approval-quorum-hash")
    generate.add_argument("--idempotency-key")
    generate.add_argument("--attempt-number")
    generate.add_argument("--payload-hash")
    generate.add_argument("--previous-payload-hash")
    generate.add_argument("--duplicate-request")
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

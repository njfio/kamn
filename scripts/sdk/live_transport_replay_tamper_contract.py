#!/usr/bin/env python3
"""Live transport replay/tamper evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_enum,
    require_keys,
    require_non_negative_int,
    require_pattern,
    require_string,
    write_json,
)

SCHEMA_VERSION = "kamn.sdk.live-transport-replay-tamper-evidence.v1"


def _parse_bool(name: str, raw_value: str) -> bool:
    if raw_value not in {"true", "false"}:
        fail(f"{name} must be true or false")
    return raw_value == "true"


def _reason_key(reason_code: str) -> str:
    return f"live_transport_replay_tamper_reason:{reason_code}:v1"


def _derive_reason_codes(
    *,
    signature_status: str,
    replay_detected: bool,
    tamper_detected: bool,
    ci_fast_gate: str,
) -> list[str]:
    reason_codes: list[str] = []
    if signature_status == "mismatch":
        reason_codes.append("signature_mismatch_detected")
    elif signature_status == "malformed":
        reason_codes.append("malformed_signature_detected")

    if replay_detected:
        reason_codes.append("replay_nonce_detected")
    if tamper_detected:
        reason_codes.append("tamper_payload_detected")
    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    if not reason_codes:
        return ["none"]
    return reason_codes


def _expected_decision(reason_codes: list[str]) -> str:
    return "GO" if reason_codes == ["none"] else "NO-GO"


def generate_bundle(args: argparse.Namespace) -> int:
    transport_lane_id = require_pattern(
        "transport-lane-id",
        args.transport_lane_id,
        r"[a-z0-9._:-]+",
        "transport-lane-id must be lowercase and URL-safe",
    )
    message_id = require_pattern(
        "message-id",
        args.message_id,
        r"[A-Za-z0-9._:-]+",
        "message-id must be non-empty and URL-safe",
    )
    from_did = require_pattern(
        "from-did",
        args.from_did,
        r"kamn:did:agent:[A-Za-z0-9_.:-]+",
        "from-did must be a valid kamn agent DID",
    )
    to_did = require_pattern(
        "to-did",
        args.to_did,
        r"kamn:did:agent:[A-Za-z0-9_.:-]+",
        "to-did must be a valid kamn agent DID",
    )
    nonce = require_non_negative_int("nonce", str(args.nonce))
    if nonce <= 0:
        fail("nonce must be greater than zero")

    signature_status = require_enum(
        "signature-status", args.signature_status, ("valid", "mismatch", "malformed")
    )
    replay_detected = _parse_bool("replay-detected", args.replay_detected)
    tamper_detected = _parse_bool("tamper-detected", args.tamper_detected)
    ci_fast_gate = require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    reason_codes = _derive_reason_codes(
        signature_status=signature_status,
        replay_detected=replay_detected,
        tamper_detected=tamper_detected,
        ci_fast_gate=ci_fast_gate,
    )
    reason_keys = [_reason_key(code) for code in reason_codes]
    final_decision = _expected_decision(reason_codes)
    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": generated_at,
        "transport_lane_id": transport_lane_id,
        "message": {
            "message_id": message_id,
            "from_did": from_did,
            "to_did": to_did,
            "nonce": nonce,
        },
        "signature_status": signature_status,
        "replay_detected": replay_detected,
        "tamper_detected": tamper_detected,
        "ci_fast_gate": ci_fast_gate,
        "reason_codes": reason_codes,
        "reason_keys": reason_keys,
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
    require_keys(
        payload,
        (
            "schema_version",
            "generated_at",
            "transport_lane_id",
            "message",
            "signature_status",
            "replay_detected",
            "tamper_detected",
            "ci_fast_gate",
            "reason_codes",
            "reason_keys",
            "final_decision",
        ),
    )

    schema_version = require_string(payload, "schema_version")
    if schema_version != SCHEMA_VERSION:
        fail("unexpected schema_version for live transport replay/tamper evidence bundle")

    transport_lane_id = require_string(payload, "transport_lane_id")
    require_pattern(
        "transport_lane_id",
        transport_lane_id,
        r"[a-z0-9._:-]+",
        "transport_lane_id must be lowercase and URL-safe",
    )
    signature_status = require_enum(
        "signature_status",
        require_string(payload, "signature_status"),
        ("valid", "mismatch", "malformed"),
    )
    ci_fast_gate = require_enum(
        "ci_fast_gate",
        require_string(payload, "ci_fast_gate"),
        ("PASS", "FAIL"),
    )

    replay_detected = payload.get("replay_detected")
    if not isinstance(replay_detected, bool):
        fail("replay_detected must be a boolean")
    tamper_detected = payload.get("tamper_detected")
    if not isinstance(tamper_detected, bool):
        fail("tamper_detected must be a boolean")

    message = payload.get("message")
    if not isinstance(message, dict):
        fail("message must be an object")
    message_id = require_string(message, "message_id")
    require_pattern(
        "message.message_id",
        message_id,
        r"[A-Za-z0-9._:-]+",
        "message.message_id must be non-empty and URL-safe",
    )
    require_pattern(
        "message.from_did",
        require_string(message, "from_did"),
        r"kamn:did:agent:[A-Za-z0-9_.:-]+",
        "message.from_did must be a valid kamn agent DID",
    )
    require_pattern(
        "message.to_did",
        require_string(message, "to_did"),
        r"kamn:did:agent:[A-Za-z0-9_.:-]+",
        "message.to_did must be a valid kamn agent DID",
    )
    nonce = message.get("nonce")
    if not isinstance(nonce, int):
        fail("message.nonce must be an integer")
    if nonce <= 0:
        fail("message.nonce must be greater than zero")

    reason_codes = payload.get("reason_codes")
    if not isinstance(reason_codes, list):
        fail("reason_codes must be an array")
    if not reason_codes or not all(isinstance(code, str) and code for code in reason_codes):
        fail("reason_codes must contain non-empty strings")
    reason_keys = payload.get("reason_keys")
    if not isinstance(reason_keys, list):
        fail("reason_keys must be an array")
    if len(reason_keys) != len(reason_codes):
        fail("reason_keys must match reason_codes length")
    if not all(isinstance(key, str) and key for key in reason_keys):
        fail("reason_keys must contain non-empty strings")

    expected_reason_codes = _derive_reason_codes(
        signature_status=signature_status,
        replay_detected=replay_detected,
        tamper_detected=tamper_detected,
        ci_fast_gate=ci_fast_gate,
    )
    if reason_codes != expected_reason_codes:
        fail(
            "reason_codes mismatch: "
            f"expected reason_codes={expected_reason_codes}, found {reason_codes}"
        )

    expected_reason_keys = [_reason_key(code) for code in expected_reason_codes]
    if reason_keys != expected_reason_keys:
        fail(
            "reason_keys mismatch: "
            f"expected reason_keys={expected_reason_keys}, found {reason_keys}"
        )

    final_decision = require_enum(
        "final_decision", require_string(payload, "final_decision"), ("GO", "NO-GO")
    )
    expected_decision = _expected_decision(expected_reason_codes)
    if final_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {final_decision}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={final_decision}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Live transport replay/tamper evidence contract utilities."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--transport-lane-id", required=True)
    generate.add_argument("--message-id", required=True)
    generate.add_argument("--from-did", required=True)
    generate.add_argument("--to-did", required=True)
    generate.add_argument("--nonce", required=True)
    generate.add_argument("--signature-status", required=True)
    generate.add_argument("--replay-detected", required=True)
    generate.add_argument("--tamper-detected", required=True)
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

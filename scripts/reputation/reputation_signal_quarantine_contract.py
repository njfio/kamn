#!/usr/bin/env python3
"""Reputation signal quarantine evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import re
import sys
from typing import Any, Mapping

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.reputation.signal-quarantine-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
ALLOW_ACTION = "ALLOW"
QUARANTINE_ACTION = "QUARANTINE"
NON_NEGATIVE_INT_PATTERN = re.compile(r"^[0-9]+$")
DID_PATTERN = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")
HASH_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")


def _parse_bool(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("boolean fields must be true or false")


def _parse_lane(raw_value: str) -> str:
    if raw_value in {"contract", "deep"}:
        return raw_value
    fail("lane must be contract or deep")


def _parse_signal_kind(raw_value: str) -> str:
    if raw_value in {"ENDORSEMENT", "DISPUTE", "CAPABILITY", "DELIVERY"}:
        return raw_value
    fail("signal-kind must be ENDORSEMENT, DISPUTE, CAPABILITY, or DELIVERY")


def _parse_source_channel(raw_value: str) -> str:
    if raw_value in {"TELEGRAM", "DISCORD", "API", "SYSTEM"}:
        return raw_value
    fail("source-channel must be TELEGRAM, DISCORD, API, or SYSTEM")


def _parse_pass_fail(raw_value: str, message: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail(message)


def _parse_non_negative_int(field_name: str, raw_value: str) -> int:
    if not NON_NEGATIVE_INT_PATTERN.fullmatch(raw_value):
        fail(f"{field_name} must be a non-negative integer")
    return int(raw_value)


def _compute_policy_checks(
    *,
    subject_did: str,
    payload_sha256: str,
    payload_signature_verified: str,
    event_age_seconds: int,
    nonce_unique: bool,
    rate_within_threshold: bool,
    source_attested: bool,
    ci_fast_gate: str,
) -> Mapping[str, bool]:
    return {
        "did_fields_valid": bool(DID_PATTERN.match(subject_did)),
        "payload_hash_valid": bool(HASH_PATTERN.match(payload_sha256)),
        "payload_signature_verified": payload_signature_verified == "PASS",
        "event_fresh": 0 <= event_age_seconds <= 300,
        "nonce_unique": nonce_unique,
        "rate_within_threshold": rate_within_threshold,
        "source_attested": source_attested,
        "ci_fast_gate_passed": ci_fast_gate == "PASS",
    }


def _compute_reason_codes(policy_checks: Mapping[str, bool]) -> list[str]:
    reason_codes: list[str] = []
    if not policy_checks["did_fields_valid"]:
        reason_codes.append("did_fields_invalid")
    if not policy_checks["payload_hash_valid"]:
        reason_codes.append("payload_hash_invalid")
    if not policy_checks["payload_signature_verified"]:
        reason_codes.append("payload_signature_unverified")
    if not policy_checks["event_fresh"]:
        reason_codes.append("event_stale")
    if not policy_checks["nonce_unique"]:
        reason_codes.append("nonce_replay_detected")
    if not policy_checks["rate_within_threshold"]:
        reason_codes.append("burst_threshold_exceeded")
    if not policy_checks["source_attested"]:
        reason_codes.append("source_unattested")
    if not policy_checks["ci_fast_gate_passed"]:
        reason_codes.append("ci_fast_gate_failed")
    return sorted(reason_codes)


def generate_bundle(args: argparse.Namespace) -> int:
    lane = _parse_lane(args.lane)
    signal_kind = _parse_signal_kind(args.signal_kind)
    source_channel = _parse_source_channel(args.source_channel)
    event_age_seconds = _parse_non_negative_int(
        "event-age-seconds", args.event_age_seconds
    )
    payload_signature_verified = _parse_pass_fail(
        args.payload_signature_verified,
        "payload-signature-verified must be PASS or FAIL",
    )
    ci_fast_gate = _parse_pass_fail(args.ci_fast_gate, "ci-fast-gate must be PASS or FAIL")
    nonce_unique = _parse_bool(args.nonce_unique)
    rate_within_threshold = _parse_bool(args.rate_within_threshold)
    source_attested = _parse_bool(args.source_attested)

    policy_checks = _compute_policy_checks(
        subject_did=args.subject_did,
        payload_sha256=args.payload_sha256,
        payload_signature_verified=payload_signature_verified,
        event_age_seconds=event_age_seconds,
        nonce_unique=nonce_unique,
        rate_within_threshold=rate_within_threshold,
        source_attested=source_attested,
        ci_fast_gate=ci_fast_gate,
    )
    is_go = all(policy_checks.values())
    final_decision = GO_DECISION if is_go else NO_GO_DECISION
    ingestion_action = ALLOW_ACTION if is_go else QUARANTINE_ACTION
    reason_codes = _compute_reason_codes(policy_checks)
    reason_key = f"reputation_signal_quarantine_reason_codes:{final_decision}:v1"
    evidence_key = f"reputation_signal_quarantine_contract:{lane}:v1"

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "lane": lane,
        "evidence_key": evidence_key,
        "reason_key": reason_key,
        "signal_context": {
            "signal_id": args.signal_id,
            "subject_did": args.subject_did,
            "signal_kind": signal_kind,
            "source_channel": source_channel,
            "event_age_seconds": event_age_seconds,
        },
        "signal_integrity": {
            "payload_sha256": args.payload_sha256,
            "payload_signature_verified": payload_signature_verified,
            "nonce_unique": nonce_unique,
        },
        "risk_controls": {
            "rate_within_threshold": rate_within_threshold,
            "source_attested": source_attested,
            "ci_fast_gate": ci_fast_gate,
        },
        "policy_checks": policy_checks,
        "reason_codes": reason_codes,
        "ingestion_action": ingestion_action,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"schema_version={SCHEMA_VERSION}")
    print(f"evidence_key={evidence_key}")
    print(f"reason_key={reason_key}")
    print(f"ingestion_action={ingestion_action}")
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
            "lane",
            "evidence_key",
            "reason_key",
            "signal_context",
            "signal_integrity",
            "risk_controls",
            "policy_checks",
            "reason_codes",
            "ingestion_action",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unexpected schema_version for reputation signal quarantine evidence bundle")

    lane = payload["lane"]
    if lane not in {"contract", "deep"}:
        fail("lane must be contract or deep")

    expected_evidence_key = f"reputation_signal_quarantine_contract:{lane}:v1"
    if payload["evidence_key"] != expected_evidence_key:
        fail(
            "evidence_key mismatch: "
            f"expected {expected_evidence_key}, found {payload['evidence_key']}"
        )

    signal_context = payload["signal_context"]
    if not isinstance(signal_context, dict):
        fail("signal_context must be an object")
    for field_name in (
        "signal_id",
        "subject_did",
        "signal_kind",
        "source_channel",
        "event_age_seconds",
    ):
        if field_name not in signal_context:
            fail(f"signal_context missing field: {field_name}")
    if not isinstance(signal_context["signal_id"], str) or not signal_context["signal_id"]:
        fail("signal_context.signal_id must be a non-empty string")
    if not isinstance(signal_context["subject_did"], str) or not signal_context["subject_did"]:
        fail("signal_context.subject_did must be a non-empty string")
    if signal_context["signal_kind"] not in {
        "ENDORSEMENT",
        "DISPUTE",
        "CAPABILITY",
        "DELIVERY",
    }:
        fail(
            "signal_context.signal_kind must be ENDORSEMENT, DISPUTE, CAPABILITY, or DELIVERY"
        )
    if signal_context["source_channel"] not in {"TELEGRAM", "DISCORD", "API", "SYSTEM"}:
        fail("signal_context.source_channel must be TELEGRAM, DISCORD, API, or SYSTEM")
    if not isinstance(signal_context["event_age_seconds"], int):
        fail("signal_context.event_age_seconds must be an integer")

    signal_integrity = payload["signal_integrity"]
    if not isinstance(signal_integrity, dict):
        fail("signal_integrity must be an object")
    for field_name in ("payload_sha256", "payload_signature_verified", "nonce_unique"):
        if field_name not in signal_integrity:
            fail(f"signal_integrity missing field: {field_name}")
    if not isinstance(signal_integrity["payload_sha256"], str):
        fail("signal_integrity.payload_sha256 must be a string")
    if signal_integrity["payload_signature_verified"] not in {"PASS", "FAIL"}:
        fail("signal_integrity.payload_signature_verified must be PASS or FAIL")
    if not isinstance(signal_integrity["nonce_unique"], bool):
        fail("signal_integrity.nonce_unique must be boolean")

    risk_controls = payload["risk_controls"]
    if not isinstance(risk_controls, dict):
        fail("risk_controls must be an object")
    for field_name in ("rate_within_threshold", "source_attested", "ci_fast_gate"):
        if field_name not in risk_controls:
            fail(f"risk_controls missing field: {field_name}")
    if not isinstance(risk_controls["rate_within_threshold"], bool):
        fail("risk_controls.rate_within_threshold must be boolean")
    if not isinstance(risk_controls["source_attested"], bool):
        fail("risk_controls.source_attested must be boolean")
    if risk_controls["ci_fast_gate"] not in {"PASS", "FAIL"}:
        fail("risk_controls.ci_fast_gate must be PASS or FAIL")

    policy_checks = payload["policy_checks"]
    if not isinstance(policy_checks, dict):
        fail("policy_checks must be an object")
    required_checks = (
        "did_fields_valid",
        "payload_hash_valid",
        "payload_signature_verified",
        "event_fresh",
        "nonce_unique",
        "rate_within_threshold",
        "source_attested",
        "ci_fast_gate_passed",
    )
    for field_name in required_checks:
        if field_name not in policy_checks:
            fail(f"policy_checks missing field: {field_name}")
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

    derived_checks = _compute_policy_checks(
        subject_did=signal_context["subject_did"],
        payload_sha256=signal_integrity["payload_sha256"],
        payload_signature_verified=signal_integrity["payload_signature_verified"],
        event_age_seconds=signal_context["event_age_seconds"],
        nonce_unique=signal_integrity["nonce_unique"],
        rate_within_threshold=risk_controls["rate_within_threshold"],
        source_attested=risk_controls["source_attested"],
        ci_fast_gate=risk_controls["ci_fast_gate"],
    )
    for key, value in derived_checks.items():
        if policy_checks[key] != value:
            fail(f"policy_checks.{key} does not match derived policy")

    expected_decision = GO_DECISION if all(derived_checks.values()) else NO_GO_DECISION
    actual_decision = payload["final_decision"]
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    expected_ingestion_action = (
        ALLOW_ACTION if actual_decision == GO_DECISION else QUARANTINE_ACTION
    )
    ingestion_action = payload["ingestion_action"]
    if ingestion_action not in {ALLOW_ACTION, QUARANTINE_ACTION}:
        fail("ingestion_action must be ALLOW or QUARANTINE")
    if ingestion_action != expected_ingestion_action:
        fail(
            "ingestion_action mismatch: "
            f"expected {expected_ingestion_action}, found {ingestion_action}"
        )

    reason_key = payload["reason_key"]
    if not isinstance(reason_key, str) or not reason_key:
        fail("reason_key must be a non-empty string")
    expected_reason_key = (
        f"reputation_signal_quarantine_reason_codes:{actual_decision}:v1"
    )
    if reason_key != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {reason_key}"
        )

    reason_codes = payload["reason_codes"]
    if not isinstance(reason_codes, list):
        fail("reason_codes must be an array")
    if not all(isinstance(item, str) and item for item in reason_codes):
        fail("reason_codes must contain non-empty strings")
    if reason_codes != sorted(reason_codes):
        fail("reason_codes must be sorted and deterministic")

    failed_checks = _compute_reason_codes(derived_checks)
    if reason_codes != failed_checks:
        fail(
            "reason_codes mismatch: "
            f"expected reason_codes={failed_checks}, found {reason_codes}"
        )

    failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"schema_version={payload['schema_version']}")
    print(f"evidence_key={payload['evidence_key']}")
    print(f"reason_key={payload['reason_key']}")
    print(f"final_decision={actual_decision}")
    print(f"ingestion_action={ingestion_action}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Reputation signal quarantine evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--lane", required=True)
    generate.add_argument("--signal-id", required=True)
    generate.add_argument("--subject-did", required=True)
    generate.add_argument("--signal-kind", required=True)
    generate.add_argument("--source-channel", required=True)
    generate.add_argument("--event-age-seconds", required=True)
    generate.add_argument("--payload-sha256", required=True)
    generate.add_argument("--payload-signature-verified", required=True)
    generate.add_argument("--nonce-unique", required=True)
    generate.add_argument("--rate-within-threshold", required=True)
    generate.add_argument("--source-attested", required=True)
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

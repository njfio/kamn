#!/usr/bin/env python3
"""Processor proof artifact schema evidence generator and policy checker."""

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
    require_keys,
    require_object,
    require_string,
    write_json,
)

SCHEMA_VERSION = "kamn.zk.processor-proof-artifact-evidence.v1"
EVIDENCE_KEY = "zk_processor_proof_artifact_contract:evidence:v1"
REASON_PREFIX = "zk_processor_proof_artifact_reason_codes"

_ARTIFACT_ID_PATTERN = re.compile(r"[A-Za-z0-9._:-]+")


def _selector_is_valid(selector: str) -> bool:
    if not selector:
        return False
    if selector.startswith(".") or selector.endswith("."):
        return False
    if ".." in selector:
        return False
    return all(
        ch.isalnum() or ch in ("_", "-", ".")
        for ch in selector
    )


def _compute_policy_checks(
    artifact_id: str,
    message_id: str,
    payload_commitment: str,
    proof_value: str,
    private_selectors: list[str],
    ci_fast_gate: str,
) -> Mapping[str, bool]:
    deduplicated_selectors = set(private_selectors)
    return {
        "artifact_id_format_valid": bool(_ARTIFACT_ID_PATTERN.fullmatch(artifact_id or "")),
        "message_id_format_valid": message_id.startswith("urn:uuid:"),
        "payload_commitment_format_valid": payload_commitment.startswith("fnv1a64:")
        and payload_commitment != "fnv1a64:",
        "proof_value_format_valid": proof_value.startswith("proof:"),
        "private_selectors_present": len(private_selectors) > 0,
        "private_selector_format_valid": all(
            _selector_is_valid(selector) for selector in private_selectors
        ),
        "private_selector_deduplicated": len(deduplicated_selectors)
        == len(private_selectors),
        "ci_fast_gate_passed": ci_fast_gate == "PASS",
    }


def _reason_messages() -> Mapping[str, str]:
    return {
        "artifact_id_format_valid": "artifact_id must match [A-Za-z0-9._:-]+",
        "message_id_format_valid": "message_id must start with urn:uuid:",
        "payload_commitment_format_valid": "payload_commitment must start with fnv1a64:",
        "proof_value_format_valid": "proof_value must start with proof:",
        "private_selectors_present": "at least one private selector is required",
        "private_selector_format_valid": "private selector syntax is invalid",
        "private_selector_deduplicated": "private selectors must be deduplicated",
        "ci_fast_gate_passed": "ci-fast-gate-failed",
    }


def _reason_key(decision: str) -> str:
    return f"{REASON_PREFIX}:{decision}:v1"


def _failed_checks(policy_checks: Mapping[str, bool]) -> list[str]:
    return sorted([name for name, passed in policy_checks.items() if not passed])


def _parse_private_selectors(values: list[str]) -> list[str]:
    selectors: list[str] = []
    for value in values:
        parsed = value.strip()
        if parsed:
            selectors.append(parsed)
    return selectors


def _require_policy_checks(payload: Mapping[str, object]) -> Mapping[str, bool]:
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


def _require_private_selectors(payload: Mapping[str, object]) -> list[str]:
    selectors_raw = payload.get("private_selectors")
    if not isinstance(selectors_raw, list):
        fail("private_selectors must be a list")
    selectors: list[str] = []
    for idx, value in enumerate(selectors_raw):
        if not isinstance(value, str):
            fail(f"private_selectors[{idx}] must be a string")
        selectors.append(value)
    return selectors


def _compute_decision(policy_checks: Mapping[str, bool]) -> tuple[str, list[str], list[str]]:
    decision = DecisionAccumulator()
    reason_messages = _reason_messages()
    for check_name in reason_messages:
        decision.reject_if(not policy_checks[check_name], reason_messages[check_name])
    final_decision, decision_reasons = decision.finalize(
        "processor proof artifact schema checks satisfied"
    )
    failed_checks = _failed_checks(policy_checks)
    return final_decision, decision_reasons, failed_checks


def generate_bundle(args: argparse.Namespace) -> int:
    require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    private_selectors = _parse_private_selectors(args.private_selector)
    policy_checks = _compute_policy_checks(
        artifact_id=args.artifact_id,
        message_id=args.message_id,
        payload_commitment=args.payload_commitment,
        proof_value=args.proof_value,
        private_selectors=private_selectors,
        ci_fast_gate=args.ci_fast_gate,
    )
    final_decision, decision_reasons, failed_checks = _compute_decision(policy_checks)
    reason_key = _reason_key(final_decision)

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "evidence_key": EVIDENCE_KEY,
        "artifact": {
            "artifact_id": args.artifact_id,
            "message_id": args.message_id,
            "payload_commitment": args.payload_commitment,
            "proof_value": args.proof_value,
        },
        "private_selectors": private_selectors,
        "ci_fast_gate": args.ci_fast_gate,
        "policy_checks": policy_checks,
        "failed_checks": failed_checks,
        "decision_reasons": decision_reasons,
        "reason_key": reason_key,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"evidence_key={EVIDENCE_KEY}")
    print(f"reason_key={reason_key}")
    print(f"final_decision={final_decision}")
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
            "evidence_key",
            "artifact",
            "private_selectors",
            "ci_fast_gate",
            "policy_checks",
            "failed_checks",
            "decision_reasons",
            "reason_key",
            "final_decision",
        ),
    )
    if require_string(payload, "schema_version") != SCHEMA_VERSION:
        fail("schema_version must be kamn.zk.processor-proof-artifact-evidence.v1")
    if require_string(payload, "evidence_key") != EVIDENCE_KEY:
        fail("evidence_key mismatch")
    require_string(payload, "generated_at")
    ci_fast_gate = require_enum(
        "ci_fast_gate", require_string(payload, "ci_fast_gate"), ("PASS", "FAIL")
    )
    final_decision = require_enum(
        "final_decision", require_string(payload, "final_decision"), ("GO", "NO-GO")
    )
    reason_key = require_string(payload, "reason_key")

    artifact = require_object(payload, "artifact")
    artifact_id = require_string(artifact, "artifact_id")
    message_id = require_string(artifact, "message_id")
    payload_commitment = require_string(artifact, "payload_commitment")
    proof_value = require_string(artifact, "proof_value")
    private_selectors = _require_private_selectors(payload)
    reported_policy_checks = _require_policy_checks(payload)

    recomputed_policy_checks = _compute_policy_checks(
        artifact_id=artifact_id,
        message_id=message_id,
        payload_commitment=payload_commitment,
        proof_value=proof_value,
        private_selectors=private_selectors,
        ci_fast_gate=ci_fast_gate,
    )
    if reported_policy_checks != recomputed_policy_checks:
        fail(
            "policy_checks mismatch: "
            f"expected {recomputed_policy_checks}, found {reported_policy_checks}"
        )

    expected_decision, expected_reasons, expected_failed_checks = _compute_decision(
        recomputed_policy_checks
    )
    if final_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected {expected_decision}, found {final_decision}"
        )

    expected_reason_key = _reason_key(expected_decision)
    if reason_key != expected_reason_key:
        fail(f"reason_key mismatch: expected {expected_reason_key}, found {reason_key}")

    decision_reasons = payload.get("decision_reasons")
    if decision_reasons != expected_reasons:
        fail(
            "decision_reasons mismatch: "
            f"expected {expected_reasons}, found {decision_reasons}"
        )

    failed_checks = payload.get("failed_checks")
    if failed_checks != expected_failed_checks:
        fail(
            "failed_checks mismatch: "
            f"expected {expected_failed_checks}, found {failed_checks}"
        )

    failed_checks_value = ",".join(expected_failed_checks) if expected_failed_checks else "none"
    print("status=validated")
    print(f"bundle_file={bundle_path}")
    print(f"reason_key={expected_reason_key}")
    print(f"final_decision={expected_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate/check processor proof artifact schema contract evidence."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate_parser = subparsers.add_parser("generate", help="Generate evidence bundle")
    generate_parser.add_argument("--output-file", required=True)
    generate_parser.add_argument("--artifact-id", required=True)
    generate_parser.add_argument("--message-id", required=True)
    generate_parser.add_argument("--payload-commitment", required=True)
    generate_parser.add_argument("--proof-value", required=True)
    generate_parser.add_argument(
        "--private-selector",
        action="append",
        default=[],
        help="Private selector to enforce (repeatable)",
    )
    generate_parser.add_argument("--ci-fast-gate", default="PASS")
    generate_parser.set_defaults(handler=generate_bundle)

    check_parser = subparsers.add_parser("check", help="Validate an evidence bundle")
    check_parser.add_argument("--bundle-file", required=True)
    check_parser.set_defaults(handler=check_bundle)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return args.handler(args)
    except ContractError as err:
        print(f"error: {err}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

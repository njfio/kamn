#!/usr/bin/env python3
"""Processor proof-admission evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
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

SCHEMA_VERSION = "kamn.runtime.processor-proof-admission-report.v1"
EVIDENCE_KEY = "processor_proof_admission_contract:evidence:v1"
REASON_PREFIX = "processor_proof_admission_reason_codes"


def _parse_bool(name: str, raw_value: str) -> bool:
    value = (raw_value or "").strip().lower()
    if value in ("true", "1", "yes", "y"):
        return True
    if value in ("false", "0", "no", "n"):
        return False
    fail(f"{name} must be true or false")


def _compute_policy_checks(
    message_id_match: bool,
    commitment_match: bool,
    proof_format_valid: bool,
    replay_guard_active: bool,
    ci_fast_gate: str,
) -> Mapping[str, bool]:
    return {
        "message_id_match": message_id_match,
        "commitment_match": commitment_match,
        "proof_format_valid": proof_format_valid,
        "replay_guard_active": replay_guard_active,
        "ci_fast_gate_passed": ci_fast_gate == "PASS",
    }


def _reason_messages() -> Mapping[str, str]:
    return {
        "message_id_match": "processor admission must reject message_id mismatch",
        "commitment_match": "processor admission must reject payload commitment mismatch",
        "proof_format_valid": "processor admission must reject invalid proof format",
        "replay_guard_active": "processor admission must reject replayed artifact ids",
        "ci_fast_gate_passed": "ci-fast-gate-failed",
    }


def _reason_key(decision: str) -> str:
    return f"{REASON_PREFIX}:{decision}:v1"


def _failed_checks(policy_checks: Mapping[str, bool]) -> list[str]:
    return sorted([key for key, passed in policy_checks.items() if not passed])


def _require_policy_checks(payload: Mapping[str, object]) -> Mapping[str, bool]:
    checks_raw = payload.get("policy_checks")
    if not isinstance(checks_raw, dict):
        fail("policy_checks must be an object")

    checks: dict[str, bool] = {}
    for key_name in _reason_messages().keys():
        if key_name not in checks_raw:
            fail(f"missing policy_checks field: {key_name}")
        value = checks_raw[key_name]
        if not isinstance(value, bool):
            fail(f"policy_checks.{key_name} must be boolean")
        checks[key_name] = value
    return checks


def _compute_decision(policy_checks: Mapping[str, bool]) -> tuple[str, list[str], list[str]]:
    decision = DecisionAccumulator()
    reason_messages = _reason_messages()
    for check_name in reason_messages:
        decision.reject_if(not policy_checks[check_name], reason_messages[check_name])
    final_decision, decision_reasons = decision.finalize(
        "processor proof admission checks satisfied"
    )
    failed_checks = _failed_checks(policy_checks)
    return final_decision, decision_reasons, failed_checks


def generate_bundle(args: argparse.Namespace) -> int:
    ci_fast_gate = require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    policy_checks = _compute_policy_checks(
        message_id_match=_parse_bool("message-id-match", args.message_id_match),
        commitment_match=_parse_bool("commitment-match", args.commitment_match),
        proof_format_valid=_parse_bool("proof-format-valid", args.proof_format_valid),
        replay_guard_active=_parse_bool("replay-guard-active", args.replay_guard_active),
        ci_fast_gate=ci_fast_gate,
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
        },
        "ci_fast_gate": ci_fast_gate,
        "policy_checks": policy_checks,
        "failed_checks": failed_checks,
        "decision_reasons": decision_reasons,
        "reason_key": reason_key,
        "final_decision": final_decision,
    }

    output_file = Path(args.output_file)
    write_json(output_file, payload)

    failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
    print("status=generated")
    print(f"bundle_file={output_file}")
    print(f"evidence_key={EVIDENCE_KEY}")
    print(f"reason_key={reason_key}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def check_bundle(args: argparse.Namespace) -> int:
    bundle_file = Path(args.bundle_file)
    if not bundle_file.is_file():
        fail(f"bundle file not found: {bundle_file}")

    payload = load_json(bundle_file)
    require_keys(
        payload,
        (
            "schema_version",
            "generated_at",
            "evidence_key",
            "artifact",
            "ci_fast_gate",
            "policy_checks",
            "failed_checks",
            "decision_reasons",
            "reason_key",
            "final_decision",
        ),
    )
    if require_string(payload, "schema_version") != SCHEMA_VERSION:
        fail(f"schema_version must be {SCHEMA_VERSION}")
    if require_string(payload, "evidence_key") != EVIDENCE_KEY:
        fail("evidence_key mismatch")
    require_string(payload, "generated_at")

    artifact = require_object(payload, "artifact")
    require_string(artifact, "artifact_id")
    require_string(artifact, "message_id")

    ci_fast_gate = require_enum(
        "ci_fast_gate", require_string(payload, "ci_fast_gate"), ("PASS", "FAIL")
    )
    final_decision = require_enum(
        "final_decision", require_string(payload, "final_decision"), ("GO", "NO-GO")
    )
    reason_key = require_string(payload, "reason_key")
    checks = _require_policy_checks(payload)

    recomputed_checks = _compute_policy_checks(
        message_id_match=checks["message_id_match"],
        commitment_match=checks["commitment_match"],
        proof_format_valid=checks["proof_format_valid"],
        replay_guard_active=checks["replay_guard_active"],
        ci_fast_gate=ci_fast_gate,
    )
    if checks != recomputed_checks:
        fail(f"policy_checks mismatch: expected {recomputed_checks}, found {checks}")

    expected_decision, expected_reasons, expected_failed_checks = _compute_decision(
        recomputed_checks
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
    print(f"bundle_file={bundle_file}")
    print(f"reason_key={expected_reason_key}")
    print(f"final_decision={expected_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate/check processor proof-admission contract evidence."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate_parser = subparsers.add_parser("generate", help="Generate evidence bundle")
    generate_parser.add_argument("--output-file", required=True)
    generate_parser.add_argument("--artifact-id", required=True)
    generate_parser.add_argument("--message-id", required=True)
    generate_parser.add_argument("--message-id-match", required=True)
    generate_parser.add_argument("--commitment-match", required=True)
    generate_parser.add_argument("--proof-format-valid", required=True)
    generate_parser.add_argument("--replay-guard-active", required=True)
    generate_parser.add_argument("--ci-fast-gate", default="PASS")
    generate_parser.set_defaults(handler=generate_bundle)

    check_parser = subparsers.add_parser("check", help="Check evidence bundle")
    check_parser.add_argument("--bundle-file", required=True)
    check_parser.set_defaults(handler=check_bundle)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return args.handler(args)
    except ContractError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Watchdog proof-consensus evidence generator and policy checker."""

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
    require_int,
    require_keys,
    require_non_negative_int,
    require_object,
    require_positive_int,
    require_string,
    write_json,
)

SCHEMA_VERSION = "kamn.runtime.watchdog-proof-consensus-report.v1"
EVIDENCE_KEY = "watchdog_proof_consensus_contract:evidence:v1"
REASON_PREFIX = "watchdog_proof_consensus_reason_codes"

CONSENSUS_STATUSES = (
    "ConsensusValid",
    "ConsensusInvalid",
    "ConsensusReplay",
    "ValidatorMismatch",
)
CADENCE_VALUES = ("fast", "scheduled", "manual")
PROJECTION_KIND_BY_STATUS = {
    "ConsensusValid": "ConsensusAligned",
    "ConsensusInvalid": "InvalidProofConsensus",
    "ConsensusReplay": "ReplayProofConsensus",
    "ValidatorMismatch": "ValidatorMismatch",
}
PROJECTION_SEVERITY_BY_STATUS = {
    "ConsensusValid": "info",
    "ConsensusInvalid": "critical",
    "ConsensusReplay": "critical",
    "ValidatorMismatch": "critical",
}


def _parse_bool(name: str, raw_value: str) -> bool:
    value = (raw_value or "").strip().lower()
    if value in ("true", "1", "yes", "y"):
        return True
    if value in ("false", "0", "no", "n"):
        return False
    fail(f"{name} must be true or false")


def _reason_messages() -> Mapping[str, str]:
    return {
        "consensus_status_go": "consensus status must remain ConsensusValid for GO release posture",
        "projection_kind_match": "watchdog projection kind must match consensus status",
        "projection_severity_match": "watchdog projection severity must match consensus status",
        "quorum_coverage_valid": "attestation coverage must meet required quorum",
        "runtime_budget_within": "proof-consensus lane exceeded runtime budget",
        "evidence_complete": "proof-consensus anomaly evidence incomplete",
        "ci_fast_gate_passed": "ci-fast-gate-failed",
    }


def _reason_key(decision: str) -> str:
    return f"{REASON_PREFIX}:{decision}:v1"


def _failed_checks(policy_checks: Mapping[str, bool]) -> list[str]:
    return sorted([key for key, passed in policy_checks.items() if not passed])


def _compute_policy_checks(
    consensus_status: str,
    projection_kind: str,
    projection_severity: str,
    required_quorum: int,
    valid_attestation_count: int,
    invalid_attestation_count: int,
    replay_attestation_count: int,
    runtime_seconds: int,
    max_seconds: int,
    evidence_complete: bool,
    ci_fast_gate: str,
) -> Mapping[str, bool]:
    expected_kind = PROJECTION_KIND_BY_STATUS[consensus_status]
    expected_severity = PROJECTION_SEVERITY_BY_STATUS[consensus_status]
    total_attestation_count = (
        valid_attestation_count + invalid_attestation_count + replay_attestation_count
    )

    return {
        "consensus_status_go": consensus_status == "ConsensusValid",
        "projection_kind_match": projection_kind == expected_kind,
        "projection_severity_match": projection_severity == expected_severity,
        "quorum_coverage_valid": total_attestation_count >= required_quorum,
        "runtime_budget_within": runtime_seconds <= max_seconds,
        "evidence_complete": evidence_complete,
        "ci_fast_gate_passed": ci_fast_gate == "PASS",
    }


def _compute_decision(policy_checks: Mapping[str, bool]) -> tuple[str, list[str], list[str]]:
    decision = DecisionAccumulator()
    reason_messages = _reason_messages()
    for check_name in reason_messages:
        decision.reject_if(not policy_checks[check_name], reason_messages[check_name])
    final_decision, decision_reasons = decision.finalize(
        "watchdog proof consensus checks satisfied"
    )
    failed_checks = _failed_checks(policy_checks)
    return final_decision, decision_reasons, failed_checks


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


def generate_bundle(args: argparse.Namespace) -> int:
    consensus_status = require_enum("consensus-status", args.consensus_status, CONSENSUS_STATUSES)
    cadence = require_enum("cadence", args.cadence, CADENCE_VALUES)
    ci_fast_gate = require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    required_quorum = require_positive_int("required-quorum", args.required_quorum)
    valid_attestation_count = require_non_negative_int(
        "valid-attestation-count", args.valid_attestation_count
    )
    invalid_attestation_count = require_non_negative_int(
        "invalid-attestation-count", args.invalid_attestation_count
    )
    replay_attestation_count = require_non_negative_int(
        "replay-attestation-count", args.replay_attestation_count
    )
    runtime_seconds = require_non_negative_int("runtime-seconds", args.runtime_seconds)
    max_seconds = require_positive_int("max-seconds", args.max_seconds)
    evidence_complete = _parse_bool("evidence-complete", args.evidence_complete)

    projection_kind = PROJECTION_KIND_BY_STATUS[consensus_status]
    projection_severity = PROJECTION_SEVERITY_BY_STATUS[consensus_status]
    policy_checks = _compute_policy_checks(
        consensus_status=consensus_status,
        projection_kind=projection_kind,
        projection_severity=projection_severity,
        required_quorum=required_quorum,
        valid_attestation_count=valid_attestation_count,
        invalid_attestation_count=invalid_attestation_count,
        replay_attestation_count=replay_attestation_count,
        runtime_seconds=runtime_seconds,
        max_seconds=max_seconds,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )

    final_decision, decision_reasons, failed_checks = _compute_decision(policy_checks)
    reason_key = _reason_key(final_decision)

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "evidence_key": EVIDENCE_KEY,
        "message_id": args.message_id,
        "artifact_id": args.artifact_id,
        "consensus_status": consensus_status,
        "projection_kind": projection_kind,
        "projection_severity": projection_severity,
        "required_quorum": required_quorum,
        "valid_attestation_count": valid_attestation_count,
        "invalid_attestation_count": invalid_attestation_count,
        "replay_attestation_count": replay_attestation_count,
        "cadence": cadence,
        "runtime_seconds": runtime_seconds,
        "max_seconds": max_seconds,
        "ci_fast_gate": ci_fast_gate,
        "evidence_complete": evidence_complete,
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
            "message_id",
            "artifact_id",
            "consensus_status",
            "projection_kind",
            "projection_severity",
            "required_quorum",
            "valid_attestation_count",
            "invalid_attestation_count",
            "replay_attestation_count",
            "cadence",
            "runtime_seconds",
            "max_seconds",
            "ci_fast_gate",
            "evidence_complete",
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
    require_string(payload, "message_id")
    require_string(payload, "artifact_id")

    consensus_status = require_enum(
        "consensus_status", require_string(payload, "consensus_status"), CONSENSUS_STATUSES
    )
    projection_kind = require_string(payload, "projection_kind")
    projection_severity = require_string(payload, "projection_severity")
    cadence = require_enum("cadence", require_string(payload, "cadence"), CADENCE_VALUES)
    ci_fast_gate = require_enum(
        "ci_fast_gate", require_string(payload, "ci_fast_gate"), ("PASS", "FAIL")
    )

    required_quorum = require_int(payload, "required_quorum", min_value=1)
    valid_attestation_count = require_int(payload, "valid_attestation_count", min_value=0)
    invalid_attestation_count = require_int(payload, "invalid_attestation_count", min_value=0)
    replay_attestation_count = require_int(payload, "replay_attestation_count", min_value=0)
    runtime_seconds = require_int(payload, "runtime_seconds", min_value=0)
    max_seconds = require_int(payload, "max_seconds", min_value=1)

    evidence_complete_raw = payload.get("evidence_complete")
    if not isinstance(evidence_complete_raw, bool):
        fail("evidence_complete must be boolean")
    evidence_complete = evidence_complete_raw

    expected_kind = PROJECTION_KIND_BY_STATUS[consensus_status]
    if projection_kind != expected_kind:
        fail(f"projection_kind mismatch: expected {expected_kind}, found {projection_kind}")
    expected_severity = PROJECTION_SEVERITY_BY_STATUS[consensus_status]
    if projection_severity != expected_severity:
        fail(
            "projection_severity mismatch: "
            f"expected {expected_severity}, found {projection_severity}"
        )

    checks = _require_policy_checks(payload)
    recomputed_checks = _compute_policy_checks(
        consensus_status=consensus_status,
        projection_kind=projection_kind,
        projection_severity=projection_severity,
        required_quorum=required_quorum,
        valid_attestation_count=valid_attestation_count,
        invalid_attestation_count=invalid_attestation_count,
        replay_attestation_count=replay_attestation_count,
        runtime_seconds=runtime_seconds,
        max_seconds=max_seconds,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )
    if checks != recomputed_checks:
        fail(f"policy_checks mismatch: expected {recomputed_checks}, found {checks}")

    final_decision = require_enum(
        "final_decision", require_string(payload, "final_decision"), ("GO", "NO-GO")
    )
    reason_key = require_string(payload, "reason_key")

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
        description="Generate/check watchdog proof-consensus contract evidence."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate_parser = subparsers.add_parser("generate", help="Generate evidence bundle")
    generate_parser.add_argument("--output-file", required=True)
    generate_parser.add_argument("--message-id", required=True)
    generate_parser.add_argument("--artifact-id", required=True)
    generate_parser.add_argument("--consensus-status", required=True)
    generate_parser.add_argument("--required-quorum", required=True)
    generate_parser.add_argument("--valid-attestation-count", required=True)
    generate_parser.add_argument("--invalid-attestation-count", required=True)
    generate_parser.add_argument("--replay-attestation-count", required=True)
    generate_parser.add_argument("--cadence", default="fast")
    generate_parser.add_argument("--runtime-seconds", default="0")
    generate_parser.add_argument("--max-seconds", default="90")
    generate_parser.add_argument("--evidence-complete", default="true")
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

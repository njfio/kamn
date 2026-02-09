#!/usr/bin/env python3
"""Reputation recovery reversal evidence generator and policy checker."""

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

SCHEMA_VERSION = "kamn.reputation.recovery-reversal-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
REVERSE_PENALTY_ACTION = "REVERSE_PENALTY"
HOLD_PENALTY_ACTION = "HOLD_PENALTY"
NON_NEGATIVE_INT_PATTERN = re.compile(r"^[0-9]+$")
DID_PATTERN = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")


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
    reviewer_did: str,
    pre_penalty_trust_score: int,
    post_penalty_trust_score: int,
    proposed_recovered_trust_score: int,
    max_reversal_points: int,
    false_positive_confirmed: bool,
    reviewer_quorum_satisfied: bool,
    audit_evidence_verified: str,
    replay_guard_passed: bool,
    ci_fast_gate: str,
) -> Mapping[str, bool]:
    reversal_points = proposed_recovered_trust_score - post_penalty_trust_score
    return {
        "did_fields_valid": bool(
            DID_PATTERN.match(subject_did) and DID_PATTERN.match(reviewer_did)
        ),
        "false_positive_confirmed": false_positive_confirmed,
        "reviewer_quorum_satisfied": reviewer_quorum_satisfied,
        "audit_evidence_verified": audit_evidence_verified == "PASS",
        "replay_guard_passed": replay_guard_passed,
        "reversal_within_limit": 0 <= reversal_points <= max_reversal_points,
        "restored_score_within_bounds": (
            0 <= pre_penalty_trust_score <= 1000
            and 0 <= post_penalty_trust_score <= 1000
            and 0 <= proposed_recovered_trust_score <= 1000
            and post_penalty_trust_score
            <= proposed_recovered_trust_score
            <= pre_penalty_trust_score
        ),
        "ci_fast_gate_passed": ci_fast_gate == "PASS",
    }


def _compute_reason_codes(policy_checks: Mapping[str, bool]) -> list[str]:
    reason_codes: list[str] = []
    if not policy_checks["did_fields_valid"]:
        reason_codes.append("did_fields_invalid")
    if not policy_checks["false_positive_confirmed"]:
        reason_codes.append("false_positive_not_confirmed")
    if not policy_checks["reviewer_quorum_satisfied"]:
        reason_codes.append("reviewer_quorum_missing")
    if not policy_checks["audit_evidence_verified"]:
        reason_codes.append("audit_evidence_verification_failed")
    if not policy_checks["replay_guard_passed"]:
        reason_codes.append("replay_guard_nonce_reused")
    if not policy_checks["reversal_within_limit"]:
        reason_codes.append("reversal_exceeds_limit")
    if not policy_checks["restored_score_within_bounds"]:
        reason_codes.append("restored_score_out_of_bounds")
    if not policy_checks["ci_fast_gate_passed"]:
        reason_codes.append("ci_fast_gate_failed")
    return sorted(reason_codes)


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.lane,
        args.recovery_id,
        args.subject_did,
        args.reviewer_did,
        args.pre_penalty_trust_score,
        args.post_penalty_trust_score,
        args.proposed_recovered_trust_score,
        args.max_reversal_points,
        args.false_positive_confirmed,
        args.reviewer_quorum_satisfied,
        args.audit_evidence_verified,
        args.replay_guard_pass,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all recovery evidence bundle arguments are required")

    lane = _parse_lane(args.lane)
    pre_penalty_trust_score = _parse_non_negative_int(
        "pre_penalty_trust_score", args.pre_penalty_trust_score
    )
    post_penalty_trust_score = _parse_non_negative_int(
        "post_penalty_trust_score", args.post_penalty_trust_score
    )
    proposed_recovered_trust_score = _parse_non_negative_int(
        "proposed_recovered_trust_score", args.proposed_recovered_trust_score
    )
    max_reversal_points = _parse_non_negative_int(
        "max_reversal_points", args.max_reversal_points
    )
    false_positive_confirmed = _parse_bool(args.false_positive_confirmed)
    reviewer_quorum_satisfied = _parse_bool(args.reviewer_quorum_satisfied)
    replay_guard_passed = _parse_bool(args.replay_guard_pass)
    audit_evidence_verified = _parse_pass_fail(
        args.audit_evidence_verified,
        "audit-evidence-verified must be PASS or FAIL",
    )
    ci_fast_gate = _parse_pass_fail(
        args.ci_fast_gate,
        "ci-fast-gate must be PASS or FAIL",
    )

    reversal_points = proposed_recovered_trust_score - post_penalty_trust_score
    policy_checks = _compute_policy_checks(
        subject_did=args.subject_did,
        reviewer_did=args.reviewer_did,
        pre_penalty_trust_score=pre_penalty_trust_score,
        post_penalty_trust_score=post_penalty_trust_score,
        proposed_recovered_trust_score=proposed_recovered_trust_score,
        max_reversal_points=max_reversal_points,
        false_positive_confirmed=false_positive_confirmed,
        reviewer_quorum_satisfied=reviewer_quorum_satisfied,
        audit_evidence_verified=audit_evidence_verified,
        replay_guard_passed=replay_guard_passed,
        ci_fast_gate=ci_fast_gate,
    )

    is_go = all(policy_checks.values())
    final_decision = GO_DECISION if is_go else NO_GO_DECISION
    recovery_action = REVERSE_PENALTY_ACTION if is_go else HOLD_PENALTY_ACTION
    reason_codes = _compute_reason_codes(policy_checks)
    reason_key = f"reputation_recovery_reason_codes:{final_decision}:v1"
    evidence_key = f"reputation_recovery_reversal_contract:{lane}:v1"

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "lane": lane,
        "evidence_key": evidence_key,
        "reason_key": reason_key,
        "recovery_context": {
            "recovery_id": args.recovery_id,
            "subject_did": args.subject_did,
            "reviewer_did": args.reviewer_did,
        },
        "score_transition": {
            "pre_penalty_trust_score": pre_penalty_trust_score,
            "post_penalty_trust_score": post_penalty_trust_score,
            "proposed_recovered_trust_score": proposed_recovered_trust_score,
            "reversal_points": reversal_points,
            "max_reversal_points": max_reversal_points,
        },
        "recovery_controls": {
            "false_positive_confirmed": false_positive_confirmed,
            "reviewer_quorum_satisfied": reviewer_quorum_satisfied,
            "audit_evidence_verified": audit_evidence_verified,
            "replay_guard_passed": replay_guard_passed,
            "ci_fast_gate": ci_fast_gate,
        },
        "policy_checks": policy_checks,
        "reason_codes": reason_codes,
        "recovery_action": recovery_action,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"schema_version={SCHEMA_VERSION}")
    print(f"evidence_key={evidence_key}")
    print(f"reason_key={reason_key}")
    print(f"recovery_action={recovery_action}")
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
            "lane",
            "evidence_key",
            "reason_key",
            "recovery_context",
            "score_transition",
            "recovery_controls",
            "policy_checks",
            "reason_codes",
            "recovery_action",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unexpected schema_version for reputation recovery evidence bundle")

    lane = payload["lane"]
    if lane not in {"contract", "deep"}:
        fail("lane must be contract or deep")

    expected_evidence_key = f"reputation_recovery_reversal_contract:{lane}:v1"
    if payload["evidence_key"] != expected_evidence_key:
        fail(
            "evidence_key mismatch: "
            f"expected {expected_evidence_key}, found {payload['evidence_key']}"
        )

    recovery_context = payload["recovery_context"]
    if not isinstance(recovery_context, dict):
        fail("recovery_context must be an object")
    for field_name in ("recovery_id", "subject_did", "reviewer_did"):
        if field_name not in recovery_context:
            fail(f"recovery_context missing field: {field_name}")
        if not isinstance(recovery_context[field_name], str) or not recovery_context[field_name]:
            fail(f"recovery_context.{field_name} must be a non-empty string")

    score_transition = payload["score_transition"]
    if not isinstance(score_transition, dict):
        fail("score_transition must be an object")
    for field_name in (
        "pre_penalty_trust_score",
        "post_penalty_trust_score",
        "proposed_recovered_trust_score",
        "reversal_points",
        "max_reversal_points",
    ):
        if field_name not in score_transition:
            fail(f"score_transition missing field: {field_name}")
        if not isinstance(score_transition[field_name], int):
            fail(f"score_transition.{field_name} must be an integer")

    recovery_controls = payload["recovery_controls"]
    if not isinstance(recovery_controls, dict):
        fail("recovery_controls must be an object")
    for field_name in (
        "false_positive_confirmed",
        "reviewer_quorum_satisfied",
        "audit_evidence_verified",
        "replay_guard_passed",
        "ci_fast_gate",
    ):
        if field_name not in recovery_controls:
            fail(f"recovery_controls missing field: {field_name}")
    if not isinstance(recovery_controls["false_positive_confirmed"], bool):
        fail("recovery_controls.false_positive_confirmed must be boolean")
    if not isinstance(recovery_controls["reviewer_quorum_satisfied"], bool):
        fail("recovery_controls.reviewer_quorum_satisfied must be boolean")
    if recovery_controls["audit_evidence_verified"] not in {"PASS", "FAIL"}:
        fail("recovery_controls.audit_evidence_verified must be PASS or FAIL")
    if not isinstance(recovery_controls["replay_guard_passed"], bool):
        fail("recovery_controls.replay_guard_passed must be boolean")
    if recovery_controls["ci_fast_gate"] not in {"PASS", "FAIL"}:
        fail("recovery_controls.ci_fast_gate must be PASS or FAIL")

    policy_checks = payload["policy_checks"]
    if not isinstance(policy_checks, dict):
        fail("policy_checks must be an object")
    required_checks = (
        "did_fields_valid",
        "false_positive_confirmed",
        "reviewer_quorum_satisfied",
        "audit_evidence_verified",
        "replay_guard_passed",
        "reversal_within_limit",
        "restored_score_within_bounds",
        "ci_fast_gate_passed",
    )
    for field_name in required_checks:
        if field_name not in policy_checks:
            fail(f"policy_checks missing field: {field_name}")
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

    pre_penalty = score_transition["pre_penalty_trust_score"]
    post_penalty = score_transition["post_penalty_trust_score"]
    proposed_recovered = score_transition["proposed_recovered_trust_score"]
    reversal_points = score_transition["reversal_points"]
    max_reversal_points = score_transition["max_reversal_points"]
    derived_reversal_points = proposed_recovered - post_penalty
    if reversal_points != derived_reversal_points:
        fail(
            "score_transition.reversal_points does not match derived score delta: "
            f"expected {derived_reversal_points}, found {reversal_points}"
        )

    derived_checks = _compute_policy_checks(
        subject_did=recovery_context["subject_did"],
        reviewer_did=recovery_context["reviewer_did"],
        pre_penalty_trust_score=pre_penalty,
        post_penalty_trust_score=post_penalty,
        proposed_recovered_trust_score=proposed_recovered,
        max_reversal_points=max_reversal_points,
        false_positive_confirmed=recovery_controls["false_positive_confirmed"],
        reviewer_quorum_satisfied=recovery_controls["reviewer_quorum_satisfied"],
        audit_evidence_verified=recovery_controls["audit_evidence_verified"],
        replay_guard_passed=recovery_controls["replay_guard_passed"],
        ci_fast_gate=recovery_controls["ci_fast_gate"],
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

    expected_action = (
        REVERSE_PENALTY_ACTION if actual_decision == GO_DECISION else HOLD_PENALTY_ACTION
    )
    recovery_action = payload["recovery_action"]
    if recovery_action not in {REVERSE_PENALTY_ACTION, HOLD_PENALTY_ACTION}:
        fail("recovery_action must be REVERSE_PENALTY or HOLD_PENALTY")
    if recovery_action != expected_action:
        fail(
            "recovery_action mismatch: "
            f"expected {expected_action}, found {recovery_action}"
        )

    reason_key = payload["reason_key"]
    if not isinstance(reason_key, str) or not reason_key:
        fail("reason_key must be a non-empty string")
    expected_reason_key = f"reputation_recovery_reason_codes:{actual_decision}:v1"
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
    print(f"recovery_action={recovery_action}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Reputation recovery reversal evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--lane")
    generate.add_argument("--recovery-id")
    generate.add_argument("--subject-did")
    generate.add_argument("--reviewer-did")
    generate.add_argument("--pre-penalty-trust-score")
    generate.add_argument("--post-penalty-trust-score")
    generate.add_argument("--proposed-recovered-trust-score")
    generate.add_argument("--max-reversal-points")
    generate.add_argument("--false-positive-confirmed")
    generate.add_argument("--reviewer-quorum-satisfied")
    generate.add_argument("--audit-evidence-verified")
    generate.add_argument("--replay-guard-pass")
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

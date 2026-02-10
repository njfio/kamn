#!/usr/bin/env python3
"""Reputation dispute evidence generator and policy checker."""

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

SCHEMA_VERSION = "kamn.reputation.dispute-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
REASON_CODES = {"QUALITY", "DELIVERY", "ABUSE", "IDENTITY"}
NON_NEGATIVE_INT_PATTERN = re.compile(r"^[0-9]+$")
DID_PATTERN = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")
HASH_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")


def _parse_bool(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("boolean fields must be true or false")


def _parse_reason_code(raw_value: str) -> str:
    if raw_value in REASON_CODES:
        return raw_value
    fail("dispute-reason-code must be QUALITY, DELIVERY, ABUSE, or IDENTITY")


def _parse_int(raw_value: str) -> int:
    if not NON_NEGATIVE_INT_PATTERN.fullmatch(raw_value):
        fail("score field must be an integer")
    return int(raw_value)


def _compute_policy_checks(
    *,
    subject_did: str,
    reviewer_did: str,
    evidence_uri: str,
    evidence_sha256: str,
    evidence_hash_verified: str,
    original_trust_score: int,
    proposed_trust_score: int,
    max_adjustment_points: int,
    policy_window_open: bool,
    approval_recorded: bool,
    ci_fast_gate: str,
) -> Mapping[str, bool]:
    score_delta = abs(proposed_trust_score - original_trust_score)
    return {
        "did_fields_valid": bool(
            DID_PATTERN.match(subject_did) and DID_PATTERN.match(reviewer_did)
        ),
        "evidence_uri_present": len(evidence_uri.strip()) > 0,
        "evidence_hash_valid": bool(HASH_PATTERN.match(evidence_sha256)),
        "evidence_hash_matches": evidence_hash_verified == "PASS",
        "trust_scores_in_range": (
            0 <= original_trust_score <= 1000 and 0 <= proposed_trust_score <= 1000
        ),
        "score_adjustment_within_limit": score_delta <= max_adjustment_points,
        "policy_window_satisfied": policy_window_open,
        "approval_satisfied": approval_recorded,
        "ci_fast_gate_passed": ci_fast_gate == "PASS",
    }


def _compute_reason_codes(policy_checks: Mapping[str, bool]) -> list[str]:
    reason_codes: list[str] = []
    if not policy_checks["did_fields_valid"]:
        reason_codes.append("did_fields_invalid")
    if not policy_checks["evidence_uri_present"]:
        reason_codes.append("evidence_uri_missing")
    if not policy_checks["evidence_hash_valid"]:
        reason_codes.append("evidence_hash_invalid")
    if not policy_checks["evidence_hash_matches"]:
        reason_codes.append("evidence_hash_verification_failed")
    if not policy_checks["trust_scores_in_range"]:
        reason_codes.append("trust_score_out_of_bounds")
    if not policy_checks["score_adjustment_within_limit"]:
        reason_codes.append("score_adjustment_exceeds_limit")
    if not policy_checks["policy_window_satisfied"]:
        reason_codes.append("policy_window_closed")
    if not policy_checks["approval_satisfied"]:
        reason_codes.append("approval_missing")
    if not policy_checks["ci_fast_gate_passed"]:
        reason_codes.append("ci_fast_gate_failed")
    return sorted(reason_codes)


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.dispute_id,
        args.subject_did,
        args.reviewer_did,
        args.dispute_reason_code,
        args.evidence_uri,
        args.evidence_sha256,
        args.evidence_hash_verified,
        args.original_trust_score,
        args.proposed_trust_score,
        args.max_adjustment_points,
        args.policy_window_open,
        args.approval_recorded,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all bundle arguments are required")

    dispute_reason_code = _parse_reason_code(args.dispute_reason_code)
    original_trust_score = _parse_int(args.original_trust_score)
    proposed_trust_score = _parse_int(args.proposed_trust_score)
    max_adjustment_points = _parse_int(args.max_adjustment_points)
    policy_window_open = _parse_bool(args.policy_window_open)
    approval_recorded = _parse_bool(args.approval_recorded)
    if args.evidence_hash_verified not in {"PASS", "FAIL"} or args.ci_fast_gate not in {
        "PASS",
        "FAIL",
    }:
        fail("evidence-hash-verified and ci-fast-gate must be PASS or FAIL")

    score_delta = abs(proposed_trust_score - original_trust_score)
    policy_checks = _compute_policy_checks(
        subject_did=args.subject_did,
        reviewer_did=args.reviewer_did,
        evidence_uri=args.evidence_uri,
        evidence_sha256=args.evidence_sha256,
        evidence_hash_verified=args.evidence_hash_verified,
        original_trust_score=original_trust_score,
        proposed_trust_score=proposed_trust_score,
        max_adjustment_points=max_adjustment_points,
        policy_window_open=policy_window_open,
        approval_recorded=approval_recorded,
        ci_fast_gate=args.ci_fast_gate,
    )

    final_decision = GO_DECISION if all(policy_checks.values()) else NO_GO_DECISION
    reason_key = f"reputation_dispute_reason_codes:{final_decision}:v1"
    reason_codes = _compute_reason_codes(policy_checks)

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "dispute_id": args.dispute_id,
        "subject_did": args.subject_did,
        "reviewer_did": args.reviewer_did,
        "dispute_reason_code": dispute_reason_code,
        "evidence_bundle": {
            "uri": args.evidence_uri,
            "sha256": args.evidence_sha256,
            "hash_verified": args.evidence_hash_verified,
        },
        "score_transition": {
            "original_trust_score": original_trust_score,
            "proposed_trust_score": proposed_trust_score,
            "score_delta": score_delta,
            "max_adjustment_points": max_adjustment_points,
        },
        "policy_window_open": policy_window_open,
        "approval_recorded": approval_recorded,
        "ci_fast_gate": args.ci_fast_gate,
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
            "dispute_id",
            "subject_did",
            "reviewer_did",
            "dispute_reason_code",
            "evidence_bundle",
            "score_transition",
            "policy_window_open",
            "approval_recorded",
            "ci_fast_gate",
            "reason_key",
            "policy_checks",
            "reason_codes",
            "final_decision",
        ),
    )

    if payload["dispute_reason_code"] not in REASON_CODES:
        fail("dispute_reason_code must be QUALITY, DELIVERY, ABUSE, or IDENTITY")

    for field_name in ("policy_window_open", "approval_recorded"):
        if not isinstance(payload[field_name], bool):
            fail(f"{field_name} must be boolean")

    if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    reason_key = payload["reason_key"]
    if not isinstance(reason_key, str) or not reason_key:
        fail("reason_key must be a non-empty string")

    evidence_bundle = payload["evidence_bundle"]
    if not isinstance(evidence_bundle, dict):
        fail("evidence_bundle must be an object")
    for field_name in ("uri", "sha256", "hash_verified"):
        if field_name not in evidence_bundle:
            fail(f"missing evidence_bundle field: {field_name}")
    if evidence_bundle["hash_verified"] not in {"PASS", "FAIL"}:
        fail("evidence_bundle.hash_verified must be PASS or FAIL")
    if not isinstance(evidence_bundle["uri"], str):
        fail("evidence_bundle.uri must be a string")
    if not isinstance(evidence_bundle["sha256"], str):
        fail("evidence_bundle.sha256 must be a string")

    score_transition = payload["score_transition"]
    if not isinstance(score_transition, dict):
        fail("score_transition must be an object")
    for field_name in (
        "original_trust_score",
        "proposed_trust_score",
        "score_delta",
        "max_adjustment_points",
    ):
        if field_name not in score_transition:
            fail(f"missing score_transition field: {field_name}")
        if not isinstance(score_transition[field_name], int):
            fail(f"score_transition.{field_name} must be an integer")

    policy_checks = payload["policy_checks"]
    if not isinstance(policy_checks, dict):
        fail("policy_checks must be an object")
    for field_name in (
        "did_fields_valid",
        "evidence_uri_present",
        "evidence_hash_valid",
        "evidence_hash_matches",
        "trust_scores_in_range",
        "score_adjustment_within_limit",
        "policy_window_satisfied",
        "approval_satisfied",
        "ci_fast_gate_passed",
    ):
        if field_name not in policy_checks:
            fail(f"missing policy_checks field: {field_name}")
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

    subject_did = str(payload["subject_did"])
    reviewer_did = str(payload["reviewer_did"])
    original_score = score_transition["original_trust_score"]
    proposed_score = score_transition["proposed_trust_score"]
    score_delta = score_transition["score_delta"]
    max_adjustment_points = score_transition["max_adjustment_points"]
    derived_score_delta = abs(proposed_score - original_score)
    if score_delta != derived_score_delta:
        fail(
            "score_transition.score_delta does not match derived score delta: "
            f"expected {derived_score_delta}, found {score_delta}"
        )

    derived_checks = _compute_policy_checks(
        subject_did=subject_did,
        reviewer_did=reviewer_did,
        evidence_uri=evidence_bundle["uri"],
        evidence_sha256=evidence_bundle["sha256"],
        evidence_hash_verified=evidence_bundle["hash_verified"],
        original_trust_score=original_score,
        proposed_trust_score=proposed_score,
        max_adjustment_points=max_adjustment_points,
        policy_window_open=payload["policy_window_open"],
        approval_recorded=payload["approval_recorded"],
        ci_fast_gate=payload["ci_fast_gate"],
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

    expected_reason_key = f"reputation_dispute_reason_codes:{actual_decision}:v1"
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
    print(f"final_decision={actual_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Reputation dispute evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--dispute-id")
    generate.add_argument("--subject-did")
    generate.add_argument("--reviewer-did")
    generate.add_argument("--dispute-reason-code")
    generate.add_argument("--evidence-uri")
    generate.add_argument("--evidence-sha256")
    generate.add_argument("--evidence-hash-verified")
    generate.add_argument("--original-trust-score")
    generate.add_argument("--proposed-trust-score")
    generate.add_argument("--max-adjustment-points")
    generate.add_argument("--policy-window-open")
    generate.add_argument("--approval-recorded")
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

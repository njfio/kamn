#!/usr/bin/env python3
"""DID multikey algorithm policy conformance contract."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    require_enum,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.did.multikey-algorithm-policy-report.v1"
EVIDENCE_KEY = "did_multikey_algorithm_policy_contract:evidence:v1"
REASON_PREFIX = "did_multikey_algorithm_policy_reason_codes"


def _load_fixture(path: Path) -> list[dict[str, Any]]:
    if not path.is_file():
        fail(f"fixture file not found: {path}")

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"fixture file is not valid JSON: {exc}")

    if not isinstance(payload, list):
        fail("fixture payload must be an array")

    vectors: list[dict[str, Any]] = []
    seen_vector_ids: set[str] = set()
    for index, item in enumerate(payload):
        if not isinstance(item, dict):
            fail(f"fixture vector at index {index} must be an object")
        require_keys(
            item,
            (
                "vector_id",
                "current_algorithms",
                "target_algorithms",
                "expect_allowed",
            ),
        )

        vector_id = item["vector_id"]
        current_algorithms = item["current_algorithms"]
        target_algorithms = item["target_algorithms"]
        expect_allowed = item["expect_allowed"]

        if not isinstance(vector_id, str) or not vector_id.strip():
            fail(f"fixture vector at index {index} has invalid vector_id")
        if vector_id in seen_vector_ids:
            fail(f"fixture vector ids must be unique: {vector_id}")
        seen_vector_ids.add(vector_id)

        if not isinstance(current_algorithms, list) or not current_algorithms:
            fail(f"fixture vector {vector_id} current_algorithms must be a non-empty array")
        if not isinstance(target_algorithms, list) or not target_algorithms:
            fail(f"fixture vector {vector_id} target_algorithms must be a non-empty array")
        if not all(isinstance(value, str) for value in current_algorithms):
            fail(f"fixture vector {vector_id} current_algorithms must contain strings")
        if not all(isinstance(value, str) for value in target_algorithms):
            fail(f"fixture vector {vector_id} target_algorithms must contain strings")
        if not isinstance(expect_allowed, bool):
            fail(f"fixture vector {vector_id} expect_allowed must be boolean")

        expected_reason = item.get("expected_reason")
        if expected_reason is not None and not isinstance(expected_reason, str):
            fail(f"fixture vector {vector_id} expected_reason must be a string")

        vectors.append(
            {
                "vector_id": vector_id,
                "current_algorithms": current_algorithms,
                "target_algorithms": target_algorithms,
                "expect_allowed": expect_allowed,
                "expected_reason": expected_reason,
            }
        )

    if not vectors:
        fail("fixture payload must contain at least one vector")
    return vectors


def _validate_algorithm_set(algorithms: list[str]) -> tuple[bool, str | None, str | None]:
    if not algorithms:
        return False, None, "empty_algorithm_set"

    normalized = [value.strip() for value in algorithms]
    if any(not value for value in normalized):
        return False, None, "empty_algorithm_entry"

    allowed_algorithms = {"Multikey", "MultikeyV2"}
    unsupported = [value for value in normalized if value not in allowed_algorithms]
    if unsupported:
        return False, None, "unsupported_algorithm"

    first = normalized[0]
    if any(value != first for value in normalized):
        return False, None, "mixed_algorithms"

    return True, first, None


def _evaluate_vector(vector: dict[str, Any]) -> dict[str, Any]:
    source_ok, source_algorithm, source_reason = _validate_algorithm_set(
        vector["current_algorithms"]
    )
    target_ok, target_algorithm, target_reason = _validate_algorithm_set(vector["target_algorithms"])

    actual_allowed = False
    actual_reason = "policy_blocked"
    if source_ok and target_ok:
        if source_algorithm == "MultikeyV2" and target_algorithm == "Multikey":
            actual_allowed = False
            actual_reason = "downgrade_blocked"
        else:
            actual_allowed = True
            actual_reason = "allowed_transition"
    elif source_reason is not None:
        actual_reason = source_reason
    elif target_reason is not None:
        actual_reason = target_reason

    expect_allowed = vector["expect_allowed"]
    expected_reason = vector.get("expected_reason")
    matches_expectation = actual_allowed == expect_allowed
    if not expect_allowed and expected_reason is not None:
        matches_expectation = matches_expectation and actual_reason == expected_reason

    return {
        "vector_id": vector["vector_id"],
        "current_algorithms": vector["current_algorithms"],
        "target_algorithms": vector["target_algorithms"],
        "expect_allowed": expect_allowed,
        "expected_reason": expected_reason,
        "actual_allowed": actual_allowed,
        "actual_reason": actual_reason,
        "matches_expectation": matches_expectation,
    }


def _evaluate_vectors(vectors: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    vector_results: list[dict[str, Any]] = []
    mismatch_vector_ids: list[str] = []
    allowed_vectors = 0
    blocked_vectors = 0

    for vector in vectors:
        result = _evaluate_vector(vector)
        if result["expect_allowed"]:
            allowed_vectors += 1
        else:
            blocked_vectors += 1
        if not result["matches_expectation"]:
            mismatch_vector_ids.append(result["vector_id"])
        vector_results.append(result)

    summary = {
        "total_vectors": len(vectors),
        "allowed_vectors": allowed_vectors,
        "blocked_vectors": blocked_vectors,
        "mismatch_vectors": len(mismatch_vector_ids),
        "mismatch_vector_ids": sorted(mismatch_vector_ids),
    }
    return vector_results, summary


def _compute_policy_checks(summary: dict[str, Any], ci_fast_gate: str) -> dict[str, bool]:
    return {
        "fixture_loaded": summary["total_vectors"] > 0,
        "has_allowed_case": summary["allowed_vectors"] > 0,
        "has_blocked_case": summary["blocked_vectors"] > 0,
        "all_vectors_match_expectation": summary["mismatch_vectors"] == 0,
        "ci_fast_gate_passed": ci_fast_gate == "PASS",
    }


def _reason_messages() -> dict[str, str]:
    return {
        "fixture_loaded": "multikey algorithm fixture must load",
        "has_allowed_case": "multikey algorithm fixture must include allowed transitions",
        "has_blocked_case": "multikey algorithm fixture must include blocked transitions",
        "all_vectors_match_expectation": "multikey algorithm vectors diverged from expected outcomes",
        "ci_fast_gate_passed": "ci-fast-gate-failed",
    }


def _compute_decision(policy_checks: dict[str, bool]) -> tuple[str, list[str], list[str]]:
    decision = DecisionAccumulator()
    reason_messages = _reason_messages()
    for key in reason_messages:
        decision.reject_if(not policy_checks[key], reason_messages[key])
    final_decision, decision_reasons = decision.finalize(
        "multikey algorithm policy vectors satisfied"
    )
    failed_checks = sorted([key for key, passed in policy_checks.items() if not passed])
    return final_decision, decision_reasons, failed_checks


def _reason_key(final_decision: str) -> str:
    return f"{REASON_PREFIX}:{final_decision}:v1"


def generate_bundle(args: argparse.Namespace) -> int:
    ci_fast_gate = require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    fixture_file = Path(args.fixture).resolve()
    vectors = _load_fixture(fixture_file)
    vector_results, summary = _evaluate_vectors(vectors)
    policy_checks = _compute_policy_checks(summary, ci_fast_gate)
    final_decision, decision_reasons, failed_checks = _compute_decision(policy_checks)
    reason_key = _reason_key(final_decision)

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "evidence_key": EVIDENCE_KEY,
        "fixture_file": str(fixture_file),
        "ci_fast_gate": ci_fast_gate,
        "vector_results": vector_results,
        "summary": summary,
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

    try:
        payload = json.loads(bundle_file.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"bundle file is not valid JSON: {exc}")
    if not isinstance(payload, dict):
        fail("bundle payload must be an object")

    require_keys(
        payload,
        (
            "schema_version",
            "generated_at",
            "evidence_key",
            "fixture_file",
            "ci_fast_gate",
            "vector_results",
            "summary",
            "policy_checks",
            "failed_checks",
            "decision_reasons",
            "reason_key",
            "final_decision",
        ),
    )
    if payload["schema_version"] != SCHEMA_VERSION:
        fail(f"schema_version must be {SCHEMA_VERSION}")
    if payload["evidence_key"] != EVIDENCE_KEY:
        fail("evidence_key mismatch")
    if not isinstance(payload["fixture_file"], str) or not payload["fixture_file"]:
        fail("fixture_file must be a non-empty string")
    ci_fast_gate = require_enum("ci_fast_gate", payload["ci_fast_gate"], ("PASS", "FAIL"))

    fixture_file = Path(payload["fixture_file"])
    vectors = _load_fixture(fixture_file)
    expected_vector_results, expected_summary = _evaluate_vectors(vectors)
    expected_policy_checks = _compute_policy_checks(expected_summary, ci_fast_gate)
    expected_decision, expected_reasons, expected_failed_checks = _compute_decision(
        expected_policy_checks
    )
    expected_reason_key = _reason_key(expected_decision)

    if payload["vector_results"] != expected_vector_results:
        fail("vector_results mismatch against evaluated fixture")
    if payload["summary"] != expected_summary:
        fail("summary mismatch against evaluated fixture")
    if payload["policy_checks"] != expected_policy_checks:
        fail("policy_checks mismatch against evaluated fixture")
    if payload["failed_checks"] != expected_failed_checks:
        fail("failed_checks mismatch against evaluated fixture")
    if payload["decision_reasons"] != expected_reasons:
        fail("decision_reasons mismatch against evaluated fixture")

    final_decision = require_enum("final_decision", payload["final_decision"], ("GO", "NO-GO"))
    if final_decision != expected_decision:
        fail(
            f"policy decision mismatch: expected {expected_decision}, found {final_decision}"
        )

    reason_key = payload["reason_key"]
    if reason_key != expected_reason_key:
        fail(f"reason_key mismatch: expected {expected_reason_key}, found {reason_key}")

    failed_checks_value = ",".join(expected_failed_checks) if expected_failed_checks else "none"
    print("status=validated")
    print(f"bundle_file={bundle_file}")
    print(f"reason_key={expected_reason_key}")
    print(f"final_decision={expected_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate/check DID multikey algorithm policy conformance evidence."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate_parser = subparsers.add_parser("generate", help="Generate evidence bundle")
    generate_parser.add_argument("--output-file", required=True)
    generate_parser.add_argument("--fixture", required=True)
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

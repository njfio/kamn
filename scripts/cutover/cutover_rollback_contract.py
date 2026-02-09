#!/usr/bin/env python3
"""Cutover rollback evidence generator and policy checker."""

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
    require_object,
    write_json,
)

SCHEMA_VERSION = "kamn.cutover.rollback-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_bool(field_name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"{field_name} must be true or false")


def _parse_rollback_trigger_status(raw_value: str) -> str:
    if raw_value in {"CLEAR", "TRIGGERED"}:
        return raw_value
    fail("--rollback-trigger-status must be CLEAR or TRIGGERED")


def _parse_checkpoint_state(raw_value: str) -> str:
    if raw_value in {"READY", "FAILED"}:
        return raw_value
    fail("--checkpoint-state must be READY or FAILED")


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("--ci-fast-gate must be PASS or FAIL")


def _compute_decision_reasons(
    *,
    ci_fast_gate: str,
    evidence_complete: bool,
    rollback_hash_match: bool,
    rollback_trigger_status: str,
    failed_checkpoint_id: str | None,
    checkpoint_state: str,
) -> list[str]:
    decision_reasons: list[str] = []
    if ci_fast_gate != "PASS":
        decision_reasons.append("ci-fast-gate-failed")
    if not evidence_complete:
        decision_reasons.append("incomplete-evidence")
    if not rollback_hash_match:
        decision_reasons.append("rollback target hash mismatch")
    if rollback_trigger_status == "TRIGGERED" and not failed_checkpoint_id:
        decision_reasons.append("missing failed checkpoint evidence")
    if rollback_trigger_status == "TRIGGERED" and checkpoint_state != "FAILED":
        decision_reasons.append("trigger-state-checkpoint-mismatch")
    if rollback_trigger_status == "CLEAR" and checkpoint_state != "READY":
        decision_reasons.append("clear-trigger-requires-ready-checkpoint")
    return decision_reasons


def generate_bundle(args: argparse.Namespace) -> int:
    rollback_trigger_status = _parse_rollback_trigger_status(args.rollback_trigger_status)
    checkpoint_state = _parse_checkpoint_state(args.checkpoint_state)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)
    evidence_complete = _parse_bool("evidence_complete", args.evidence_complete)
    failed_checkpoint_id = args.failed_checkpoint_id or None
    rollback_hash_match = args.rollback_target_hash == args.post_rollback_hash

    decision_reasons = _compute_decision_reasons(
        ci_fast_gate=ci_fast_gate,
        evidence_complete=evidence_complete,
        rollback_hash_match=rollback_hash_match,
        rollback_trigger_status=rollback_trigger_status,
        failed_checkpoint_id=failed_checkpoint_id,
        checkpoint_state=checkpoint_state,
    )
    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "cutover_manifest_id": args.cutover_manifest_id,
        "rollback": {
            "trigger_status": rollback_trigger_status,
            "checkpoint_state": checkpoint_state,
            "failed_checkpoint_id": failed_checkpoint_id,
            "rollback_target_hash": args.rollback_target_hash,
            "post_rollback_hash": args.post_rollback_hash,
            "rollback_hash_match": rollback_hash_match,
            "evidence_complete": evidence_complete,
            "ci_fast_gate": ci_fast_gate,
        },
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    print(f"rollback_hash_match={str(rollback_hash_match).lower()}")
    print(f"evidence_complete={str(evidence_complete).lower()}")
    return 0


def _require_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str):
        fail(f"{field_name} must be a string")
    return value


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
            "cutover_manifest_id",
            "rollback",
            "decision_reasons",
            "final_decision",
        ),
    )

    if payload.get("schema_version") != SCHEMA_VERSION:
        fail("unexpected rollback evidence schema_version")

    cutover_manifest_id = payload.get("cutover_manifest_id")
    if not isinstance(cutover_manifest_id, str) or not cutover_manifest_id.strip():
        fail("cutover_manifest_id must be a non-empty string")

    rollback = require_object(payload, "rollback")
    for field_name in (
        "trigger_status",
        "checkpoint_state",
        "failed_checkpoint_id",
        "rollback_target_hash",
        "post_rollback_hash",
        "rollback_hash_match",
        "evidence_complete",
        "ci_fast_gate",
    ):
        if field_name not in rollback:
            fail(f"missing rollback field: {field_name}")

    trigger_status = rollback.get("trigger_status")
    if trigger_status not in {"CLEAR", "TRIGGERED"}:
        fail("rollback.trigger_status must be CLEAR or TRIGGERED")

    checkpoint_state = rollback.get("checkpoint_state")
    if checkpoint_state not in {"READY", "FAILED"}:
        fail("rollback.checkpoint_state must be READY or FAILED")

    ci_fast_gate = rollback.get("ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("rollback.ci_fast_gate must be PASS or FAIL")

    failed_checkpoint_id = rollback.get("failed_checkpoint_id")
    if failed_checkpoint_id is not None and not isinstance(failed_checkpoint_id, str):
        fail("rollback.failed_checkpoint_id must be a string or null")

    rollback_target_hash = _require_string(rollback, "rollback_target_hash")
    post_rollback_hash = _require_string(rollback, "post_rollback_hash")

    rollback_hash_match = rollback.get("rollback_hash_match")
    if not isinstance(rollback_hash_match, bool):
        fail("rollback.rollback_hash_match must be a boolean")

    evidence_complete = rollback.get("evidence_complete")
    if not isinstance(evidence_complete, bool):
        fail("rollback.evidence_complete must be a boolean")

    derived_hash_match = rollback_target_hash == post_rollback_hash
    if rollback_hash_match != derived_hash_match:
        fail(
            "rollback target hash mismatch: "
            f"declared rollback_hash_match={rollback_hash_match} "
            f"but hashes compare as {derived_hash_match}"
        )

    decision_reasons = _compute_decision_reasons(
        ci_fast_gate=ci_fast_gate,
        evidence_complete=evidence_complete,
        rollback_hash_match=rollback_hash_match,
        rollback_trigger_status=trigger_status,
        failed_checkpoint_id=failed_checkpoint_id,
        checkpoint_state=checkpoint_state,
    )

    expected_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    actual_decision = payload.get("final_decision")
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        reasons = ", ".join(decision_reasons) or "all rollback gates satisfied"
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}; "
            f"reasons={reasons}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"trigger_status={trigger_status}")
    print(f"rollback_hash_match={str(rollback_hash_match).lower()}")
    print(f"evidence_complete={str(evidence_complete).lower()}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Cutover rollback evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--cutover-manifest-id", required=True)
    generate.add_argument("--rollback-trigger-status", required=True)
    generate.add_argument("--checkpoint-state", required=True)
    generate.add_argument("--failed-checkpoint-id", required=True)
    generate.add_argument("--rollback-target-hash", required=True)
    generate.add_argument("--post-rollback-hash", required=True)
    generate.add_argument("--evidence-complete", required=True)
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

#!/usr/bin/env python3
"""Governance simulation evidence generator and policy checker."""

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
    require_object,
    write_json,
)

SCHEMA_VERSION = "kamn.governance.simulation-veto.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
SIMULATION_HASH_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")


def _parse_bool(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("boolean fields must be true or false")


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("ci-fast-gate must be PASS or FAIL")


def _parse_int(field_name: str, raw_value: str) -> int:
    try:
        return int(raw_value)
    except ValueError:
        fail(f"{field_name} must be an integer")


def _compute_policy_checks(
    *, simulation_hash: str, required_approvals: int, received_approvals: int
) -> Mapping[str, bool]:
    return {
        "simulation_hash_valid": bool(SIMULATION_HASH_PATTERN.match(simulation_hash)),
        "approval_quorum_met": received_approvals >= required_approvals,
    }


def _compute_reason_codes(
    *,
    simulation_complete: bool,
    simulation_hash_valid: bool,
    veto_window_open: bool,
    veto_recorded: bool,
    timelock_expired: bool,
    approval_quorum_met: bool,
    ci_fast_gate: str,
) -> list[str]:
    reason_codes: list[str] = []
    if not simulation_complete:
        reason_codes.append("simulation_missing")
    if not simulation_hash_valid:
        reason_codes.append("simulation_hash_invalid")
    if veto_window_open:
        reason_codes.append("veto_window_open")
    if veto_recorded:
        reason_codes.append("veto_recorded")
    if not timelock_expired:
        reason_codes.append("timelock_not_expired")
    if not approval_quorum_met:
        reason_codes.append("approval_quorum_missing")
    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")
    return reason_codes


def generate_bundle(args: argparse.Namespace) -> int:
    simulation_complete = _parse_bool(args.simulation_complete)
    veto_window_open = _parse_bool(args.veto_window_open)
    veto_recorded = _parse_bool(args.veto_recorded)
    timelock_expired = _parse_bool(args.timelock_expired)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)

    required_approvals = _parse_int("required-approvals", args.required_approvals)
    received_approvals = _parse_int("received-approvals", args.received_approvals)
    if required_approvals < 1:
        fail("required-approvals must be >= 1")
    if received_approvals < 0:
        fail("received-approvals must be >= 0")

    policy_checks = _compute_policy_checks(
        simulation_hash=args.simulation_hash,
        required_approvals=required_approvals,
        received_approvals=received_approvals,
    )
    simulation_hash_valid = policy_checks["simulation_hash_valid"]
    approval_quorum_met = policy_checks["approval_quorum_met"]

    is_go = (
        simulation_complete
        and simulation_hash_valid
        and not veto_window_open
        and not veto_recorded
        and timelock_expired
        and approval_quorum_met
        and ci_fast_gate == "PASS"
    )
    final_decision = GO_DECISION if is_go else NO_GO_DECISION

    reason_codes = _compute_reason_codes(
        simulation_complete=simulation_complete,
        simulation_hash_valid=simulation_hash_valid,
        veto_window_open=veto_window_open,
        veto_recorded=veto_recorded,
        timelock_expired=timelock_expired,
        approval_quorum_met=approval_quorum_met,
        ci_fast_gate=ci_fast_gate,
    )

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "proposal_id": args.proposal_id,
        "simulation_hash": args.simulation_hash,
        "simulation_complete": simulation_complete,
        "veto_window_open": veto_window_open,
        "veto_recorded": veto_recorded,
        "timelock_expired": timelock_expired,
        "approvals": {
            "required": required_approvals,
            "received": received_approvals,
        },
        "ci_fast_gate": ci_fast_gate,
        "policy_checks": policy_checks,
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    return 0


def _require_bool(payload: Mapping[str, Any], field_name: str) -> bool:
    value = payload.get(field_name)
    if not isinstance(value, bool):
        fail(f"{field_name} must be boolean")
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
            "proposal_id",
            "simulation_hash",
            "simulation_complete",
            "veto_window_open",
            "veto_recorded",
            "timelock_expired",
            "approvals",
            "ci_fast_gate",
            "policy_checks",
            "reason_codes",
            "final_decision",
        ),
    )

    simulation_complete = _require_bool(payload, "simulation_complete")
    veto_window_open = _require_bool(payload, "veto_window_open")
    veto_recorded = _require_bool(payload, "veto_recorded")
    timelock_expired = _require_bool(payload, "timelock_expired")

    ci_fast_gate = payload.get("ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    approvals = require_object(payload, "approvals")
    if "required" not in approvals:
        fail("missing approvals field: required")
    if "received" not in approvals:
        fail("missing approvals field: received")

    required_approvals = approvals["required"]
    received_approvals = approvals["received"]
    if not isinstance(required_approvals, int):
        fail("approvals.required must be an integer")
    if not isinstance(received_approvals, int):
        fail("approvals.received must be an integer")
    if required_approvals < 1:
        fail("approvals.required must be >= 1")
    if received_approvals < 0:
        fail("approvals.received must be >= 0")

    policy_checks = require_object(payload, "policy_checks")
    if "simulation_hash_valid" not in policy_checks:
        fail("missing policy_checks field: simulation_hash_valid")
    if "approval_quorum_met" not in policy_checks:
        fail("missing policy_checks field: approval_quorum_met")

    for field_name in ("simulation_hash_valid", "approval_quorum_met"):
        if not isinstance(policy_checks[field_name], bool):
            fail(f"policy_checks.{field_name} must be boolean")

    simulation_hash = str(payload.get("simulation_hash"))
    hash_valid = bool(SIMULATION_HASH_PATTERN.match(simulation_hash))
    approval_quorum_met = received_approvals >= required_approvals

    if policy_checks["simulation_hash_valid"] != hash_valid:
        fail("policy_checks.simulation_hash_valid does not match derived policy")
    if policy_checks["approval_quorum_met"] != approval_quorum_met:
        fail("policy_checks.approval_quorum_met does not match derived policy")

    expected_go = (
        simulation_complete
        and hash_valid
        and not veto_window_open
        and not veto_recorded
        and timelock_expired
        and approval_quorum_met
        and ci_fast_gate == "PASS"
    )
    expected_decision = GO_DECISION if expected_go else NO_GO_DECISION

    actual_decision = payload.get("final_decision")
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    failed_checks = _compute_reason_codes(
        simulation_complete=simulation_complete,
        simulation_hash_valid=hash_valid,
        veto_window_open=veto_window_open,
        veto_recorded=veto_recorded,
        timelock_expired=timelock_expired,
        approval_quorum_met=approval_quorum_met,
        ci_fast_gate=ci_fast_gate,
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
            "Governance simulation evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--proposal-id", required=True)
    generate.add_argument("--simulation-hash", required=True)
    generate.add_argument("--simulation-complete", required=True)
    generate.add_argument("--veto-window-open", required=True)
    generate.add_argument("--veto-recorded", required=True)
    generate.add_argument("--timelock-expired", required=True)
    generate.add_argument("--required-approvals", required=True)
    generate.add_argument("--received-approvals", required=True)
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

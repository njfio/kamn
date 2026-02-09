#!/usr/bin/env python3
"""Treasury disbursement evidence generator and policy checker."""

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
    require_pattern,
    require_string,
    write_json,
)

SCHEMA_VERSION = "kamn.treasury.disbursement-approval.v1"


def _parse_policy_window_open(raw_value: str) -> bool:
    if raw_value not in {"true", "false"}:
        fail("policy-window-open must be true or false")
    return raw_value == "true"


def generate_bundle(args: argparse.Namespace) -> int:
    require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    require_pattern(
        "asset-symbol",
        args.asset_symbol,
        r"[A-Z0-9]+",
        "asset-symbol must be uppercase alphanumeric",
    )
    policy_window_open = _parse_policy_window_open(args.policy_window_open)

    disbursement_amount = require_non_negative_int(
        "disbursement-amount", str(args.disbursement_amount)
    )
    daily_limit_amount = require_non_negative_int(
        "daily-limit-amount", str(args.daily_limit_amount)
    )
    required_approvals = require_non_negative_int(
        "required-approvals", str(args.required_approvals)
    )
    received_approvals = require_non_negative_int(
        "received-approvals", str(args.received_approvals)
    )

    decision = DecisionAccumulator()
    decision.reject_if(
        disbursement_amount <= 0, "disbursement amount must be greater than zero"
    )
    decision.reject_if(
        daily_limit_amount <= 0, "daily limit amount must be greater than zero"
    )
    decision.reject_if(
        disbursement_amount > daily_limit_amount,
        "disbursement amount exceeds daily limit amount",
    )
    decision.reject_if(
        required_approvals <= 0, "required approvals must be greater than zero"
    )
    decision.reject_if(
        received_approvals < required_approvals,
        "received approvals are below required approvals",
    )
    decision.reject_if(
        not args.approval_quorum_hash.startswith("sha256:")
        or len(args.approval_quorum_hash) <= len("sha256:"),
        "approval quorum hash must be a non-empty sha256 digest",
    )
    decision.reject_if(not policy_window_open, "policy approval window is closed")
    decision.reject_if(args.ci_fast_gate != "PASS", "ci-fast-gate-failed")

    final_decision, decision_reasons = decision.finalize(
        "all treasury disbursement approval gates satisfied"
    )
    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": generated_at,
        "disbursement": {
            "disbursement_id": args.disbursement_id,
            "treasury_account_id": args.treasury_account_id,
            "destination_account_id": args.destination_account_id,
            "asset_symbol": args.asset_symbol,
            "amount": disbursement_amount,
            "daily_limit_amount": daily_limit_amount,
        },
        "approvals": {
            "required": required_approvals,
            "received": received_approvals,
            "approval_quorum_hash": args.approval_quorum_hash,
        },
        "policy_window_open": policy_window_open,
        "ci_fast_gate": args.ci_fast_gate,
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    return 0


def _expected_decision(payload: Mapping[str, object]) -> str:
    disbursement = require_object(payload, "disbursement")
    approvals = require_object(payload, "approvals")

    require_string(disbursement, "disbursement_id")
    require_string(disbursement, "treasury_account_id")
    require_string(disbursement, "destination_account_id")
    asset_symbol = require_string(disbursement, "asset_symbol")
    require_pattern(
        "disbursement.asset_symbol",
        asset_symbol,
        r"[A-Z0-9]+",
        "disbursement.asset_symbol must be uppercase alphanumeric",
    )
    amount = require_int(disbursement, "amount", min_value=0)
    daily_limit_amount = require_int(disbursement, "daily_limit_amount", min_value=0)

    required_approvals = require_int(approvals, "required", min_value=0)
    received_approvals = require_int(approvals, "received", min_value=0)
    approval_quorum_hash = require_string(approvals, "approval_quorum_hash")

    policy_window_open = payload.get("policy_window_open")
    if not isinstance(policy_window_open, bool):
        fail("policy_window_open must be a boolean")

    ci_fast_gate = require_string(payload, "ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    decision_reasons = payload.get("decision_reasons")
    if not isinstance(decision_reasons, list):
        fail("decision_reasons must be an array")
    if not all(isinstance(entry, str) for entry in decision_reasons):
        fail("decision_reasons entries must be strings")

    expected_go = True
    if amount <= 0:
        expected_go = False
    if daily_limit_amount <= 0:
        expected_go = False
    if amount > daily_limit_amount:
        expected_go = False
    if required_approvals <= 0:
        expected_go = False
    if received_approvals < required_approvals:
        expected_go = False
    if not approval_quorum_hash.startswith("sha256:") or len(approval_quorum_hash) <= len(
        "sha256:"
    ):
        expected_go = False
    if not policy_window_open:
        expected_go = False
    if ci_fast_gate != "PASS":
        expected_go = False

    return "GO" if expected_go else "NO-GO"


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
            "disbursement",
            "approvals",
            "policy_window_open",
            "ci_fast_gate",
            "decision_reasons",
            "final_decision",
        ),
    )

    schema_version = require_string(payload, "schema_version")
    if schema_version != SCHEMA_VERSION:
        fail(
            "unexpected schema_version for treasury disbursement approval evidence bundle"
        )

    expected_decision = _expected_decision(payload)
    actual_decision = require_string(payload, "final_decision")
    if actual_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Treasury disbursement evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--disbursement-id", required=True)
    generate.add_argument("--treasury-account-id", required=True)
    generate.add_argument("--destination-account-id", required=True)
    generate.add_argument("--asset-symbol", required=True)
    generate.add_argument("--disbursement-amount", required=True)
    generate.add_argument("--daily-limit-amount", required=True)
    generate.add_argument("--required-approvals", required=True)
    generate.add_argument("--received-approvals", required=True)
    generate.add_argument("--approval-quorum-hash", required=True)
    generate.add_argument("--policy-window-open", required=True)
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


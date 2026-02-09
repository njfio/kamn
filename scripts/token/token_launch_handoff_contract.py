#!/usr/bin/env python3
"""Token launch handoff evidence generator and policy checker."""

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

SCHEMA_VERSION = "kamn.token.launch-handoff.v1"


def generate_bundle(args: argparse.Namespace) -> int:
    require_enum("ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    require_pattern(
        "token-symbol",
        args.token_symbol,
        r"[A-Z0-9]+",
        "token-symbol must be uppercase alphanumeric",
    )

    configured_total = require_non_negative_int(
        "configured-total-supply", str(args.configured_total_supply)
    )
    expected_total = require_non_negative_int(
        "expected-total-supply", str(args.expected_total_supply)
    )
    configured_allocation = require_non_negative_int(
        "configured-allocation-sum", str(args.configured_allocation_sum)
    )
    expected_allocation = require_non_negative_int(
        "expected-allocation-sum", str(args.expected_allocation_sum)
    )
    allocation_bucket_count = require_non_negative_int(
        "allocation-bucket-count", str(args.allocation_bucket_count)
    )
    expected_bucket_count = require_non_negative_int(
        "expected-bucket-count", str(args.expected_bucket_count)
    )
    required_approvals = require_non_negative_int(
        "required-approvals", str(args.required_approvals)
    )
    received_approvals = require_non_negative_int(
        "received-approvals", str(args.received_approvals)
    )

    decision = DecisionAccumulator()
    decision.reject_if(
        configured_total != expected_total,
        "configured total supply does not match expected total supply",
    )
    decision.reject_if(
        configured_allocation != expected_allocation,
        "configured allocation sum does not match expected allocation sum",
    )
    decision.reject_if(
        configured_allocation != configured_total,
        "configured allocation sum does not match configured total supply",
    )
    decision.reject_if(
        expected_allocation != expected_total,
        "expected allocation sum does not match expected total supply",
    )
    decision.reject_if(
        allocation_bucket_count != expected_bucket_count or allocation_bucket_count <= 0,
        "allocation bucket count mismatch",
    )
    decision.reject_if(
        required_approvals <= 0, "required approvals must be greater than zero"
    )
    decision.reject_if(
        received_approvals < required_approvals,
        "received approvals are below required approvals",
    )
    decision.reject_if(
        not args.genesis_hash.startswith("sha256:")
        or len(args.genesis_hash) <= len("sha256:"),
        "genesis hash must be a non-empty sha256 digest",
    )
    decision.reject_if(args.ci_fast_gate != "PASS", "ci-fast-gate-failed")

    final_decision, decision_reasons = decision.finalize(
        "all token launch handoff invariants satisfied"
    )
    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": generated_at,
        "token_symbol": args.token_symbol,
        "supply": {
            "configured_total_supply": configured_total,
            "expected_total_supply": expected_total,
        },
        "allocations": {
            "configured_sum": configured_allocation,
            "expected_sum": expected_allocation,
            "bucket_count": allocation_bucket_count,
            "expected_bucket_count": expected_bucket_count,
        },
        "genesis": {
            "genesis_hash": args.genesis_hash,
        },
        "approvals": {
            "required": required_approvals,
            "received": received_approvals,
        },
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
    supply = require_object(payload, "supply")
    allocations = require_object(payload, "allocations")
    genesis = require_object(payload, "genesis")
    approvals = require_object(payload, "approvals")

    configured_total_supply = require_int(
        supply, "configured_total_supply", min_value=0
    )
    expected_total_supply = require_int(supply, "expected_total_supply", min_value=0)
    configured_sum = require_int(allocations, "configured_sum", min_value=0)
    expected_sum = require_int(allocations, "expected_sum", min_value=0)
    bucket_count = require_int(allocations, "bucket_count", min_value=0)
    expected_bucket_count = require_int(
        allocations, "expected_bucket_count", min_value=0
    )
    genesis_hash = require_string(genesis, "genesis_hash")
    required_approvals = require_int(approvals, "required", min_value=0)
    received_approvals = require_int(approvals, "received", min_value=0)

    token_symbol = require_string(payload, "token_symbol")
    require_pattern(
        "token_symbol",
        token_symbol,
        r"[A-Z0-9]+",
        "token_symbol must be uppercase alphanumeric",
    )

    ci_fast_gate = require_string(payload, "ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    decision_reasons = payload.get("decision_reasons")
    if not isinstance(decision_reasons, list):
        fail("decision_reasons must be an array")
    if not all(isinstance(entry, str) for entry in decision_reasons):
        fail("decision_reasons entries must be strings")

    expected_go = True
    if configured_total_supply != expected_total_supply:
        expected_go = False
    if configured_sum != expected_sum:
        expected_go = False
    if configured_sum != configured_total_supply:
        expected_go = False
    if expected_sum != expected_total_supply:
        expected_go = False
    if bucket_count != expected_bucket_count or bucket_count <= 0:
        expected_go = False
    if required_approvals <= 0:
        expected_go = False
    if received_approvals < required_approvals:
        expected_go = False
    if not genesis_hash.startswith("sha256:") or len(genesis_hash) <= len("sha256:"):
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
            "token_symbol",
            "supply",
            "allocations",
            "genesis",
            "approvals",
            "ci_fast_gate",
            "decision_reasons",
            "final_decision",
        ),
    )

    schema_version = require_string(payload, "schema_version")
    if schema_version != SCHEMA_VERSION:
        fail("unexpected schema_version for token launch handoff evidence bundle")

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
            "Token launch handoff evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--token-symbol", required=True)
    generate.add_argument("--configured-total-supply", required=True)
    generate.add_argument("--expected-total-supply", required=True)
    generate.add_argument("--configured-allocation-sum", required=True)
    generate.add_argument("--expected-allocation-sum", required=True)
    generate.add_argument("--allocation-bucket-count", required=True)
    generate.add_argument("--expected-bucket-count", required=True)
    generate.add_argument("--genesis-hash", required=True)
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

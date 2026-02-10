#!/usr/bin/env python3
"""Live-network pilot artifact summary generator contract."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402


def _require_non_negative_int(field_name: str, raw_value: str) -> int:
    if not raw_value.isdigit():
        fail(f"{field_name} must be a non-negative integer")
    return int(raw_value)


def _require_bool(field_name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"{field_name} must be true or false")


def generate_summary(args: argparse.Namespace) -> int:
    if not args.output_file:
        fail("all arguments are required")

    for name, value in (
        ("--event-name", args.event_name),
        ("--cadence", args.cadence),
        ("--smoke-status", args.smoke_status),
        ("--smoke-decision", args.smoke_decision),
        ("--smoke-elapsed-seconds", args.smoke_elapsed_seconds),
        ("--deep-status", args.deep_status),
        ("--deep-decision", args.deep_decision),
        ("--deep-elapsed-seconds", args.deep_elapsed_seconds),
        ("--budget-status", args.budget_status),
        ("--evidence-complete", args.evidence_complete),
        ("--ci-fast-gate", args.ci_fast_gate),
    ):
        if not value:
            fail("all arguments are required")

    if args.event_name not in {"schedule", "workflow_dispatch"}:
        fail("--event-name must be schedule or workflow_dispatch")

    if args.cadence not in {"scheduled", "manual"}:
        fail("--cadence must be scheduled or manual")

    if args.smoke_status not in {"pass", "fail"}:
        fail("--smoke-status must be pass or fail")

    if args.deep_status not in {"pass", "fail"}:
        fail("--deep-status must be pass or fail")

    if args.smoke_decision not in {"GO", "NO-GO"}:
        fail("--smoke-decision must be GO or NO-GO")

    if args.deep_decision not in {"GO", "NO-GO"}:
        fail("--deep-decision must be GO or NO-GO")

    if args.budget_status not in {"within", "exceeded"}:
        fail("--budget-status must be within or exceeded")

    if args.ci_fast_gate not in {"PASS", "FAIL"}:
        fail("--ci-fast-gate must be PASS or FAIL")

    smoke_elapsed_seconds = _require_non_negative_int(
        "smoke_elapsed_seconds", args.smoke_elapsed_seconds
    )
    deep_elapsed_seconds = _require_non_negative_int(
        "deep_elapsed_seconds", args.deep_elapsed_seconds
    )
    evidence_complete = _require_bool("evidence_complete", args.evidence_complete)

    decision_reasons: list[str] = []
    if args.smoke_status != "pass":
        decision_reasons.append("smoke_lane_failed")
    if args.smoke_decision != "GO":
        decision_reasons.append("smoke_decision_no_go")
    if args.deep_status != "pass":
        decision_reasons.append("deep_lane_failed")
    if args.deep_decision != "GO":
        decision_reasons.append("deep_decision_no_go")
    if args.budget_status != "within":
        decision_reasons.append("runtime_budget_exceeded")
    if not evidence_complete:
        decision_reasons.append("evidence_incomplete")
    if args.ci_fast_gate != "PASS":
        decision_reasons.append("ci_fast_gate_failed")
    if args.event_name not in {"schedule", "workflow_dispatch"}:
        decision_reasons.append("invalid_event")

    final_decision = "GO" if not decision_reasons else "NO-GO"

    payload = {
        "schema_version": "kamn.runtime.live-network-pilot-artifact-summary.v1",
        "event_name": args.event_name,
        "cadence": args.cadence,
        "smoke": {
            "status": args.smoke_status,
            "final_decision": args.smoke_decision,
            "elapsed_seconds": smoke_elapsed_seconds,
        },
        "deep": {
            "status": args.deep_status,
            "final_decision": args.deep_decision,
            "elapsed_seconds": deep_elapsed_seconds,
        },
        "budget_status": args.budget_status,
        "evidence_complete": evidence_complete,
        "ci_fast_gate": args.ci_fast_gate,
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    reason_codes = "none" if not decision_reasons else ",".join(decision_reasons)
    print("status=generated")
    print(f"summary_file={output_path}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={reason_codes}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Live-network pilot artifact summary generator."
    )
    parser.add_argument("--output-file")
    parser.add_argument("--event-name")
    parser.add_argument("--cadence")
    parser.add_argument("--smoke-status")
    parser.add_argument("--smoke-decision")
    parser.add_argument("--smoke-elapsed-seconds")
    parser.add_argument("--deep-status")
    parser.add_argument("--deep-decision")
    parser.add_argument("--deep-elapsed-seconds")
    parser.add_argument("--budget-status")
    parser.add_argument("--evidence-complete")
    parser.add_argument("--ci-fast-gate")
    parser.set_defaults(handler=generate_summary)
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

#!/usr/bin/env python3
"""Live-network pilot artifact summary policy checker."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json  # noqa: E402


def check_summary(args: argparse.Namespace) -> int:
    if not args.summary_file:
        fail("--summary-file is required")

    summary_path = Path(args.summary_file)
    if not summary_path.is_file():
        fail(f"summary file not found: {summary_path}")

    payload = load_json(summary_path)

    required_keys = {
        "schema_version",
        "event_name",
        "cadence",
        "smoke",
        "deep",
        "budget_status",
        "evidence_complete",
        "ci_fast_gate",
        "decision_reasons",
        "final_decision",
    }
    missing = sorted(required_keys.difference(payload.keys()))
    if missing:
        fail(f"missing required summary fields: {','.join(missing)}")

    if payload["schema_version"] != "kamn.runtime.live-network-pilot-artifact-summary.v1":
        fail("unexpected live-network pilot summary schema_version")

    event_name = payload["event_name"]
    if event_name not in {"schedule", "workflow_dispatch"}:
        fail("event_name must be schedule or workflow_dispatch")

    cadence = payload["cadence"]
    if cadence not in {"scheduled", "manual"}:
        fail("cadence must be scheduled or manual")

    if event_name == "schedule" and cadence != "scheduled":
        fail("schedule event must map to scheduled cadence")
    if event_name == "workflow_dispatch" and cadence != "manual":
        fail("workflow_dispatch event must map to manual cadence")

    for lane_key in ("smoke", "deep"):
        lane_payload = payload[lane_key]
        if not isinstance(lane_payload, dict):
            fail(f"{lane_key} payload must be an object")
        for required in ("status", "final_decision", "elapsed_seconds"):
            if required not in lane_payload:
                fail(f"{lane_key} payload missing {required}")
        if lane_payload["status"] not in {"pass", "fail"}:
            fail(f"{lane_key}.status must be pass or fail")
        if lane_payload["final_decision"] not in {"GO", "NO-GO"}:
            fail(f"{lane_key}.final_decision must be GO or NO-GO")
        if (
            not isinstance(lane_payload["elapsed_seconds"], int)
            or lane_payload["elapsed_seconds"] < 0
        ):
            fail(f"{lane_key}.elapsed_seconds must be a non-negative integer")

    budget_status = payload["budget_status"]
    if budget_status not in {"within", "exceeded"}:
        fail("budget_status must be within or exceeded")

    if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    if not isinstance(payload["evidence_complete"], bool):
        fail("evidence_complete must be a boolean")

    actual_reasons = payload["decision_reasons"]
    if not isinstance(actual_reasons, list) or not all(
        isinstance(value, str) for value in actual_reasons
    ):
        fail("decision_reasons must be an array of strings")

    expected_reasons: list[str] = []
    if payload["smoke"]["status"] != "pass":
        expected_reasons.append("smoke_lane_failed")
    if payload["smoke"]["final_decision"] != "GO":
        expected_reasons.append("smoke_decision_no_go")
    if payload["deep"]["status"] != "pass":
        expected_reasons.append("deep_lane_failed")
    if payload["deep"]["final_decision"] != "GO":
        expected_reasons.append("deep_decision_no_go")
    if budget_status != "within":
        expected_reasons.append("runtime_budget_exceeded")
    if payload["evidence_complete"] is False:
        expected_reasons.append("evidence_incomplete")
    if payload["ci_fast_gate"] != "PASS":
        expected_reasons.append("ci_fast_gate_failed")

    if actual_reasons != expected_reasons:
        fail(
            "decision_reasons mismatch: "
            f"expected {expected_reasons}, found {actual_reasons}"
        )

    expected_decision = "GO" if not expected_reasons else "NO-GO"
    actual_decision = payload["final_decision"]
    if actual_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            f"expected final_decision={expected_decision}, found {actual_decision}; "
            f"reasons={actual_reasons}"
        )

    failed_checks = "none" if not actual_reasons else ",".join(actual_reasons)
    print("status=ok")
    print(f"final_decision={actual_decision}")
    print(f"failed_checks={failed_checks}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Live-network pilot artifact summary policy checker."
    )
    parser.add_argument("--summary-file")
    parser.set_defaults(handler=check_summary)
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

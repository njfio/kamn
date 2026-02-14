#!/usr/bin/env python3
"""Go/no-go release evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    parse_int,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.release.gonogo.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
REQUIRED_EVIDENCE_MARKERS = (
    "ci_fast_gate",
    "ci_deep_lane",
    "rollback_precheck",
    "rollback_trigger_status",
    "approval_quorum",
    "runtime_image_digest",
)


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.release_candidate,
        args.schema_target_version,
        args.runtime_image_digest,
        args.ci_fast_gate,
        args.ci_deep_lane,
        args.rollback_precheck,
        args.rollback_trigger_status,
        args.required_approvals,
        args.received_approvals,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all bundle arguments are required")

    ci_fast_gate = args.ci_fast_gate
    ci_deep_lane = args.ci_deep_lane
    rollback_precheck = args.rollback_precheck
    for field_name, value in (
        ("ci-fast-gate", ci_fast_gate),
        ("ci-deep-lane", ci_deep_lane),
        ("rollback-precheck", rollback_precheck),
    ):
        if value not in {"PASS", "FAIL"}:
            fail(f"{field_name} must be PASS or FAIL")

    rollback_trigger_status = args.rollback_trigger_status
    if rollback_trigger_status not in {"CLEAR", "TRIGGERED"}:
        fail("rollback-trigger-status must be CLEAR or TRIGGERED")

    required_approvals = parse_int("required-approvals", args.required_approvals)
    received_approvals = parse_int("received-approvals", args.received_approvals)
    if required_approvals < 1:
        fail("required-approvals must be >= 1")
    if received_approvals < 0:
        fail("received-approvals must be >= 0")

    final_decision = (
        GO_DECISION
        if (
            ci_fast_gate == "PASS"
            and ci_deep_lane == "PASS"
            and rollback_precheck == "PASS"
            and rollback_trigger_status == "CLEAR"
            and received_approvals >= required_approvals
        )
        else NO_GO_DECISION
    )

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "release_candidate": args.release_candidate,
        "schema_target_version": args.schema_target_version,
        "runtime_image_digest": args.runtime_image_digest,
        "evidence_markers": list(REQUIRED_EVIDENCE_MARKERS),
        "gates": {
            "ci_fast_gate": ci_fast_gate,
            "ci_deep_lane": ci_deep_lane,
            "rollback_precheck": rollback_precheck,
        },
        "rollback_trigger_status": rollback_trigger_status,
        "approvals": {
            "required": required_approvals,
            "received": received_approvals,
        },
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
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
            "release_candidate",
            "schema_target_version",
            "runtime_image_digest",
            "evidence_markers",
            "gates",
            "rollback_trigger_status",
            "approvals",
            "final_decision",
        ),
    )

    gates = payload["gates"]
    if not isinstance(gates, dict):
        fail("bundle field 'gates' must be an object")

    for gate_name in ("ci_fast_gate", "ci_deep_lane", "rollback_precheck"):
        if gate_name not in gates:
            fail(f"missing gate field: {gate_name}")
        if gates[gate_name] not in {"PASS", "FAIL"}:
            fail(f"gate '{gate_name}' must be PASS or FAIL")

    rollback_trigger_status = payload["rollback_trigger_status"]
    if rollback_trigger_status not in {"CLEAR", "TRIGGERED"}:
        fail("rollback_trigger_status must be CLEAR or TRIGGERED")

    evidence_markers = payload["evidence_markers"]
    if not isinstance(evidence_markers, list):
        fail("bundle field 'evidence_markers' must be an array")
    if any(not isinstance(marker, str) or marker == "" for marker in evidence_markers):
        fail("evidence_markers entries must be non-empty strings")
    missing_required_markers = [
        marker for marker in REQUIRED_EVIDENCE_MARKERS if marker not in evidence_markers
    ]
    if missing_required_markers:
        fail(
            "missing required evidence markers: "
            + ",".join(sorted(set(missing_required_markers)))
        )

    approvals = payload["approvals"]
    if not isinstance(approvals, dict):
        fail("bundle field 'approvals' must be an object")
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

    expected_go = (
        gates["ci_fast_gate"] == "PASS"
        and gates["ci_deep_lane"] == "PASS"
        and gates["rollback_precheck"] == "PASS"
        and rollback_trigger_status == "CLEAR"
        and received_approvals >= required_approvals
    )
    expected_decision = GO_DECISION if expected_go else NO_GO_DECISION
    actual_decision = payload["final_decision"]
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"required_approvals={required_approvals}")
    print(f"received_approvals={received_approvals}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Go/no-go release evidence contract utilities (generate/check)."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--release-candidate")
    generate.add_argument("--schema-target-version")
    generate.add_argument("--runtime-image-digest")
    generate.add_argument("--ci-fast-gate")
    generate.add_argument("--ci-deep-lane")
    generate.add_argument("--rollback-precheck")
    generate.add_argument("--rollback-trigger-status")
    generate.add_argument("--required-approvals")
    generate.add_argument("--received-approvals")
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

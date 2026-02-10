#!/usr/bin/env python3
"""Bridge replay/redaction evidence generator and policy checker."""

from __future__ import annotations

import argparse
import json
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
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.bridge.replay-redaction-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_lane(raw_value: str) -> str:
    if raw_value in {"contract", "deep"}:
        return raw_value
    fail("lane must be contract or deep")


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("ci-fast-gate must be PASS or FAIL")


def _load_report(path: Path, report_name: str) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        fail(f"{report_name} report is not valid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{report_name} report must be a JSON object")
    return payload


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.lane,
        args.replay_report_file,
        args.redaction_report_file,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all bridge replay/redaction evidence bundle arguments are required")

    replay_report_path = Path(args.replay_report_file)
    redaction_report_path = Path(args.redaction_report_file)
    if not replay_report_path.is_file():
        fail(f"replay report file not found: {args.replay_report_file}")
    if not redaction_report_path.is_file():
        fail(f"redaction report file not found: {args.redaction_report_file}")

    lane = _parse_lane(args.lane)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)

    replay_payload = _load_report(replay_report_path, "replay")
    redaction_payload = _load_report(redaction_report_path, "redaction")

    replay_status = str(replay_payload.get("status", ""))
    try:
        case_count = int(replay_payload.get("case_count", 0))
    except (TypeError, ValueError):
        fail("replay report case_count must be an integer")
    try:
        failed_count = int(replay_payload.get("failed_count", 0))
    except (TypeError, ValueError):
        fail("replay report failed_count must be an integer")

    requested_suites = replay_payload.get("requested_suites", [])
    if not isinstance(requested_suites, list):
        fail("replay report requested_suites must be an array")
    failed_case_ids = replay_payload.get("failed_case_ids", [])
    if not isinstance(failed_case_ids, list):
        fail("replay report failed_case_ids must be an array")

    redaction_status = str(redaction_payload.get("status", ""))
    redaction_mode = str(redaction_payload.get("mode", ""))
    connectors = redaction_payload.get("connectors", [])
    if not isinstance(connectors, list):
        fail("redaction report connectors must be an array")
    connector_count = len(connectors)

    leaked_connectors = redaction_payload.get("leaked_connectors", [])
    if leaked_connectors is None:
        leaked_connectors = []
    if not isinstance(leaked_connectors, list):
        fail("redaction report leaked_connectors must be an array")
    leaked_connector_count = len(leaked_connectors)

    decision_reasons: list[str] = []
    if replay_status != "pass":
        decision_reasons.append("bridge replay matrix status is not pass")
    if case_count <= 0:
        decision_reasons.append("bridge replay matrix must include at least one case")
    if failed_count > 0:
        decision_reasons.append("bridge replay matrix reported failed cases")
    if redaction_status != "pass":
        decision_reasons.append("bridge redaction checker status is not pass")
    if connector_count <= 0:
        decision_reasons.append("bridge redaction report must include connector samples")
    if leaked_connector_count > 0:
        decision_reasons.append("bridge redaction report indicates leaked connectors")

    expected_redaction_mode = "contract" if lane == "contract" else "deep"
    if redaction_mode != expected_redaction_mode:
        decision_reasons.append(
            f"redaction mode mismatch: expected {expected_redaction_mode}, got {redaction_mode}"
        )

    if lane == "contract" and ci_fast_gate != "PASS":
        decision_reasons.append("ci-fast-gate-failed")

    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    if not decision_reasons:
        decision_reasons.append(
            "all bridge replay/redaction evidence invariants satisfied"
        )

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "lane": lane,
        "replay": {
            "status": replay_status,
            "case_count": case_count,
            "failed_count": failed_count,
            "requested_suites": requested_suites,
            "failed_case_ids": failed_case_ids,
        },
        "redaction": {
            "status": redaction_status,
            "mode": redaction_mode,
            "connector_count": connector_count,
            "leaked_connector_count": leaked_connector_count,
            "leaked_connectors": leaked_connectors,
        },
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
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
            "lane",
            "replay",
            "redaction",
            "ci_fast_gate",
            "decision_reasons",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unexpected schema_version for bridge replay/redaction evidence bundle")

    lane = payload["lane"]
    if lane not in {"contract", "deep"}:
        fail("lane must be contract or deep")

    if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    decision_reasons = payload["decision_reasons"]
    if not isinstance(decision_reasons, list) or not all(
        isinstance(item, str) for item in decision_reasons
    ):
        fail("decision_reasons must be an array of strings")

    replay = payload["replay"]
    if not isinstance(replay, dict):
        fail("replay must be an object")
    for field_name in (
        "status",
        "case_count",
        "failed_count",
        "requested_suites",
        "failed_case_ids",
    ):
        if field_name not in replay:
            fail(f"replay missing field: {field_name}")
    if replay["status"] not in {"pass", "fail"}:
        fail("replay.status must be pass or fail")
    if not isinstance(replay["case_count"], int):
        fail("replay.case_count must be an integer")
    if not isinstance(replay["failed_count"], int):
        fail("replay.failed_count must be an integer")
    if not isinstance(replay["requested_suites"], list):
        fail("replay.requested_suites must be an array")
    if not isinstance(replay["failed_case_ids"], list):
        fail("replay.failed_case_ids must be an array")

    redaction = payload["redaction"]
    if not isinstance(redaction, dict):
        fail("redaction must be an object")
    for field_name in (
        "status",
        "mode",
        "connector_count",
        "leaked_connector_count",
        "leaked_connectors",
    ):
        if field_name not in redaction:
            fail(f"redaction missing field: {field_name}")
    if redaction["status"] not in {"pass", "fail"}:
        fail("redaction.status must be pass or fail")
    if redaction["mode"] not in {"contract", "deep"}:
        fail("redaction.mode must be contract or deep")
    if not isinstance(redaction["connector_count"], int):
        fail("redaction.connector_count must be an integer")
    if not isinstance(redaction["leaked_connector_count"], int):
        fail("redaction.leaked_connector_count must be an integer")
    if not isinstance(redaction["leaked_connectors"], list):
        fail("redaction.leaked_connectors must be an array")

    expected_go = True
    if replay["status"] != "pass":
        expected_go = False
    if replay["case_count"] <= 0:
        expected_go = False
    if replay["failed_count"] > 0:
        expected_go = False
    if redaction["status"] != "pass":
        expected_go = False
    if redaction["connector_count"] <= 0:
        expected_go = False
    if redaction["leaked_connector_count"] > 0:
        expected_go = False
    if lane == "contract" and payload["ci_fast_gate"] != "PASS":
        expected_go = False
    if lane == "contract" and redaction["mode"] != "contract":
        expected_go = False
    if lane == "deep" and redaction["mode"] != "deep":
        expected_go = False

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
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Bridge replay/redaction evidence contract utilities (generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--lane")
    generate.add_argument("--replay-report-file")
    generate.add_argument("--redaction-report-file")
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

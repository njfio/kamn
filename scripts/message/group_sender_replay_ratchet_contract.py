#!/usr/bin/env python3
"""Group sender replay/ratchet evidence generator and policy checker."""

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

SCHEMA_VERSION = "kamn.group-sender.replay-ratchet-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _load_report(report_path: Path) -> dict[str, Any]:
    try:
        report = json.loads(report_path.read_text())
    except json.JSONDecodeError as error:
        fail(f"report file is not valid JSON: {error}")

    if not isinstance(report, dict):
        fail("report payload must be a JSON object")
    return report


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.lane,
        args.report_file,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all group sender replay/ratchet evidence bundle arguments are required")

    lane = args.lane
    if lane not in {"contract", "deep"}:
        fail("lane must be contract or deep")

    ci_fast_gate = args.ci_fast_gate
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci-fast-gate must be PASS or FAIL")

    report_path = Path(args.report_file)
    if not report_path.is_file():
        fail(f"report file not found: {report_path}")

    report = _load_report(report_path)
    status = str(report.get("status", ""))
    nonce_replay_detected = report.get("nonce_replay_detected")
    stale_generation_detected = report.get("stale_generation_detected")
    signature_tamper_detected = report.get("signature_tamper_detected")
    reason_codes = report.get("reason_codes", [])

    if status not in {"pass", "fail"}:
        fail("report.status must be pass or fail")
    if not isinstance(nonce_replay_detected, bool):
        fail("report.nonce_replay_detected must be a boolean")
    if not isinstance(stale_generation_detected, bool):
        fail("report.stale_generation_detected must be a boolean")
    if not isinstance(signature_tamper_detected, bool):
        fail("report.signature_tamper_detected must be a boolean")
    if not isinstance(reason_codes, list) or not all(
        isinstance(item, str) and item for item in reason_codes
    ):
        fail("report.reason_codes must be an array of non-empty strings")
    reason_codes = sorted(reason_codes)

    decision_reasons: list[str] = []
    if status != "pass":
        decision_reasons.append("replay_ratchet_status_not_pass")
    if nonce_replay_detected:
        decision_reasons.append("nonce_replay_detected")
    if stale_generation_detected:
        decision_reasons.append("stale_generation_payload_detected")
    if signature_tamper_detected:
        decision_reasons.append("signature_tamper_detected")
    if lane == "contract" and ci_fast_gate != "PASS":
        decision_reasons.append("ci_fast_gate_failed")
    if not reason_codes:
        decision_reasons.append("reason_codes_missing")

    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    if not decision_reasons:
        decision_reasons.append("all group sender replay/ratchet checks passed")

    evidence_key = f"group_sender_replay_ratchet:{lane}:v1"
    reason_key = f"group_sender_replay_ratchet_reason:{final_decision}:v1"

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "lane": lane,
        "evidence_key": evidence_key,
        "reason_key": reason_key,
        "report": {
            "status": status,
            "nonce_replay_detected": nonce_replay_detected,
            "stale_generation_detected": stale_generation_detected,
            "signature_tamper_detected": signature_tamper_detected,
            "reason_codes": reason_codes,
        },
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"schema_version={SCHEMA_VERSION}")
    print(f"evidence_key={evidence_key}")
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
            "lane",
            "evidence_key",
            "reason_key",
            "report",
            "ci_fast_gate",
            "decision_reasons",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unexpected schema_version for group sender replay/ratchet evidence bundle")

    lane = payload["lane"]
    if lane not in {"contract", "deep"}:
        fail("lane must be contract or deep")

    expected_evidence_key = f"group_sender_replay_ratchet:{lane}:v1"
    if payload["evidence_key"] != expected_evidence_key:
        fail(
            "evidence_key mismatch: "
            f"expected {expected_evidence_key}, found {payload['evidence_key']}"
        )

    ci_fast_gate = payload["ci_fast_gate"]
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    decision_reasons = payload["decision_reasons"]
    if not isinstance(decision_reasons, list) or not all(
        isinstance(item, str) and item for item in decision_reasons
    ):
        fail("decision_reasons must be an array of non-empty strings")

    report = payload["report"]
    if not isinstance(report, dict):
        fail("report must be an object")
    for field_name in (
        "status",
        "nonce_replay_detected",
        "stale_generation_detected",
        "signature_tamper_detected",
        "reason_codes",
    ):
        if field_name not in report:
            fail(f"report missing field: {field_name}")

    if report["status"] not in {"pass", "fail"}:
        fail("report.status must be pass or fail")
    if not isinstance(report["nonce_replay_detected"], bool):
        fail("report.nonce_replay_detected must be a boolean")
    if not isinstance(report["stale_generation_detected"], bool):
        fail("report.stale_generation_detected must be a boolean")
    if not isinstance(report["signature_tamper_detected"], bool):
        fail("report.signature_tamper_detected must be a boolean")
    if not isinstance(report["reason_codes"], list) or not all(
        isinstance(item, str) and item for item in report["reason_codes"]
    ):
        fail("report.reason_codes must be an array of non-empty strings")
    if report["reason_codes"] != sorted(report["reason_codes"]):
        fail("report.reason_codes must be sorted and deterministic")

    expected_go = True
    if report["status"] != "pass":
        expected_go = False
    if report["nonce_replay_detected"]:
        expected_go = False
    if report["stale_generation_detected"]:
        expected_go = False
    if report["signature_tamper_detected"]:
        expected_go = False
    if lane == "contract" and ci_fast_gate != "PASS":
        expected_go = False
    if not report["reason_codes"]:
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

    expected_reason_key = f"group_sender_replay_ratchet_reason:{actual_decision}:v1"
    if payload["reason_key"] != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {payload['reason_key']}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"schema_version={payload['schema_version']}")
    print(f"evidence_key={payload['evidence_key']}")
    print(f"reason_key={payload['reason_key']}")
    print(f"final_decision={actual_decision}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Group sender replay/ratchet evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--lane")
    generate.add_argument("--report-file")
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

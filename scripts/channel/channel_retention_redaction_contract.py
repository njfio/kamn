#!/usr/bin/env python3
"""Channel retention/redaction evidence generator and policy checker."""

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

SCHEMA_VERSION = "kamn.channel.retention-redaction-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_reason_codes(payload: dict[str, Any], field_prefix: str) -> list[str]:
    reason_codes = payload.get("reason_codes", [])
    if not isinstance(reason_codes, list):
        fail(f"{field_prefix}.reason_codes must be an array")
    if not all(isinstance(item, str) and item for item in reason_codes):
        fail(f"{field_prefix}.reason_codes must contain non-empty strings")
    return sorted(reason_codes)


def _parse_lane(raw_value: str) -> str:
    if raw_value in {"contract", "deep"}:
        return raw_value
    fail("lane must be contract or deep")


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("ci-fast-gate must be PASS or FAIL")


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.lane,
        args.retention_report_file,
        args.redaction_report_file,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all channel retention/redaction evidence bundle arguments are required")

    if not Path(args.retention_report_file).is_file():
        fail(f"retention report file not found: {args.retention_report_file}")
    if not Path(args.redaction_report_file).is_file():
        fail(f"redaction report file not found: {args.redaction_report_file}")

    lane = _parse_lane(args.lane)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)

    try:
        retention_payload = json.loads(Path(args.retention_report_file).read_text())
    except json.JSONDecodeError as exc:
        fail(f"retention report is not valid JSON: {exc}")
    try:
        redaction_payload = json.loads(Path(args.redaction_report_file).read_text())
    except json.JSONDecodeError as exc:
        fail(f"redaction report is not valid JSON: {exc}")

    retention_status = str(retention_payload.get("status", ""))
    retention_total_candidates = retention_payload.get("total_candidates")
    retention_replay_safe = retention_payload.get("replay_safe")
    retention_reason_codes = _parse_reason_codes(retention_payload, "retention")

    if retention_status not in {"pass", "fail"}:
        fail("retention.status must be pass or fail")
    if not isinstance(retention_total_candidates, int):
        fail("retention.total_candidates must be an integer")
    if not isinstance(retention_replay_safe, bool):
        fail("retention.replay_safe must be a boolean")

    redaction_status = str(redaction_payload.get("status", ""))
    redaction_applied_count = redaction_payload.get("applied_count")
    redaction_replay_safe = redaction_payload.get("replay_safe")
    redaction_reason_codes = _parse_reason_codes(redaction_payload, "redaction")

    if redaction_status not in {"pass", "fail"}:
        fail("redaction.status must be pass or fail")
    if not isinstance(redaction_applied_count, int):
        fail("redaction.applied_count must be an integer")
    if not isinstance(redaction_replay_safe, bool):
        fail("redaction.replay_safe must be a boolean")

    decision_reasons: list[str] = []
    if retention_status != "pass":
        decision_reasons.append("retention_status_not_pass")
    if redaction_status != "pass":
        decision_reasons.append("redaction_status_not_pass")
    if not retention_replay_safe:
        decision_reasons.append("retention_replay_safe_false")
    if not redaction_replay_safe:
        decision_reasons.append("redaction_replay_safe_false")
    if lane == "contract" and ci_fast_gate != "PASS":
        decision_reasons.append("ci_fast_gate_failed")
    if not retention_reason_codes:
        decision_reasons.append("retention_reason_codes_missing")
    if not redaction_reason_codes:
        decision_reasons.append("redaction_reason_codes_missing")

    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    if not decision_reasons:
        decision_reasons.append(
            "all channel retention/redaction evidence invariants satisfied"
        )

    combined_reason_codes = sorted(set(retention_reason_codes + redaction_reason_codes))
    evidence_key = f"channel_retention_redaction:{lane}:v1"
    reason_key = f"channel_retention_redaction_reason:{final_decision}:v1"

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "lane": lane,
        "evidence_key": evidence_key,
        "reason_key": reason_key,
        "retention": {
            "status": retention_status,
            "total_candidates": retention_total_candidates,
            "replay_safe": retention_replay_safe,
            "reason_codes": retention_reason_codes,
        },
        "redaction": {
            "status": redaction_status,
            "applied_count": redaction_applied_count,
            "replay_safe": redaction_replay_safe,
            "reason_codes": redaction_reason_codes,
        },
        "combined_reason_codes": combined_reason_codes,
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
            "retention",
            "redaction",
            "combined_reason_codes",
            "ci_fast_gate",
            "decision_reasons",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unexpected schema_version for channel retention/redaction evidence bundle")

    lane = payload["lane"]
    if lane not in {"contract", "deep"}:
        fail("lane must be contract or deep")

    expected_evidence_key = f"channel_retention_redaction:{lane}:v1"
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

    retention = payload["retention"]
    if not isinstance(retention, dict):
        fail("retention must be an object")
    for field_name in ("status", "total_candidates", "replay_safe", "reason_codes"):
        if field_name not in retention:
            fail(f"retention missing field: {field_name}")
    if retention["status"] not in {"pass", "fail"}:
        fail("retention.status must be pass or fail")
    if not isinstance(retention["total_candidates"], int):
        fail("retention.total_candidates must be an integer")
    if not isinstance(retention["replay_safe"], bool):
        fail("retention.replay_safe must be a boolean")
    if not isinstance(retention["reason_codes"], list):
        fail("retention.reason_codes must be an array")
    if retention["reason_codes"] != sorted(retention["reason_codes"]):
        fail("retention.reason_codes must be sorted and deterministic")
    if not all(isinstance(item, str) and item for item in retention["reason_codes"]):
        fail("retention.reason_codes must contain non-empty strings")

    redaction = payload["redaction"]
    if not isinstance(redaction, dict):
        fail("redaction must be an object")
    for field_name in ("status", "applied_count", "replay_safe", "reason_codes"):
        if field_name not in redaction:
            fail(f"redaction missing field: {field_name}")
    if redaction["status"] not in {"pass", "fail"}:
        fail("redaction.status must be pass or fail")
    if not isinstance(redaction["applied_count"], int):
        fail("redaction.applied_count must be an integer")
    if not isinstance(redaction["replay_safe"], bool):
        fail("redaction.replay_safe must be a boolean")
    if not isinstance(redaction["reason_codes"], list):
        fail("redaction.reason_codes must be an array")
    if redaction["reason_codes"] != sorted(redaction["reason_codes"]):
        fail("redaction.reason_codes must be sorted and deterministic")
    if not all(isinstance(item, str) and item for item in redaction["reason_codes"]):
        fail("redaction.reason_codes must contain non-empty strings")

    combined_reason_codes = payload["combined_reason_codes"]
    if not isinstance(combined_reason_codes, list) or not all(
        isinstance(item, str) and item for item in combined_reason_codes
    ):
        fail("combined_reason_codes must be an array of non-empty strings")
    if combined_reason_codes != sorted(set(combined_reason_codes)):
        fail("combined_reason_codes must be sorted unique deterministic values")

    expected_go = True
    if retention["status"] != "pass":
        expected_go = False
    if redaction["status"] != "pass":
        expected_go = False
    if not retention["replay_safe"]:
        expected_go = False
    if not redaction["replay_safe"]:
        expected_go = False
    if lane == "contract" and ci_fast_gate != "PASS":
        expected_go = False
    if not retention["reason_codes"]:
        expected_go = False
    if not redaction["reason_codes"]:
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

    expected_reason_key = f"channel_retention_redaction_reason:{actual_decision}:v1"
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
            "Channel retention/redaction evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--lane")
    generate.add_argument("--retention-report-file")
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

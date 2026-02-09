#!/usr/bin/env python3
"""SOC2 control evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import re
import sys
from typing import Mapping

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

SCHEMA_VERSION = "kamn.compliance.soc2-control-evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
DATE_PATTERN = re.compile(r"^\d{4}-\d{2}-\d{2}$")
SHA256_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")


def _parse_status(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("check statuses must be PASS or FAIL")


def _derive_checks(
    *,
    audit_period_start: str,
    audit_period_end: str,
    evidence_sha256: str,
    tamper_check: str,
    completeness_check: str,
    ci_fast_gate: str,
) -> Mapping[str, bool | str]:
    period_valid = (
        bool(DATE_PATTERN.match(audit_period_start))
        and bool(DATE_PATTERN.match(audit_period_end))
        and audit_period_start <= audit_period_end
    )
    hash_valid = bool(SHA256_PATTERN.match(evidence_sha256))

    return {
        "tamper": tamper_check,
        "completeness": completeness_check,
        "ci_fast_gate": ci_fast_gate,
        "period_valid": period_valid,
        "hash_valid": hash_valid,
    }


def _is_go(checks: Mapping[str, bool | str]) -> bool:
    return (
        checks["tamper"] == "PASS"
        and checks["completeness"] == "PASS"
        and checks["ci_fast_gate"] == "PASS"
        and bool(checks["period_valid"])
        and bool(checks["hash_valid"])
    )


def generate_bundle(args: argparse.Namespace) -> int:
    checks = _derive_checks(
        audit_period_start=args.audit_period_start,
        audit_period_end=args.audit_period_end,
        evidence_sha256=args.evidence_sha256,
        tamper_check=_parse_status(args.tamper_check),
        completeness_check=_parse_status(args.completeness_check),
        ci_fast_gate=_parse_status(args.ci_fast_gate),
    )

    final_decision = GO_DECISION if _is_go(checks) else NO_GO_DECISION
    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "control_id": args.control_id,
        "audit_period": {
            "start": args.audit_period_start,
            "end": args.audit_period_end,
        },
        "collector_did": args.collector_did,
        "evidence_uri": args.evidence_uri,
        "evidence_sha256": args.evidence_sha256,
        "checks": checks,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    return 0


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
            "control_id",
            "audit_period",
            "collector_did",
            "evidence_uri",
            "evidence_sha256",
            "checks",
            "final_decision",
        ),
    )

    audit_period = require_object(payload, "audit_period")
    for field_name in ("start", "end"):
        if field_name not in audit_period:
            fail(f"missing audit_period field: {field_name}")
        if not isinstance(audit_period[field_name], str):
            fail(f"audit_period.{field_name} must be a string")

    checks = require_object(payload, "checks")
    for field_name in ("tamper", "completeness", "ci_fast_gate"):
        if field_name not in checks:
            fail(f"missing checks field: {field_name}")
        if checks[field_name] not in {"PASS", "FAIL"}:
            fail(f"checks.{field_name} must be PASS or FAIL")

    for field_name in ("period_valid", "hash_valid"):
        if field_name not in checks:
            fail(f"missing checks field: {field_name}")
        if not isinstance(checks[field_name], bool):
            fail(f"checks.{field_name} must be boolean")

    audit_start = audit_period["start"]
    audit_end = audit_period["end"]
    date_values_valid = bool(DATE_PATTERN.match(audit_start)) and bool(
        DATE_PATTERN.match(audit_end)
    )
    period_valid = date_values_valid and audit_start <= audit_end and checks["period_valid"]

    evidence_sha256 = str(payload.get("evidence_sha256"))
    hash_valid = bool(SHA256_PATTERN.match(evidence_sha256))
    if checks["hash_valid"] != hash_valid:
        fail("checks.hash_valid must match evidence_sha256 format validation")

    expected_go = (
        checks["tamper"] == "PASS"
        and checks["completeness"] == "PASS"
        and checks["ci_fast_gate"] == "PASS"
        and period_valid
        and hash_valid
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

    failed_checks: list[str] = []
    if checks["tamper"] != "PASS":
        failed_checks.append("tamper")
    if checks["completeness"] != "PASS":
        failed_checks.append("completeness")
    if checks["ci_fast_gate"] != "PASS":
        failed_checks.append("ci_fast_gate")
    if not period_valid:
        failed_checks.append("period_valid")
    if not hash_valid:
        failed_checks.append("hash_valid")

    failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"failed_checks={failed_checks_value}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "SOC2 control evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--control-id", required=True)
    generate.add_argument("--audit-period-start", required=True)
    generate.add_argument("--audit-period-end", required=True)
    generate.add_argument("--collector-did", required=True)
    generate.add_argument("--evidence-uri", required=True)
    generate.add_argument("--evidence-sha256", required=True)
    generate.add_argument("--tamper-check", required=True)
    generate.add_argument("--completeness-check", required=True)
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

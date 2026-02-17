#!/usr/bin/env python3
"""Declarative policy checker for contract-lane report validation."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json, write_json  # noqa: E402

POLICY_SCHEMA_VERSION = "kamn.framework.declarative-policy.v1"
OUTPUT_SCHEMA_VERSION = "kamn.framework.declarative-policy-report.v1"

_MISSING = object()
_CHECK_OPERATORS = {
    "equals",
    "not_equals",
    "in",
    "not_in",
    "contains",
    "not_contains",
    "gt",
    "gte",
    "lt",
    "lte",
    "regex",
    "exists",
}


def _parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Evaluate a report JSON file against declarative policy checks."
    )
    parser.add_argument("--policy-file", default="", help="Declarative policy JSON file.")
    parser.add_argument("--report-file", default="", help="Report JSON file.")
    parser.add_argument(
        "--bundle-file",
        default="",
        help="Compatibility alias for --report-file.",
    )
    parser.add_argument(
        "--expected-final-decision",
        choices=("GO", "NO-GO"),
        default="",
        help="Optional decision assertion for contract-lane wrappers.",
    )
    parser.add_argument(
        "--ci-fast-gate",
        default="",
        help="Optional CI gate marker to include in emitted JSON output.",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for checker evaluation report JSON.",
    )
    parser.add_argument(
        "--legacy-target",
        default="",
        help="Compatibility mode: execute legacy checker target path directly.",
    )
    parser.add_argument(
        "--legacy-interpreter",
        choices=("python3", "bash"),
        default="python3",
        help="Compatibility mode interpreter for --legacy-target.",
    )
    parser.add_argument(
        "--legacy-args-prefix",
        action="append",
        default=[],
        help="Compatibility mode prefix argument; may be repeated.",
    )
    parser.add_argument("forward_args", nargs=argparse.REMAINDER)
    return parser.parse_args(argv)


def _run_legacy_delegate(args: argparse.Namespace, forward_args: list[str]) -> int:
    legacy_target = Path(args.legacy_target).resolve()
    if not legacy_target.is_file():
        fail(f"legacy target file not found: {legacy_target}")

    command: list[str] = [args.legacy_interpreter, str(legacy_target), *args.legacy_args_prefix]
    command.extend(forward_args)

    env = dict(os.environ)
    env["KAMN_DECLARATIVE_POLICY_CHECKER_DELEGATE"] = "1"
    completed = subprocess.run(
        command,
        check=False,
        env=env,
        capture_output=True,
        text=True,
    )
    if completed.stdout:
        print(completed.stdout, end="")
    if completed.stderr:
        print(completed.stderr, end="", file=sys.stderr)
    return int(completed.returncode)


def _validate_policy(policy: dict[str, Any]) -> dict[str, Any]:
    if policy.get("schema_version") != POLICY_SCHEMA_VERSION:
        fail("policy schema_version mismatch")

    required_string_fields = (
        "policy_name",
        "reason_taxonomy_version",
        "reason_key_prefix",
        "success_reason_code",
    )
    for field_name in required_string_fields:
        value = policy.get(field_name)
        if not isinstance(value, str) or not value.strip():
            fail(f"policy field '{field_name}' must be a non-empty string")

    checks = policy.get("checks")
    if not isinstance(checks, list) or not checks:
        fail("policy field 'checks' must be a non-empty list")

    for idx, check in enumerate(checks):
        if not isinstance(check, dict):
            fail(f"policy check at index {idx} must be an object")

        field_name = check.get("field")
        op = check.get("op")
        reason_code = check.get("reason_code")

        if not isinstance(field_name, str) or not field_name.strip():
            fail(f"policy check {idx} field must be a non-empty string")
        if not isinstance(op, str) or op not in _CHECK_OPERATORS:
            fail(f"policy check {idx} op must be one of: {', '.join(sorted(_CHECK_OPERATORS))}")
        if not isinstance(reason_code, str) or not reason_code.strip():
            fail(f"policy check {idx} reason_code must be a non-empty string")

        if op != "exists" and "expected" not in check:
            fail(f"policy check {idx} missing required 'expected' value")

        if op == "exists" and "expected" in check and not isinstance(check["expected"], bool):
            fail(f"policy check {idx} expected must be a boolean for exists op")

    return policy


def _resolve_field(payload: Any, field_path: str) -> Any:
    cursor: Any = payload
    for token in field_path.split("."):
        if isinstance(cursor, dict):
            if token not in cursor:
                return _MISSING
            cursor = cursor[token]
            continue
        if isinstance(cursor, list):
            if not token.isdigit():
                return _MISSING
            index = int(token)
            if index < 0 or index >= len(cursor):
                return _MISSING
            cursor = cursor[index]
            continue
        return _MISSING
    return cursor


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _evaluate_check(check: dict[str, Any], report_payload: dict[str, Any]) -> bool:
    field_value = _resolve_field(report_payload, check["field"])
    exists = field_value is not _MISSING
    op = check["op"]
    expected = check.get("expected")

    if op == "exists":
        expected_exists = expected if isinstance(expected, bool) else True
        return exists == expected_exists

    if not exists:
        return False

    if op == "equals":
        return field_value == expected
    if op == "not_equals":
        return field_value != expected
    if op == "in":
        return isinstance(expected, list) and field_value in expected
    if op == "not_in":
        return isinstance(expected, list) and field_value not in expected
    if op == "contains":
        if isinstance(field_value, str):
            return isinstance(expected, str) and expected in field_value
        if isinstance(field_value, list):
            return expected in field_value
        return False
    if op == "not_contains":
        if isinstance(field_value, str):
            return isinstance(expected, str) and expected not in field_value
        if isinstance(field_value, list):
            return expected not in field_value
        return False
    if op == "gt":
        return _is_number(field_value) and _is_number(expected) and field_value > expected
    if op == "gte":
        return _is_number(field_value) and _is_number(expected) and field_value >= expected
    if op == "lt":
        return _is_number(field_value) and _is_number(expected) and field_value < expected
    if op == "lte":
        return _is_number(field_value) and _is_number(expected) and field_value <= expected
    if op == "regex":
        if not isinstance(field_value, str) or not isinstance(expected, str):
            return False
        return re.search(expected, field_value) is not None
    fail(f"unsupported op: {op}")
    return False


def _evaluate(policy: dict[str, Any], report_payload: dict[str, Any]) -> dict[str, Any]:
    failed_reasons: list[str] = []
    for check in policy["checks"]:
        if not _evaluate_check(check, report_payload):
            failed_reasons.append(check["reason_code"])

    final_decision = "GO" if not failed_reasons else "NO-GO"
    reason_codes = [policy["success_reason_code"]] if final_decision == "GO" else failed_reasons
    reason_codes_csv = ",".join(reason_codes)
    status = "pass" if final_decision == "GO" else "fail"
    reason_key = f"{policy['reason_key_prefix']}:{final_decision}:v1"

    return {
        "schema_version": OUTPUT_SCHEMA_VERSION,
        "policy_schema_version": policy["schema_version"],
        "policy_name": policy["policy_name"],
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": policy["reason_taxonomy_version"],
        "reason_codes": reason_codes,
        "reason_codes_csv": reason_codes_csv,
        "reason_key": reason_key,
        "check_count": len(policy["checks"]),
        "failed_check_count": len(failed_reasons),
    }


def main(argv: list[str]) -> int:
    args = _parse_args(argv)

    forward_args = list(args.forward_args)
    if forward_args and forward_args[0] == "--":
        forward_args = forward_args[1:]

    if args.legacy_target:
        return _run_legacy_delegate(args, forward_args)

    if forward_args:
        fail(f"unexpected positional arguments: {' '.join(forward_args)}")

    policy_arg = args.policy_file
    report_arg = args.report_file or args.bundle_file
    if not policy_arg:
        fail("--policy-file is required")
    if not report_arg:
        fail("--report-file is required")

    policy_file = Path(policy_arg).resolve()
    report_file = Path(report_arg).resolve()

    if not policy_file.is_file():
        fail(f"policy file not found: {policy_file}")
    if not report_file.is_file():
        fail(f"report file not found: {report_file}")

    policy_payload = _validate_policy(dict(load_json(policy_file)))
    report_payload = dict(load_json(report_file))

    output = _evaluate(policy_payload, report_payload)
    output["policy_file"] = str(policy_file)
    output["report_file"] = str(report_file)
    if args.ci_fast_gate:
        output["ci_fast_gate"] = args.ci_fast_gate

    output_json_path = Path(args.output_json).resolve() if args.output_json else None
    if output_json_path is not None:
        write_json(output_json_path, output)

    print("status=ok")
    print(f"policy_name={output['policy_name']}")
    print(f"final_decision={output['final_decision']}")
    print(f"reason_codes={output['reason_codes_csv']}")
    print(f"reason_taxonomy_version={output['reason_taxonomy_version']}")
    print(f"reason_key={output['reason_key']}")
    print(f"check_count={output['check_count']}")
    print(f"failed_check_count={output['failed_check_count']}")
    if output_json_path is not None:
        print(f"output_json={output_json_path}")

    if args.expected_final_decision and output["final_decision"] != args.expected_final_decision:
        fail(
            "expected-final-decision mismatch: "
            f"expected {args.expected_final_decision}, found {output['final_decision']}"
        )

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

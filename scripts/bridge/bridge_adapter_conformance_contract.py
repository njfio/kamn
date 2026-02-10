#!/usr/bin/env python3
"""Bridge adapter conformance evidence generator and policy checker."""

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
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.bridge.adapter-conformance.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def parse_csv(raw: str) -> list[str]:
    fields = [part.strip() for part in raw.split(",")]
    return sorted({field for field in fields if field})


def ensure_string_list(value: object, field_name: str) -> list[str]:
    if not isinstance(value, list):
        fail(f"{field_name} must be an array")
    if not all(isinstance(item, str) for item in value):
        fail(f"{field_name} entries must be strings")
    return [item.strip() for item in value if item.strip()]


def normalize_set(values: list[str]) -> list[str]:
    return sorted(set(values))


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.adapter_id,
        args.bridge_network,
        args.dry_run,
        args.request_expected_schema_version,
        args.request_observed_schema_version,
        args.request_required_fields,
        args.request_observed_fields,
        args.receipt_expected_schema_version,
        args.receipt_observed_schema_version,
        args.receipt_required_fields,
        args.receipt_observed_fields,
        args.ci_fast_gate,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all bridge adapter conformance evidence arguments are required")

    bridge_network = args.bridge_network
    if bridge_network not in {"ethereum", "solana", "near", "custom"}:
        fail("bridge-network must be ethereum|solana|near|custom")

    dry_run_raw = args.dry_run
    if dry_run_raw not in {"true", "false"}:
        fail("dry-run must be true or false")
    dry_run = dry_run_raw == "true"

    if args.ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci-fast-gate must be PASS or FAIL")

    adapter_id = args.adapter_id
    if not adapter_id.strip():
        fail("adapter-id must be non-empty")

    request_required_fields = parse_csv(args.request_required_fields)
    request_observed_fields = parse_csv(args.request_observed_fields)
    receipt_required_fields = parse_csv(args.receipt_required_fields)
    receipt_observed_fields = parse_csv(args.receipt_observed_fields)

    request_missing_required_fields = sorted(
        field
        for field in request_required_fields
        if field not in set(request_observed_fields)
    )
    receipt_missing_required_fields = sorted(
        field
        for field in receipt_required_fields
        if field not in set(receipt_observed_fields)
    )

    reason_codes: list[str] = []
    if not dry_run:
        reason_codes.append("dry_run_disabled")
    if args.request_expected_schema_version != args.request_observed_schema_version:
        reason_codes.append("request_schema_version_mismatch")
    if args.receipt_expected_schema_version != args.receipt_observed_schema_version:
        reason_codes.append("receipt_schema_version_mismatch")
    if not request_required_fields:
        reason_codes.append("request_required_fields_contract_missing")
    if request_missing_required_fields:
        reason_codes.append("request_required_fields_missing")
    if not receipt_required_fields:
        reason_codes.append("receipt_required_fields_contract_missing")
    if receipt_missing_required_fields:
        reason_codes.append("receipt_required_fields_missing")
    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION
    if final_decision == GO_DECISION:
        reason_codes = [
            "adapter_conformance_dry_run_mode",
            "adapter_request_receipt_contracts_compatible",
        ]

    reason_key = f"bridge_adapter_conformance_reason_codes:{final_decision}:v1"

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "adapter_id": adapter_id,
        "bridge_network": bridge_network,
        "dry_run": dry_run,
        "request_contract": {
            "expected_schema_version": args.request_expected_schema_version,
            "observed_schema_version": args.request_observed_schema_version,
            "required_fields": request_required_fields,
            "observed_fields": request_observed_fields,
            "missing_required_fields": request_missing_required_fields,
        },
        "receipt_contract": {
            "expected_schema_version": args.receipt_expected_schema_version,
            "observed_schema_version": args.receipt_observed_schema_version,
            "required_fields": receipt_required_fields,
            "observed_fields": receipt_observed_fields,
            "missing_required_fields": receipt_missing_required_fields,
        },
        "ci_fast_gate": args.ci_fast_gate,
        "reason_key": reason_key,
        "reason_codes": reason_codes,
        "decision_reasons": reason_codes,
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    print(f"reason_key={reason_key}")
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
            "adapter_id",
            "bridge_network",
            "dry_run",
            "request_contract",
            "receipt_contract",
            "ci_fast_gate",
            "reason_key",
            "reason_codes",
            "decision_reasons",
            "final_decision",
        ),
    )

    if payload["schema_version"] != SCHEMA_VERSION:
        fail("unexpected schema_version for bridge adapter conformance evidence bundle")

    adapter_id = payload["adapter_id"]
    if not isinstance(adapter_id, str) or not adapter_id.strip():
        fail("adapter_id must be a non-empty string")

    bridge_network = payload["bridge_network"]
    if bridge_network not in {"ethereum", "solana", "near", "custom"}:
        fail("bridge_network must be ethereum|solana|near|custom")

    dry_run = payload["dry_run"]
    if not isinstance(dry_run, bool):
        fail("dry_run must be a boolean")

    ci_fast_gate = payload["ci_fast_gate"]
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    request_contract = payload["request_contract"]
    if not isinstance(request_contract, dict):
        fail("request_contract must be an object")
    receipt_contract = payload["receipt_contract"]
    if not isinstance(receipt_contract, dict):
        fail("receipt_contract must be an object")

    for section_name, section in (
        ("request_contract", request_contract),
        ("receipt_contract", receipt_contract),
    ):
        for key in (
            "expected_schema_version",
            "observed_schema_version",
            "required_fields",
            "observed_fields",
            "missing_required_fields",
        ):
            if key not in section:
                fail(f"{section_name} missing field: {key}")
        if not isinstance(section["expected_schema_version"], str):
            fail(f"{section_name}.expected_schema_version must be a string")
        if not isinstance(section["observed_schema_version"], str):
            fail(f"{section_name}.observed_schema_version must be a string")

    request_required_fields = normalize_set(
        ensure_string_list(
            request_contract["required_fields"], "request_contract.required_fields"
        )
    )
    request_observed_fields = normalize_set(
        ensure_string_list(
            request_contract["observed_fields"], "request_contract.observed_fields"
        )
    )
    request_missing_required_fields = normalize_set(
        ensure_string_list(
            request_contract["missing_required_fields"],
            "request_contract.missing_required_fields",
        )
    )

    receipt_required_fields = normalize_set(
        ensure_string_list(
            receipt_contract["required_fields"], "receipt_contract.required_fields"
        )
    )
    receipt_observed_fields = normalize_set(
        ensure_string_list(
            receipt_contract["observed_fields"], "receipt_contract.observed_fields"
        )
    )
    receipt_missing_required_fields = normalize_set(
        ensure_string_list(
            receipt_contract["missing_required_fields"],
            "receipt_contract.missing_required_fields",
        )
    )

    computed_request_missing_required_fields = sorted(
        field
        for field in request_required_fields
        if field not in set(request_observed_fields)
    )
    computed_receipt_missing_required_fields = sorted(
        field
        for field in receipt_required_fields
        if field not in set(receipt_observed_fields)
    )

    if request_missing_required_fields != computed_request_missing_required_fields:
        fail(
            "request missing_required_fields mismatch: "
            f"expected {computed_request_missing_required_fields}, found {request_missing_required_fields}"
        )
    if receipt_missing_required_fields != computed_receipt_missing_required_fields:
        fail(
            "receipt missing_required_fields mismatch: "
            f"expected {computed_receipt_missing_required_fields}, found {receipt_missing_required_fields}"
        )

    reason_key = payload["reason_key"]
    if not isinstance(reason_key, str):
        fail("reason_key must be a string")

    reason_codes = ensure_string_list(payload["reason_codes"], "reason_codes")
    decision_reasons = ensure_string_list(payload["decision_reasons"], "decision_reasons")

    expected_reason_codes: list[str] = []
    if not dry_run:
        expected_reason_codes.append("dry_run_disabled")
    if (
        request_contract["expected_schema_version"]
        != request_contract["observed_schema_version"]
    ):
        expected_reason_codes.append("request_schema_version_mismatch")
    if (
        receipt_contract["expected_schema_version"]
        != receipt_contract["observed_schema_version"]
    ):
        expected_reason_codes.append("receipt_schema_version_mismatch")
    if not request_required_fields:
        expected_reason_codes.append("request_required_fields_contract_missing")
    if computed_request_missing_required_fields:
        expected_reason_codes.append("request_required_fields_missing")
    if not receipt_required_fields:
        expected_reason_codes.append("receipt_required_fields_contract_missing")
    if computed_receipt_missing_required_fields:
        expected_reason_codes.append("receipt_required_fields_missing")
    if ci_fast_gate != "PASS":
        expected_reason_codes.append("ci_fast_gate_failed")

    expected_final_decision = GO_DECISION
    if expected_reason_codes:
        expected_final_decision = NO_GO_DECISION
    else:
        expected_reason_codes = [
            "adapter_conformance_dry_run_mode",
            "adapter_request_receipt_contracts_compatible",
        ]

    actual_final_decision = payload["final_decision"]
    if actual_final_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_final_decision != expected_final_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_final_decision}, found {actual_final_decision}"
        )

    if reason_codes != expected_reason_codes:
        fail(
            "reason_codes mismatch: "
            f"expected {expected_reason_codes}, found {reason_codes}"
        )

    if decision_reasons != expected_reason_codes:
        fail(
            "decision_reasons mismatch: "
            f"expected {expected_reason_codes}, found {decision_reasons}"
        )

    expected_reason_key = (
        f"bridge_adapter_conformance_reason_codes:{expected_final_decision}:v1"
    )
    if reason_key != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {reason_key}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_final_decision}")
    print(f"reason_key={reason_key}")
    print(f"decision_reasons={'; '.join(expected_reason_codes)}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Bridge adapter conformance evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--adapter-id")
    generate.add_argument("--bridge-network")
    generate.add_argument("--dry-run")
    generate.add_argument("--request-expected-schema-version")
    generate.add_argument("--request-observed-schema-version")
    generate.add_argument("--request-required-fields")
    generate.add_argument("--request-observed-fields")
    generate.add_argument("--receipt-expected-schema-version")
    generate.add_argument("--receipt-observed-schema-version")
    generate.add_argument("--receipt-required-fields")
    generate.add_argument("--receipt-observed-fields")
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

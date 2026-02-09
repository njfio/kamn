#!/usr/bin/env python3
"""Settlement reconciliation evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any, Mapping

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_int,
    require_keys,
    require_non_negative_int,
    require_object,
    write_json,
)

SCHEMA_VERSION = "kamn.escrow.settlement-reconciliation.v1"

SETTLEMENT_OUTCOMES = {
    "RELEASED",
    "REFUNDED",
    "TIMEOUT_REFUNDED",
    "DISPUTED_RESOLVED",
}
RECEIPT_FINALITY = {"FINAL", "PENDING", "FAILED"}
SETTLEMENT_PATHS = {"PAYOUT", "REFUND", "DISPUTE"}
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"


def _parse_timeout_elapsed(raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail("timeout-elapsed must be true or false")


def _parse_settlement_outcome(raw_value: str) -> str:
    if raw_value not in SETTLEMENT_OUTCOMES:
        fail(
            "settlement-outcome must be "
            "RELEASED|REFUNDED|TIMEOUT_REFUNDED|DISPUTED_RESOLVED"
        )
    return raw_value


def _parse_receipt_finality(raw_value: str) -> str:
    if raw_value not in RECEIPT_FINALITY:
        fail("receipt-finality must be FINAL|PENDING|FAILED")
    return raw_value


def _parse_ci_fast_gate(raw_value: str, *, field_name: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail(f"{field_name} must be PASS or FAIL")


def _settlement_path(settlement_outcome: str) -> tuple[str, str]:
    if settlement_outcome == "RELEASED":
        return "PAYOUT", "settlement_path_payout"
    if settlement_outcome in {"REFUNDED", "TIMEOUT_REFUNDED"}:
        return "REFUND", "settlement_path_refund"
    return "DISPUTE", "settlement_path_dispute"


def _failed_reason_codes(
    *,
    receipt_id: str,
    receipt_finality: str,
    expected_release: int,
    expected_refund: int,
    observed_release: int,
    observed_refund: int,
    ledger_reference_id: str,
    settlement_outcome: str,
    timeout_elapsed: bool,
    ci_fast_gate: str,
) -> list[str]:
    failed: list[str] = []
    if not receipt_id.strip() or receipt_finality != "FINAL":
        failed.append("receipt_evidence_invalid")
    if expected_release != observed_release or expected_refund != observed_refund:
        failed.append("ledger_amount_drift_detected")
    if not ledger_reference_id.strip():
        failed.append("ledger_reference_missing")
    if settlement_outcome == "TIMEOUT_REFUNDED" and not timeout_elapsed:
        failed.append("timeout_not_elapsed")
    if ci_fast_gate != "PASS":
        failed.append("ci_fast_gate_failed")
    return failed


def generate_bundle(args: argparse.Namespace) -> int:
    settlement_outcome = _parse_settlement_outcome(args.settlement_outcome)
    receipt_finality = _parse_receipt_finality(args.receipt_finality)
    timeout_elapsed = _parse_timeout_elapsed(args.timeout_elapsed)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate, field_name="ci-fast-gate")

    expected_release = require_non_negative_int(
        "expected-release-amount", str(args.expected_release_amount)
    )
    expected_refund = require_non_negative_int(
        "expected-refund-amount", str(args.expected_refund_amount)
    )
    observed_release = require_non_negative_int(
        "observed-release-amount", str(args.observed_release_amount)
    )
    observed_refund = require_non_negative_int(
        "observed-refund-amount", str(args.observed_refund_amount)
    )

    settlement_path, path_reason_code = _settlement_path(settlement_outcome)
    failed_reason_codes = _failed_reason_codes(
        receipt_id=args.receipt_id,
        receipt_finality=receipt_finality,
        expected_release=expected_release,
        expected_refund=expected_refund,
        observed_release=observed_release,
        observed_refund=observed_refund,
        ledger_reference_id=args.ledger_reference_id,
        settlement_outcome=settlement_outcome,
        timeout_elapsed=timeout_elapsed,
        ci_fast_gate=ci_fast_gate,
    )

    if failed_reason_codes:
        final_decision = NO_GO_DECISION
        reason_codes = [path_reason_code] + failed_reason_codes
    else:
        final_decision = GO_DECISION
        reason_codes = [path_reason_code, "all_settlement_reconciliation_checks_passed"]
    reason_key = f"settlement_reconciliation_reason_codes:{final_decision}:v1"

    payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "escrow_id": args.escrow_id,
        "settlement_outcome": settlement_outcome,
        "settlement_path": settlement_path,
        "receipt": {
            "receipt_id": args.receipt_id,
            "finality": receipt_finality,
        },
        "expected_amounts": {
            "release": expected_release,
            "refund": expected_refund,
        },
        "observed_amounts": {
            "release": observed_release,
            "refund": observed_refund,
        },
        "ledger": {
            "reference_id": args.ledger_reference_id,
        },
        "timeout_elapsed": timeout_elapsed,
        "ci_fast_gate": ci_fast_gate,
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


def _require_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str):
        fail(f"{field_name} must be a string")
    return value


def _require_string_array(payload: Mapping[str, Any], field_name: str) -> list[str]:
    value = payload.get(field_name)
    if not isinstance(value, list):
        fail(f"{field_name} must be an array")
    if not all(isinstance(item, str) for item in value):
        fail(f"{field_name} entries must be strings")
    return value


def _validate_amount_section(
    payload: Mapping[str, Any], section_name: str
) -> tuple[int, int]:
    section = payload.get(section_name)
    if not isinstance(section, dict):
        fail("expected_amounts and observed_amounts must be objects")

    if "release" not in section:
        fail(f"missing {section_name}.release")
    if "refund" not in section:
        fail(f"missing {section_name}.refund")

    release = section.get("release")
    refund = section.get("refund")
    if not isinstance(release, int):
        fail(f"{section_name}.release must be an integer")
    if not isinstance(refund, int):
        fail(f"{section_name}.refund must be an integer")
    if release < 0:
        fail(f"{section_name}.release must be >= 0")
    if refund < 0:
        fail(f"{section_name}.refund must be >= 0")
    return release, refund


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
            "escrow_id",
            "settlement_outcome",
            "settlement_path",
            "receipt",
            "expected_amounts",
            "observed_amounts",
            "ledger",
            "timeout_elapsed",
            "ci_fast_gate",
            "reason_key",
            "reason_codes",
            "decision_reasons",
            "final_decision",
        ),
    )

    if payload.get("schema_version") != SCHEMA_VERSION:
        fail("unexpected schema_version for settlement reconciliation evidence bundle")

    settlement_outcome = _require_string(payload, "settlement_outcome")
    if settlement_outcome not in SETTLEMENT_OUTCOMES:
        fail("settlement_outcome must be RELEASED|REFUNDED|TIMEOUT_REFUNDED|DISPUTED_RESOLVED")

    settlement_path = _require_string(payload, "settlement_path")
    if settlement_path not in SETTLEMENT_PATHS:
        fail("settlement_path must be PAYOUT|REFUND|DISPUTE")

    receipt = require_object(payload, "receipt")
    if "receipt_id" not in receipt or "finality" not in receipt:
        fail("receipt object must contain receipt_id and finality")
    receipt_id = receipt.get("receipt_id")
    if not isinstance(receipt_id, str):
        fail("receipt.receipt_id must be a string")
    receipt_finality = receipt.get("finality")
    if receipt_finality not in RECEIPT_FINALITY:
        fail("receipt.finality must be FINAL|PENDING|FAILED")

    expected_release, expected_refund = _validate_amount_section(
        payload, "expected_amounts"
    )
    observed_release, observed_refund = _validate_amount_section(
        payload, "observed_amounts"
    )

    ledger = require_object(payload, "ledger")
    if "reference_id" not in ledger:
        fail("missing ledger.reference_id")
    ledger_reference_id = ledger.get("reference_id")
    if not isinstance(ledger_reference_id, str):
        fail("ledger.reference_id must be a string")

    timeout_elapsed = payload.get("timeout_elapsed")
    if not isinstance(timeout_elapsed, bool):
        fail("timeout_elapsed must be a boolean")

    ci_fast_gate = payload.get("ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    reason_key = _require_string(payload, "reason_key")
    reason_codes = _require_string_array(payload, "reason_codes")

    decision_reasons_raw = payload.get("decision_reasons")
    if not isinstance(decision_reasons_raw, list):
        fail("decision_reasons must be an array")
    if not all(isinstance(item, str) for item in decision_reasons_raw):
        fail("decision_reasons entries must be strings")

    expected_path, path_reason_code = _settlement_path(settlement_outcome)
    if settlement_path != expected_path:
        fail(
            "settlement_path mismatch: "
            f"expected {expected_path}, found {settlement_path}"
        )

    failed_reason_codes = _failed_reason_codes(
        receipt_id=receipt_id,
        receipt_finality=receipt_finality,
        expected_release=expected_release,
        expected_refund=expected_refund,
        observed_release=observed_release,
        observed_refund=observed_refund,
        ledger_reference_id=ledger_reference_id,
        settlement_outcome=settlement_outcome,
        timeout_elapsed=timeout_elapsed,
        ci_fast_gate=ci_fast_gate,
    )

    if failed_reason_codes:
        expected_decision = NO_GO_DECISION
        expected_reason_codes = [path_reason_code] + failed_reason_codes
    else:
        expected_decision = GO_DECISION
        expected_reason_codes = [
            path_reason_code,
            "all_settlement_reconciliation_checks_passed",
        ]

    if reason_codes != expected_reason_codes:
        fail(
            "reason_codes mismatch: "
            f"expected {expected_reason_codes}, found {reason_codes}"
        )

    if decision_reasons_raw != expected_reason_codes:
        fail(
            "decision_reasons mismatch: "
            f"expected {expected_reason_codes}, found {decision_reasons_raw}"
        )

    actual_decision = payload.get("final_decision")
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    expected_reason_key = f"settlement_reconciliation_reason_codes:{expected_decision}:v1"
    if reason_key != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {reason_key}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"reason_key={reason_key}")
    print(f"decision_reasons={'; '.join(decision_reasons_raw)}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Settlement reconciliation evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--escrow-id", required=True)
    generate.add_argument("--settlement-outcome", required=True)
    generate.add_argument("--receipt-id", required=True)
    generate.add_argument("--receipt-finality", required=True)
    generate.add_argument("--expected-release-amount", required=True)
    generate.add_argument("--expected-refund-amount", required=True)
    generate.add_argument("--observed-release-amount", required=True)
    generate.add_argument("--observed-refund-amount", required=True)
    generate.add_argument("--ledger-reference-id", required=True)
    generate.add_argument("--timeout-elapsed", required=True)
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

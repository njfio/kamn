#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, load_json, write_json
from gonogo_evidence_contract import (
    GO_DECISION,
    LIVE_GONOGO_REASON_TAXONOMY_VERSION,
    NO_GO_DECISION,
    _validated_expected_milestone_bundle,
)

SCHEMA_VERSION = "kamn.release.upgrade-rehearsal-lineage-policy-report.v1"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Validate go/no-go milestone upgrade rehearsal lineage and deterministic "
            "promotion-gate reason mapping."
        )
    )
    parser.add_argument("--bundle-file", required=True)
    parser.add_argument(
        "--expected-final-decision",
        required=True,
        choices=[GO_DECISION, NO_GO_DECISION],
    )
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def _fail(message: str) -> None:
    raise ContractError(message)


def _as_reason_codes(value: object) -> list[str]:
    if not isinstance(value, list):
        _fail("milestone_review_bundle.reason_codes must be a list")
    normalized: list[str] = []
    for entry in value:
        if not isinstance(entry, str) or not entry.strip():
            _fail("milestone_review_bundle.reason_codes entries must be non-empty strings")
        normalized.append(entry)
    return normalized


def _validate_promotion_reason_mapping(milestone_bundle: dict[str, object]) -> tuple[list[str], str]:
    reason_taxonomy_version = str(milestone_bundle.get("reason_taxonomy_version", ""))
    if reason_taxonomy_version != LIVE_GONOGO_REASON_TAXONOMY_VERSION:
        _fail(
            "promotion gate reason taxonomy mismatch: expected "
            f"{LIVE_GONOGO_REASON_TAXONOMY_VERSION}, found {reason_taxonomy_version or '<missing>'}"
        )

    reason_codes = _as_reason_codes(milestone_bundle.get("reason_codes"))
    if reason_codes != sorted(set(reason_codes)):
        _fail(
            "promotion gate reason mapping mismatch: reason_codes must be sorted and unique"
        )

    expected_reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)
    reason_codes_csv = str(milestone_bundle.get("reason_codes_csv", ""))
    reason_codes_value = str(milestone_bundle.get("reason_codes_value", ""))
    if reason_codes_csv != expected_reason_codes_csv:
        _fail(
            "promotion gate reason mapping mismatch: "
            f"reason_codes_csv expected {expected_reason_codes_csv}, found {reason_codes_csv or '<missing>'}"
        )
    if reason_codes_value != expected_reason_codes_csv:
        _fail(
            "promotion gate reason mapping mismatch: "
            f"reason_codes_value expected {expected_reason_codes_csv}, found {reason_codes_value or '<missing>'}"
        )
    return reason_codes, expected_reason_codes_csv


def main() -> int:
    args = parse_args()
    bundle_file = Path(args.bundle_file).resolve()
    if not bundle_file.is_file():
        _fail(f"bundle file not found: {bundle_file}")

    payload = load_json(bundle_file)
    if not isinstance(payload, dict):
        _fail("bundle payload must be a JSON object")

    milestone_bundle = payload.get("milestone_review_bundle")
    if not isinstance(milestone_bundle, dict):
        _fail("bundle field 'milestone_review_bundle' must be an object")

    reason_codes, reason_codes_csv = _validate_promotion_reason_mapping(milestone_bundle)
    expected_bundle, derived_final_decision = _validated_expected_milestone_bundle(payload)
    if derived_final_decision != args.expected_final_decision:
        _fail(
            "lineage final decision mismatch: "
            f"expected {args.expected_final_decision}, found {derived_final_decision}"
        )

    for required_reason_code in args.require_reason_code:
        if required_reason_code not in reason_codes:
            _fail(
                "missing required reason code: "
                f"{required_reason_code} (observed={reason_codes_csv})"
            )

    status_payload = {
        "schema_version": SCHEMA_VERSION,
        "status": "ok",
        "bundle_file": str(bundle_file),
        "upgrade_lineage_final_decision": derived_final_decision,
        "upgrade_lineage_reason_taxonomy_version": LIVE_GONOGO_REASON_TAXONOMY_VERSION,
        "upgrade_lineage_reason_codes_csv": reason_codes_csv,
        "upgrade_lineage_reason_codes_value": reason_codes_csv,
        "promotion_gate_reason_taxonomy_version": LIVE_GONOGO_REASON_TAXONOMY_VERSION,
        "promotion_gate_reason_codes_csv": reason_codes_csv,
        "promotion_gate_reason_codes_value": reason_codes_csv,
        "required_reason_codes": list(args.require_reason_code),
        "reason_codes": reason_codes,
        "lineage_status": expected_bundle["lineage_status"],
        "final_decision": derived_final_decision,
    }

    if args.output_json:
        write_json(Path(args.output_json), status_payload)

    print("status=ok")
    print(f"bundle_file={bundle_file}")
    print(f"upgrade_lineage_final_decision={derived_final_decision}")
    print(f"upgrade_lineage_reason_taxonomy_version={LIVE_GONOGO_REASON_TAXONOMY_VERSION}")
    print(f"upgrade_lineage_reason_codes_csv={reason_codes_csv}")
    print(f"upgrade_lineage_reason_codes_value={reason_codes_csv}")
    print(f"promotion_gate_reason_taxonomy_version={LIVE_GONOGO_REASON_TAXONOMY_VERSION}")
    print(f"promotion_gate_reason_codes_csv={reason_codes_csv}")
    print(f"promotion_gate_reason_codes_value={reason_codes_csv}")
    print(f"final_decision={derived_final_decision}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

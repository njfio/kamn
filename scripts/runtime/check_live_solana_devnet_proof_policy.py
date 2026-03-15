#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

VALIDATION_SCHEMA_VERSION = "kamn.solana.devnet.live-proof-validation.v1"
NORMALIZATION_SCHEMA_VERSION = "kamn.solana.devnet.live-normalization-report.v1"
POLICY_SCHEMA_VERSION = "kamn.solana.devnet.live-proof-policy.v1"
EXPECTED_FINALITIES = {
    "processed": "Pending",
    "confirmed": "Pending",
    "finalized": "Final",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce GO/NO-GO policy for live Solana devnet proof artifacts."
    )
    parser.add_argument("--validation-report-file", required=True)
    parser.add_argument("--normalization-report-file", required=True)
    parser.add_argument("--expected-final-decision", default="GO", choices=["GO", "NO-GO"])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def load_json(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit(f"expected JSON object in {path}")
    return payload


def evaluate(validation: dict[str, object], normalization: dict[str, object]) -> list[str]:
    reasons: list[str] = []
    if validation.get("schema_version") != VALIDATION_SCHEMA_VERSION:
        reasons.append("validation_schema_version_mismatch")
    if validation.get("final_decision") != "PASS":
        reasons.append("validation_not_pass")
    if normalization.get("schema_version") != NORMALIZATION_SCHEMA_VERSION:
        reasons.append("normalization_schema_version_mismatch")
    if normalization.get("status") != "ok":
        reasons.append("normalization_status_not_ok")
    if normalization.get("assertions_passed") is not True:
        reasons.append("normalization_assertions_missing")
    finalities = normalization.get("normalized_finalities")
    if finalities != EXPECTED_FINALITIES:
        reasons.append("normalized_finalities_invalid")
    return sorted(set(reasons))


def main() -> int:
    args = parse_args()
    validation_file = Path(args.validation_report_file).resolve()
    normalization_file = Path(args.normalization_report_file).resolve()
    validation = load_json(validation_file)
    normalization = load_json(normalization_file)
    reason_codes = evaluate(validation, normalization)
    if args.expected_final_decision == "GO":
        status = "ok" if not reason_codes else "fail"
        final_decision = "GO" if not reason_codes else "NO-GO"
    else:
        status = "ok" if reason_codes else "fail"
        final_decision = "NO-GO" if reason_codes else "GO"
    if not reason_codes and final_decision == "GO":
        reason_codes = ["live_solana_devnet_proof_policy_passed"]
    output = {
        "schema_version": POLICY_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "reason_codes": reason_codes,
        "source_validation_report_file": str(validation_file),
        "source_normalization_report_file": str(normalization_file),
    }
    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(output, sort_keys=True, indent=2) + "\n", encoding="utf-8")
        print(f"policy_report_file={output_file}")
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print("reason_codes=" + ("none" if not reason_codes else ",".join(reason_codes)))
    return 0 if status == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

RUNNER_SCHEMA_VERSION = "kamn.solana.devnet.live-proof-report.v1"
VALIDATION_SCHEMA_VERSION = "kamn.solana.devnet.live-proof-validation.v1"
EXPECTED_LABELS = ["processed", "confirmed", "finalized"]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate live Solana devnet proof report shape and evidence."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def load_report(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit("live Solana report must be a JSON object")
    return payload


def evaluate(report: dict[str, object]) -> list[str]:
    reasons: list[str] = []
    if report.get("schema_version") != RUNNER_SCHEMA_VERSION:
        reasons.append("report_schema_version_mismatch")
    if report.get("health_status") != "ok":
        reasons.append("health_status_not_ok")
    if not isinstance(report.get("rpc_url"), str) or not str(report.get("rpc_url")).startswith("http"):
        reasons.append("rpc_url_invalid")
    if not isinstance(report.get("solana_core_version"), str) or not report.get("solana_core_version"):
        reasons.append("solana_core_version_missing")
    if not isinstance(report.get("feature_set"), int):
        reasons.append("feature_set_invalid")
    slots = report.get("commitment_slots")
    if not isinstance(slots, dict):
        reasons.append("commitment_slots_invalid")
        slots = {}
    slot_values: dict[str, int] = {}
    for label in EXPECTED_LABELS:
        value = slots.get(label)
        if not isinstance(value, int):
            reasons.append(f"slot_{label}_invalid")
        else:
            slot_values[label] = value
    if slot_values and not (
        slot_values["processed"] >= slot_values["confirmed"] >= slot_values["finalized"]
    ):
        reasons.append("slot_order_invalid")
    if report.get("slot_order_valid") is not True:
        reasons.append("slot_order_flag_invalid")
    labels = report.get("finality_labels")
    if labels != EXPECTED_LABELS:
        reasons.append("finality_labels_invalid")
    proofs = report.get("receipt_proofs")
    if not isinstance(proofs, list) or len(proofs) != 3:
        reasons.append("receipt_proofs_invalid")
        proofs = []
    observed_labels: list[str] = []
    for proof in proofs:
        if not isinstance(proof, dict):
            reasons.append("receipt_proof_entry_invalid")
            continue
        label = proof.get("finality_label")
        observed_labels.append(label)
        if label not in EXPECTED_LABELS:
            reasons.append("receipt_proof_finality_invalid")
        if proof.get("network") != "solana":
            reasons.append("receipt_proof_network_invalid")
        if proof.get("status") != "success":
            reasons.append("receipt_proof_status_invalid")
    if sorted(observed_labels) != sorted(EXPECTED_LABELS):
        reasons.append("receipt_proof_labels_missing")
    return sorted(set(reasons))


def main() -> int:
    args = parse_args()
    report_file = Path(args.report_file).resolve()
    report = load_report(report_file)
    reason_codes = evaluate(report)
    status = "ok" if not reason_codes else "fail"
    final_decision = "PASS" if not reason_codes else "FAIL"
    output = {
        "schema_version": VALIDATION_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "reason_codes": reason_codes,
        "source_report_file": str(report_file),
        "observed_slots": report.get("commitment_slots"),
        "observed_labels": report.get("finality_labels"),
    }
    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(output, sort_keys=True, indent=2) + "\n", encoding="utf-8")
        print(f"validation_report_file={output_file}")
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print("reason_codes=" + ("none" if not reason_codes else ",".join(reason_codes)))
    return 0 if status == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())

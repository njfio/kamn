#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

RECOVERY_REASON_TAXONOMY_VERSION = "kamn.kolme.runtime-commit-recovery-reason-taxonomy.v1"
RECOVERY_REASON_CODES = (
    "recovery_nonce_not_monotonic",
    "recovery_payload_hash_mismatch",
    "recovery_receipt_not_final",
    "recovery_replay_detected",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Evaluate fail-closed runtime commit replay/tamper policy checks."
    )
    parser.add_argument("--operation-id", required=True)
    parser.add_argument("--idempotency-key", required=True)
    parser.add_argument("--receipt-provider", required=True)
    parser.add_argument("--expected-receipt-provider", required=True)
    parser.add_argument("--receipt-commit-id", required=True)
    parser.add_argument("--expected-receipt-commit-id", required=True)
    parser.add_argument("--nonce-monotonic", required=True, choices=["true", "false"])
    parser.add_argument("--replay-detected", required=True, choices=["true", "false"])
    parser.add_argument("--payload-hash-match", required=True, choices=["true", "false"])
    parser.add_argument(
        "--receipt-finality", required=True, choices=["PENDING", "FINAL", "FAILED"]
    )
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(args: argparse.Namespace) -> tuple[str, list[str], str]:
    nonce_monotonic = args.nonce_monotonic == "true"
    replay_detected = args.replay_detected == "true"
    payload_hash_match = args.payload_hash_match == "true"

    reason_codes: list[str] = []
    recovery_reason_codes: list[str] = []
    if not args.operation_id.strip():
        reason_codes.append("operation_id_missing")
    if not args.idempotency_key.strip():
        reason_codes.append("idempotency_key_missing")
    if not args.receipt_provider.strip():
        reason_codes.append("receipt_provider_missing")
    if args.receipt_provider != args.expected_receipt_provider:
        reason_codes.append("receipt_provider_mismatch")
    if not args.receipt_commit_id.strip():
        reason_codes.append("receipt_commit_id_missing")
    if args.receipt_commit_id != args.expected_receipt_commit_id:
        reason_codes.append("receipt_commit_id_mismatch")
    if not nonce_monotonic:
        reason_codes.append("nonce_not_monotonic")
        recovery_reason_codes.append("recovery_nonce_not_monotonic")
    if replay_detected:
        reason_codes.append("replay_detected")
        recovery_reason_codes.append("recovery_replay_detected")
    if not payload_hash_match:
        reason_codes.append("payload_hash_mismatch")
        recovery_reason_codes.append("recovery_payload_hash_mismatch")
    if args.receipt_finality != "FINAL":
        reason_codes.append("receipt_not_final")
        recovery_reason_codes.append("recovery_receipt_not_final")
    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_recovery_reason_codes = set(recovery_reason_codes)
    recovery_reason_codes_value = ",".join(
        code for code in RECOVERY_REASON_CODES if code in observed_recovery_reason_codes
    )
    if not recovery_reason_codes_value:
        recovery_reason_codes_value = "none"

    final_decision = "GO" if not reason_codes else "NO-GO"
    return final_decision, reason_codes, recovery_reason_codes_value


def main() -> int:
    args = parse_args()
    final_decision, reason_codes, recovery_reason_codes_value = evaluate(args)

    report = {
        "schema_version": "kamn.kolme.runtime-commit-replay-policy-report.v1",
        "operation_id": args.operation_id,
        "idempotency_key": args.idempotency_key,
        "receipt_provider": args.receipt_provider,
        "expected_receipt_provider": args.expected_receipt_provider,
        "receipt_commit_id": args.receipt_commit_id,
        "expected_receipt_commit_id": args.expected_receipt_commit_id,
        "nonce_monotonic": args.nonce_monotonic == "true",
        "replay_detected": args.replay_detected == "true",
        "payload_hash_match": args.payload_hash_match == "true",
        "receipt_finality": args.receipt_finality,
        "ci_fast_gate": args.ci_fast_gate,
        "recovery_reason_taxonomy_version": RECOVERY_REASON_TAXONOMY_VERSION,
        "recovery_reason_codes_csv": ",".join(RECOVERY_REASON_CODES),
        "recovery_reason_codes_value": recovery_reason_codes_value,
        "retransmission_evidence_contract_version": "v1",
        "nonce_idempotency_contract_version": "v1",
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    status = "ok" if final_decision == "GO" else "fail"
    failed_checks = ",".join(reason_codes) if reason_codes else "none"
    print(f"status={status}")
    print(f"operation_id={args.operation_id}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    print(f"recovery_reason_taxonomy_version={RECOVERY_REASON_TAXONOMY_VERSION}")
    print(f"recovery_reason_codes_csv={','.join(RECOVERY_REASON_CODES)}")
    print(f"recovery_reason_codes_value={recovery_reason_codes_value}")
    print("retransmission_evidence_contract_version=v1")
    print("nonce_idempotency_contract_version=v1")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())

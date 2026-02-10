#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Evaluate fail-closed nonce/broadcast parity policy checks."
    )
    parser.add_argument("--case-id", required=True)
    parser.add_argument("--operation", required=True, choices=["nonce", "broadcast"])
    parser.add_argument("--http-status", required=True, type=int)
    parser.add_argument("--nonce-value", required=True, type=int)
    parser.add_argument("--broadcast-accepted", required=True, choices=["true", "false"])
    parser.add_argument("--duplicate-detected", required=True, choices=["true", "false"])
    parser.add_argument("--payload-valid", required=True, choices=["true", "false"])
    parser.add_argument("--authorization-present", required=True, choices=["true", "false"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    payload_valid = args.payload_valid == "true"
    broadcast_accepted = args.broadcast_accepted == "true"
    duplicate_detected = args.duplicate_detected == "true"
    authorization_present = args.authorization_present == "true"

    if not args.case_id.strip():
        reason_codes.append("case_id_missing")
    if args.http_status < 100 or args.http_status > 599:
        reason_codes.append("http_status_out_of_range")
    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")
    if not payload_valid:
        reason_codes.append("malformed_payload")

    if args.operation == "nonce":
        if broadcast_accepted:
            reason_codes.append("nonce_case_invalid_broadcast_acceptance_flag")
        if duplicate_detected:
            reason_codes.append("nonce_case_invalid_duplicate_flag")
        if args.http_status == 200 and args.nonce_value < 0:
            reason_codes.append("nonce_missing_or_invalid")
    else:
        if args.http_status == 200 and not broadcast_accepted:
            reason_codes.append("broadcast_not_accepted")
        if args.http_status == 409 and not duplicate_detected:
            reason_codes.append("duplicate_flag_missing")
        if duplicate_detected and args.http_status != 409:
            reason_codes.append("duplicate_status_mismatch")
        if args.http_status == 422:
            reason_codes.append("rejected_by_backend")

    if args.http_status in (401, 403):
        reason_codes.append("unauthorized_status")
        if not authorization_present:
            reason_codes.append("authorization_missing")
    elif args.http_status == 429:
        reason_codes.append("rate_limited")
    elif args.http_status in (400, 404):
        reason_codes.append("invalid_request")
    elif args.http_status >= 500:
        reason_codes.append("backend_unavailable")
    elif args.http_status == 409 and args.operation == "nonce":
        reason_codes.append("invalid_request")
    elif args.http_status not in (200, 409, 422):
        reason_codes.append("unexpected_status")

    final_decision = "GO" if not reason_codes else "NO-GO"
    return final_decision, reason_codes


def main() -> int:
    args = parse_args()
    final_decision, reason_codes = evaluate(args)

    report = {
        "schema_version": "kamn.kolme.nonce-broadcast-parity-policy-report.v1",
        "case_id": args.case_id,
        "operation": args.operation,
        "http_status": args.http_status,
        "nonce_value": args.nonce_value,
        "broadcast_accepted": args.broadcast_accepted == "true",
        "duplicate_detected": args.duplicate_detected == "true",
        "payload_valid": args.payload_valid == "true",
        "authorization_present": args.authorization_present == "true",
        "ci_fast_gate": args.ci_fast_gate,
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
    print(f"case_id={args.case_id}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())

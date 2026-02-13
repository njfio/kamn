#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

ALLOWED_CASE_REASON_CODES = {
    "parity_signature_mismatch",
    "parity_recovery_id_mismatch",
    "parity_pubkey_mismatch",
    "parity_probe_failed",
    "vector_payload_invalid",
    "expected_final_decision_invalid",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate Kolme-fork signature parity matrix report policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-vector-id", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.kolme.signature-parity-matrix-report.v1":
        reason_codes.append("schema_version_mismatch")

    status = report.get("status")
    if status not in ("pass", "fail"):
        reason_codes.append("status_invalid")

    vector_count = report.get("vector_count")
    if not isinstance(vector_count, int) or vector_count <= 0:
        reason_codes.append("vector_count_invalid")

    failed_count = report.get("failed_count")
    if not isinstance(failed_count, int) or failed_count < 0:
        reason_codes.append("failed_count_invalid")

    failed_vector_ids = report.get("failed_vector_ids")
    if not isinstance(failed_vector_ids, list):
        reason_codes.append("failed_vector_ids_invalid")

    cases = report.get("cases")
    if not isinstance(cases, list) or not cases:
        reason_codes.append("cases_missing")
    else:
        observed_vector_ids: set[str] = set()
        for entry in cases:
            if not isinstance(entry, dict):
                reason_codes.append("case_entry_invalid")
                continue
            vector_id = entry.get("vector_id")
            if not isinstance(vector_id, str) or not vector_id.strip():
                reason_codes.append("vector_id_invalid")
                continue
            observed_vector_ids.add(vector_id)
            passed = entry.get("passed")
            if not isinstance(passed, bool):
                reason_codes.append(f"case_passed_invalid:{vector_id}")
            observed_decision = entry.get("observed_final_decision")
            if observed_decision not in ("GO", "NO-GO"):
                reason_codes.append(f"case_observed_final_decision_invalid:{vector_id}")
            case_reason_codes = entry.get("reason_codes")
            normalized_case_reason_codes: list[str] = []
            if not isinstance(case_reason_codes, list):
                reason_codes.append(f"case_reason_codes_invalid:{vector_id}")
            else:
                for case_reason_code in case_reason_codes:
                    if not isinstance(case_reason_code, str) or not case_reason_code.strip():
                        reason_codes.append(f"case_reason_codes_invalid:{vector_id}")
                        continue
                    normalized_case_reason_codes.append(case_reason_code)
                for case_reason_code in normalized_case_reason_codes:
                    if (
                        case_reason_code not in ALLOWED_CASE_REASON_CODES
                        and not case_reason_code.startswith("missing_field:")
                    ):
                        reason_codes.append(f"case_reason_code_unrecognized:{vector_id}:{case_reason_code}")
            if observed_decision == "NO-GO" and not normalized_case_reason_codes:
                reason_codes.append(f"case_reason_codes_missing:{vector_id}")
            missing_required_reason_codes = entry.get("missing_required_reason_codes")
            if isinstance(missing_required_reason_codes, list):
                if missing_required_reason_codes:
                    reason_codes.append(f"case_missing_required_reason_codes_present:{vector_id}")
            elif missing_required_reason_codes is not None:
                reason_codes.append(f"case_missing_required_reason_codes_invalid:{vector_id}")

        for required_vector_id in args.require_vector_id:
            if required_vector_id not in observed_vector_ids:
                reason_codes.append(f"required_vector_id_missing:{required_vector_id}")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "pass":
        observed_final_decision = "GO"
        if failed_count not in (0,):
            reason_codes.append("pass_status_failed_count_mismatch")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if isinstance(failed_count, int) and failed_count == 0:
            reason_codes.append("fail_status_failed_count_mismatch")

    if observed_final_decision and observed_final_decision != args.expected_final_decision:
        reason_codes.append("observed_final_decision_mismatch")

    final_decision = "GO" if not reason_codes else "NO-GO"
    return final_decision, reason_codes


def main() -> int:
    args = parse_args()
    report_path = Path(args.report_file).resolve()
    report = json.loads(report_path.read_text(encoding="utf-8"))

    observed_status = report.get("status")
    observed_final_decision = ""
    if observed_status == "pass":
        observed_final_decision = "GO"
    elif observed_status == "fail":
        observed_final_decision = "NO-GO"

    final_decision, reason_codes = evaluate(report, args)
    output = {
        "schema_version": "kamn.kolme.signature-parity-policy-report.v1",
        "report_file": str(report_path),
        "expected_final_decision": args.expected_final_decision,
        "ci_fast_gate": args.ci_fast_gate,
        "required_vector_ids": args.require_vector_id,
        "observed_status": observed_status,
        "observed_final_decision": observed_final_decision,
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_path = Path(args.output_json).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(output, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    status = "ok" if final_decision == "GO" else "fail"
    failed_checks = ",".join(reason_codes) if reason_codes else "none"
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())

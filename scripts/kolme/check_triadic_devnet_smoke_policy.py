#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

VALIDATION_REPORT_SCHEMA_VERSION = "kamn.kolme.triadic-devnet-smoke-validation-report.v1"
POLICY_REPORT_SCHEMA_VERSION = "kamn.kolme.triadic-devnet-smoke-policy-report.v1"
REASON_TAXONOMY_VERSION = "kamn.kolme.triadic-devnet-smoke-policy-reason-taxonomy.v1"
REASON_TAXONOMY_CODES_CSV = (
    "report_schema_version_mismatch,"
    "report_fixture_path_missing,"
    "report_marker_file_missing,"
    "report_required_markers_invalid,"
    "report_observed_markers_invalid,"
    "report_missing_markers_invalid,"
    "report_final_decision_invalid,"
    "report_missing_markers_non_empty,"
    "report_fail_without_missing_markers,"
    "report_final_decision_not_pass,"
    "report_final_decision_not_fail,"
    "report_expected_failure_markers_missing,"
    "required_reason_code_missing"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Validate triadic devnet smoke validation report schema and marker policy "
            "contracts."
        )
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument(
        "--expected-final-decision",
        default="GO",
        choices=["GO", "NO-GO"],
    )
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def _is_non_empty_str(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _validate_marker_list(
    value: object,
    reason_code: str,
    allow_empty: bool,
    reason_codes: list[str],
) -> list[str]:
    if not isinstance(value, list):
        reason_codes.append(reason_code)
        return []
    if not allow_empty and not value:
        reason_codes.append(reason_code)
        return []
    normalized: list[str] = []
    for marker in value:
        if not _is_non_empty_str(marker):
            reason_codes.append(reason_code)
            return []
        normalized.append(marker.strip())
    return normalized


def evaluate_policy(
    report: dict[str, object], expected_final_decision: str
) -> list[str]:
    reason_codes: list[str] = []

    if report.get("schema_version") != VALIDATION_REPORT_SCHEMA_VERSION:
        reason_codes.append("report_schema_version_mismatch")

    if not _is_non_empty_str(report.get("fixture")):
        reason_codes.append("report_fixture_path_missing")
    if not _is_non_empty_str(report.get("marker_file")):
        reason_codes.append("report_marker_file_missing")

    required_markers = _validate_marker_list(
        report.get("required_markers"),
        "report_required_markers_invalid",
        allow_empty=False,
        reason_codes=reason_codes,
    )
    observed_markers = _validate_marker_list(
        report.get("observed_markers"),
        "report_observed_markers_invalid",
        allow_empty=False,
        reason_codes=reason_codes,
    )
    missing_markers = _validate_marker_list(
        report.get("missing_markers"),
        "report_missing_markers_invalid",
        allow_empty=True,
        reason_codes=reason_codes,
    )

    report_final_decision = report.get("final_decision")
    if report_final_decision not in ("PASS", "FAIL"):
        reason_codes.append("report_final_decision_invalid")
    else:
        if report_final_decision == "PASS" and missing_markers:
            reason_codes.append("report_missing_markers_non_empty")
        if report_final_decision == "FAIL" and not missing_markers:
            reason_codes.append("report_fail_without_missing_markers")
        if report_final_decision == "PASS":
            missing_required = [
                marker for marker in required_markers if marker not in set(observed_markers)
            ]
            if missing_required:
                reason_codes.append("report_missing_markers_non_empty")

    if expected_final_decision == "GO":
        if report_final_decision != "PASS":
            reason_codes.append("report_final_decision_not_pass")
        if missing_markers:
            reason_codes.append("report_missing_markers_non_empty")
    else:
        if report_final_decision != "FAIL":
            reason_codes.append("report_final_decision_not_fail")
        if not missing_markers:
            reason_codes.append("report_expected_failure_markers_missing")

    return sorted(set(reason_codes))


def main() -> int:
    args = parse_args()
    report_file = Path(args.report_file).resolve()
    report = json.loads(report_file.read_text(encoding="utf-8"))
    if not isinstance(report, dict):
        raise SystemExit("triadic devnet smoke report must be a JSON object")

    reason_codes = evaluate_policy(report, args.expected_final_decision)
    for required_code in args.require_reason_code:
        if required_code not in reason_codes and not (
            required_code == "triadic_devnet_smoke_policy_passed" and not reason_codes
        ):
            reason_codes.append(f"required_reason_code_missing:{required_code}")

    reason_codes = sorted(set(reason_codes))
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"
        policy_reason_codes = reason_codes
    else:
        status = "ok"
        final_decision = "GO"
        policy_reason_codes = ["triadic_devnet_smoke_policy_passed"]

    report_payload = {
        "schema_version": POLICY_REPORT_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_taxonomy_codes_csv": REASON_TAXONOMY_CODES_CSV,
        "reason_codes": policy_reason_codes,
        "expected_final_decision": args.expected_final_decision,
        "source_report_file": str(report_file),
    }

    if args.output_json:
        output_json = Path(args.output_json).resolve()
        output_json.parent.mkdir(parents=True, exist_ok=True)
        output_json.write_text(
            json.dumps(report_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_taxonomy_codes_csv={REASON_TAXONOMY_CODES_CSV}")
    print(
        "reason_codes="
        + ("none" if not reason_codes else ",".join(policy_reason_codes))
    )
    print(f"source_report_file={report_file}")
    if args.output_json:
        print(f"policy_report_file={Path(args.output_json).resolve()}")

    return 0 if status == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())

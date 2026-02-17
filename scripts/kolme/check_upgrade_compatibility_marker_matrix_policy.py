#!/usr/bin/env python3
"""Validate upgrade compatibility marker matrix policy contracts."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path

REPORT_SCHEMA_VERSION = "kamn.kolme.upgrade-compatibility-marker-matrix-policy-report.v1"
REASON_TAXONOMY_VERSION = "kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "version_report_missing,"
    "fork_policy_report_missing,"
    "version_report_schema_mismatch,"
    "version_report_reason_taxonomy_mismatch,"
    "version_report_reason_codes_csv_mismatch,"
    "version_report_rehearsal_bypass_guard_status_mismatch,"
    "version_report_rehearsal_output_normalization_status_mismatch,"
    "fork_policy_report_schema_mismatch,"
    "fork_policy_report_reason_taxonomy_mismatch,"
    "fork_policy_report_reason_codes_csv_mismatch,"
    "fork_policy_report_rehearsal_bypass_guard_status_mismatch,"
    "fork_policy_report_rehearsal_output_normalization_status_mismatch,"
    "expected_final_decision_mismatch,"
    "ci_fast_gate_failed"
)
REASON_CODES_ORDER = tuple(REASON_CODES_CSV.split(","))

EXPECTED_VERSION_REPORT_SCHEMA = "kamn.kolme.version-compatibility-report.v1"
EXPECTED_VERSION_REASON_TAXONOMY = "kamn.kolme.version-compatibility-reason-taxonomy.v1"
EXPECTED_VERSION_REASON_CODES_CSV = (
    "unsupported_kamn_major,unsupported_kolme_major,kolme_minor_out_of_supported_window,"
    "kolme_minor_too_old_for_kamn_minor,ci_fast_gate_failed"
)

EXPECTED_FORK_POLICY_REPORT_SCHEMA = "kamn.kolme.fork-compatibility-policy-report.v1"
EXPECTED_FORK_POLICY_REASON_TAXONOMY = "kamn.kolme.fork-compatibility-reason-taxonomy.v1"
EXPECTED_FORK_POLICY_REASON_CODES_CSV = (
    "unsupported_upstream_major,unsupported_fork_major,upstream_minor_out_of_supported_window,"
    "fork_minor_out_of_supported_window,fork_release_tag_mismatch,fork_ref_missing,ci_fast_gate_failed"
)


@dataclass
class Inputs:
    version_report_file: Path
    fork_policy_report_file: Path
    expected_final_decision: str
    ci_fast_gate: str
    output_json: Path | None


def parse_args() -> Inputs:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version-report-file", type=Path, required=True)
    parser.add_argument("--fork-policy-report-file", type=Path, required=True)
    parser.add_argument(
        "--expected-final-decision",
        choices=("GO", "NO-GO"),
        required=True,
    )
    parser.add_argument("--ci-fast-gate", choices=("PASS", "FAIL"), required=True)
    parser.add_argument("--output-json", type=Path)
    args = parser.parse_args()
    return Inputs(
        version_report_file=args.version_report_file,
        fork_policy_report_file=args.fork_policy_report_file,
        expected_final_decision=args.expected_final_decision,
        ci_fast_gate=args.ci_fast_gate,
        output_json=args.output_json,
    )


def normalize_reason_codes(reason_codes: list[str]) -> list[str]:
    observed = set(reason_codes)
    return [code for code in REASON_CODES_ORDER if code in observed]


def reason_codes_value(reason_codes: list[str]) -> str:
    return "none" if not reason_codes else ",".join(reason_codes)


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    args = parse_args()
    raw_reason_codes: list[str] = []

    version_report: dict[str, object] | None = None
    if not args.version_report_file.is_file():
        raw_reason_codes.append("version_report_missing")
    else:
        version_report = load_json(args.version_report_file)

    fork_policy_report: dict[str, object] | None = None
    if not args.fork_policy_report_file.is_file():
        raw_reason_codes.append("fork_policy_report_missing")
    else:
        fork_policy_report = load_json(args.fork_policy_report_file)

    if version_report is not None:
        if version_report.get("schema_version") != EXPECTED_VERSION_REPORT_SCHEMA:
            raw_reason_codes.append("version_report_schema_mismatch")
        if version_report.get("reason_taxonomy_version") != EXPECTED_VERSION_REASON_TAXONOMY:
            raw_reason_codes.append("version_report_reason_taxonomy_mismatch")
        if version_report.get("reason_codes_csv") != EXPECTED_VERSION_REASON_CODES_CSV:
            raw_reason_codes.append("version_report_reason_codes_csv_mismatch")
        if version_report.get("upgrade_rehearsal_bypass_guard_status") != "verified":
            raw_reason_codes.append("version_report_rehearsal_bypass_guard_status_mismatch")
        if version_report.get("upgrade_rehearsal_output_normalization_status") != "verified":
            raw_reason_codes.append(
                "version_report_rehearsal_output_normalization_status_mismatch"
            )

    if fork_policy_report is not None:
        if fork_policy_report.get("schema_version") != EXPECTED_FORK_POLICY_REPORT_SCHEMA:
            raw_reason_codes.append("fork_policy_report_schema_mismatch")
        if (
            fork_policy_report.get("reason_taxonomy_version")
            != EXPECTED_FORK_POLICY_REASON_TAXONOMY
        ):
            raw_reason_codes.append("fork_policy_report_reason_taxonomy_mismatch")
        if fork_policy_report.get("reason_codes_csv") != EXPECTED_FORK_POLICY_REASON_CODES_CSV:
            raw_reason_codes.append("fork_policy_report_reason_codes_csv_mismatch")
        if fork_policy_report.get("upgrade_rehearsal_bypass_guard_status") != "verified":
            raw_reason_codes.append("fork_policy_report_rehearsal_bypass_guard_status_mismatch")
        if fork_policy_report.get("upgrade_rehearsal_output_normalization_status") != "verified":
            raw_reason_codes.append(
                "fork_policy_report_rehearsal_output_normalization_status_mismatch"
            )

    if args.ci_fast_gate != "PASS":
        raw_reason_codes.append("ci_fast_gate_failed")

    preliminary_reasons = normalize_reason_codes(raw_reason_codes)
    computed_decision = "GO" if not preliminary_reasons else "NO-GO"
    if computed_decision != args.expected_final_decision:
        raw_reason_codes.append("expected_final_decision_mismatch")

    reasons = normalize_reason_codes(raw_reason_codes)
    reason_value = reason_codes_value(reasons)
    final_decision = "GO" if not reasons else "NO-GO"
    status = "ok" if not reasons else "fail"

    report = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "version_report_file": str(args.version_report_file),
        "fork_policy_report_file": str(args.fork_policy_report_file),
        "expected_final_decision": args.expected_final_decision,
        "ci_fast_gate": args.ci_fast_gate,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes_value": reason_value,
        "reason_codes": reasons,
    }

    if args.output_json is not None:
        args.output_json.parent.mkdir(parents=True, exist_ok=True)
        args.output_json.write_text(
            json.dumps(report, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes_value={reason_value}")
    print(f"report_schema_version={REPORT_SCHEMA_VERSION}")

    return 0 if status == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())

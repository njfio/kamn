#!/usr/bin/env python3
"""Policy checker for sqlite crash-restart local-heavy lane artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

RUNNER_REPORT_SCHEMA = "kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1"
RUNNER_ARTIFACT_SCHEMA = "kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1"
RUNNER_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1"
)
RUNNER_REASON_CODES_CSV = (
    "crash_restart_profile_restart_status_mismatch,"
    "crash_restart_profile_corruption_status_mismatch,"
    "crash_restart_profile_combined_status_mismatch"
)
SOURCE_REPORT_SCHEMA = "kamn.runtime.sqlite-crash-recovery-live-contract-lane-report.v1"

POLICY_REPORT_SCHEMA = "kamn.runtime.sqlite-crash-restart-local-heavy-policy-report.v1"
POLICY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.sqlite-crash-restart-local-heavy-policy-reason-taxonomy.v1"
)
POLICY_REASON_CODES_CSV = (
    "sqlite_crash_restart_policy_required_field_missing,"
    "sqlite_crash_restart_policy_marker_mismatch,"
    "sqlite_crash_restart_policy_reason_taxonomy_mismatch,"
    "sqlite_crash_restart_policy_profile_contract_mismatch,"
    "sqlite_crash_restart_policy_runbook_marker_parity_mismatch,"
    "sqlite_crash_restart_policy_strategy_marker_parity_mismatch,"
    "ci_fast_gate_failed,"
    "sqlite_crash_restart_policy_expected_decision_mismatch,"
    "sqlite_crash_restart_policy_violation"
)
POLICY_REASON_CODES_ORDER = POLICY_REASON_CODES_CSV.split(",")

RUNBOOK_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.sqlite-crash-restart-local-heavy-runbook-reason-taxonomy.v1"
)
RUNBOOK_REASON_CODES_CSV = (
    "sqlite_crash_restart_recovery_taxonomy_mapping_drift_detected,"
    "runbook_marker_parity_mismatch"
)

DEFAULT_RUNBOOK_FILE = Path("docs/deploy/kolme_devnet_ops.md")
DEFAULT_STRATEGY_DOC = Path("docs/ci/strategy.md")

REQUIRED_REPORT_FIELDS = (
    "schema_version",
    "artifact_schema_version",
    "reason_taxonomy_version",
    "reason_codes_csv",
    "status",
    "final_decision",
    "lane_mode",
    "profile",
    "profile_status",
    "reason_code",
    "restart_drill_status",
    "corruption_drill_status",
    "ci_fast_gate",
    "source_report_schema_version",
    "source_command_count",
)

RUNBOOK_REQUIRED_TAXONOMY_MARKERS = (
    "sqlite_crash_restart_recovery_marker_status=verified",
    "sqlite_crash_restart_runbook_reason_taxonomy_version="
    "kamn.runtime.sqlite-crash-restart-local-heavy-runbook-reason-taxonomy.v1",
    "sqlite_crash_restart_runbook_reason_codes_csv="
    "sqlite_crash_restart_recovery_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
)

RUNBOOK_REQUIRED_PARITY_MARKERS = (
    "sqlite_crash_restart_runbook_marker_parity_status=verified",
    "sqlite_crash_restart_runbook_reason_code=none|<reason>",
)

STRATEGY_REQUIRED_MARKERS = (
    "## SQLite Crash-Restart Local-Heavy Policy Checker Contract",
    "bash scripts/runtime/check_sqlite_crash_restart_local_heavy_policy.sh --report-file "
    "/tmp/sqlite-crash-restart-local-heavy-lane-report.json --expected-final-decision GO "
    "--ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md "
    "--strategy-doc docs/ci/strategy.md --output-json "
    "/tmp/sqlite-crash-restart-local-heavy-policy-report.json",
    "bash scripts/runtime/test_check_sqlite_crash_restart_local_heavy_policy.sh",
    "sqlite_crash_restart_local_heavy_policy_reason_taxonomy_version="
    "kamn.runtime.sqlite-crash-restart-local-heavy-policy-reason-taxonomy.v1",
    "sqlite_crash_restart_local_heavy_policy_reason_codes_csv="
    "sqlite_crash_restart_policy_required_field_missing,"
    "sqlite_crash_restart_policy_marker_mismatch,"
    "sqlite_crash_restart_policy_reason_taxonomy_mismatch,"
    "sqlite_crash_restart_policy_profile_contract_mismatch,"
    "sqlite_crash_restart_policy_runbook_marker_parity_mismatch,"
    "sqlite_crash_restart_policy_strategy_marker_parity_mismatch,"
    "ci_fast_gate_failed,"
    "sqlite_crash_restart_policy_expected_decision_mismatch,"
    "sqlite_crash_restart_policy_violation",
    "sqlite_crash_restart_local_heavy_policy_runbook_path=docs/deploy/kolme_devnet_ops.md",
    "sqlite_crash_restart_local_heavy_policy_strategy_doc_path=docs/ci/strategy.md",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report-file", required=True, type=Path)
    parser.add_argument(
        "--expected-final-decision",
        required=True,
        choices=("GO", "NO-GO"),
    )
    parser.add_argument("--ci-fast-gate", required=True, choices=("PASS", "FAIL"))
    parser.add_argument("--runbook-file", type=Path, default=DEFAULT_RUNBOOK_FILE)
    parser.add_argument("--strategy-doc", type=Path, default=DEFAULT_STRATEGY_DOC)
    parser.add_argument("--output-json", type=Path)
    return parser.parse_args()


def load_json_dict(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("report-json-must-be-object")
    return payload


def normalize_reason_codes(raw_reason_codes: list[str]) -> list[str]:
    observed = set(raw_reason_codes)
    normalized: list[str] = []
    for reason_code in POLICY_REASON_CODES_ORDER:
        if reason_code in observed:
            normalized.append(reason_code)
    return normalized


def reason_codes_value(reason_codes: list[str]) -> str:
    return "none" if not reason_codes else ",".join(reason_codes)


def profile_expected_markers(
    profile: str,
) -> tuple[str, str, str]:
    if profile == "restart":
        return ("verified", "not_applicable", "verified")
    if profile == "corruption":
        return ("not_applicable", "verified", "verified")
    if profile == "combined":
        return ("verified", "verified", "verified")
    return ("invalid", "invalid", "invalid")


def evaluate_runbook_marker_parity(runbook_file: Path) -> tuple[str, str]:
    try:
        runbook_text = runbook_file.read_text(encoding="utf-8")
    except OSError:
        return ("violation", "runbook_marker_parity_mismatch")

    missing_taxonomy_markers = [
        marker
        for marker in RUNBOOK_REQUIRED_TAXONOMY_MARKERS
        if marker not in runbook_text
    ]
    if missing_taxonomy_markers:
        return ("violation", "sqlite_crash_restart_recovery_taxonomy_mapping_drift_detected")

    missing_parity_markers = [
        marker for marker in RUNBOOK_REQUIRED_PARITY_MARKERS if marker not in runbook_text
    ]
    if missing_parity_markers:
        return ("violation", "runbook_marker_parity_mismatch")

    return ("verified", "none")


def evaluate_strategy_marker_parity(strategy_doc: Path) -> str:
    try:
        strategy_text = strategy_doc.read_text(encoding="utf-8")
    except OSError:
        return "violation"

    missing_markers = [
        marker for marker in STRATEGY_REQUIRED_MARKERS if marker not in strategy_text
    ]
    return "violation" if missing_markers else "verified"


def main() -> int:
    args = parse_args()

    raw_reason_codes: list[str] = []
    marker_status = "verified"
    profile_contract_status = "verified"
    reason_taxonomy_status = "verified"

    if not args.report_file.exists():
        raw_reason_codes.append("sqlite_crash_restart_policy_required_field_missing")
    else:
        try:
            payload = load_json_dict(args.report_file)
        except Exception:
            payload = {}
            raw_reason_codes.append("sqlite_crash_restart_policy_marker_mismatch")

        missing_fields = [
            field_name
            for field_name in REQUIRED_REPORT_FIELDS
            if field_name not in payload
        ]
        if missing_fields:
            raw_reason_codes.append("sqlite_crash_restart_policy_required_field_missing")

        if payload.get("schema_version") != RUNNER_REPORT_SCHEMA:
            marker_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_marker_mismatch")
        if payload.get("artifact_schema_version") != RUNNER_ARTIFACT_SCHEMA:
            marker_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_marker_mismatch")

        if payload.get("reason_taxonomy_version") != RUNNER_REASON_TAXONOMY_VERSION:
            reason_taxonomy_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_reason_taxonomy_mismatch")
        if payload.get("reason_codes_csv") != RUNNER_REASON_CODES_CSV:
            reason_taxonomy_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_reason_taxonomy_mismatch")

        if payload.get("status") != "pass":
            marker_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_marker_mismatch")
        if payload.get("final_decision") not in ("GO", "NO-GO"):
            marker_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_marker_mismatch")
        if payload.get("ci_fast_gate") != args.ci_fast_gate:
            raw_reason_codes.append("ci_fast_gate_failed")

        if args.ci_fast_gate != "PASS":
            raw_reason_codes.append("ci_fast_gate_failed")

        lane_mode = payload.get("lane_mode")
        if lane_mode not in ("dry-run", "run"):
            profile_contract_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_profile_contract_mismatch")

        profile = payload.get("profile")
        if profile not in ("restart", "corruption", "combined"):
            profile_contract_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_profile_contract_mismatch")
        else:
            expected_restart, expected_corruption, expected_profile_status = (
                profile_expected_markers(profile)
            )
            if payload.get("restart_drill_status") != expected_restart:
                profile_contract_status = "violation"
                raw_reason_codes.append("sqlite_crash_restart_policy_profile_contract_mismatch")
            if payload.get("corruption_drill_status") != expected_corruption:
                profile_contract_status = "violation"
                raw_reason_codes.append("sqlite_crash_restart_policy_profile_contract_mismatch")
            if payload.get("profile_status") != expected_profile_status:
                profile_contract_status = "violation"
                raw_reason_codes.append("sqlite_crash_restart_policy_profile_contract_mismatch")

        if payload.get("reason_code") != "none":
            profile_contract_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_profile_contract_mismatch")

        if payload.get("source_report_schema_version") != SOURCE_REPORT_SCHEMA:
            marker_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_marker_mismatch")

        source_command_count = payload.get("source_command_count")
        if (
            not isinstance(source_command_count, int)
            or isinstance(source_command_count, bool)
            or source_command_count < 0
        ):
            marker_status = "violation"
            raw_reason_codes.append("sqlite_crash_restart_policy_marker_mismatch")

        if payload.get("final_decision") != args.expected_final_decision:
            raw_reason_codes.append("sqlite_crash_restart_policy_expected_decision_mismatch")

    runbook_parity_status, runbook_reason_code = evaluate_runbook_marker_parity(
        args.runbook_file
    )
    if runbook_parity_status != "verified":
        raw_reason_codes.append("sqlite_crash_restart_policy_runbook_marker_parity_mismatch")

    strategy_parity_status = evaluate_strategy_marker_parity(args.strategy_doc)
    if strategy_parity_status != "verified":
        raw_reason_codes.append("sqlite_crash_restart_policy_strategy_marker_parity_mismatch")

    normalized_reason_codes = normalize_reason_codes(raw_reason_codes)
    if raw_reason_codes and not normalized_reason_codes:
        normalized_reason_codes = ["sqlite_crash_restart_policy_violation"]

    status = "pass" if not normalized_reason_codes else "fail"
    final_decision = "GO" if status == "pass" else "NO-GO"
    if final_decision != args.expected_final_decision:
        if "sqlite_crash_restart_policy_expected_decision_mismatch" not in normalized_reason_codes:
            normalized_reason_codes.append(
                "sqlite_crash_restart_policy_expected_decision_mismatch"
            )
        normalized_reason_codes = normalize_reason_codes(normalized_reason_codes)
        status = "pass" if not normalized_reason_codes else "fail"
        final_decision = "GO" if status == "pass" else "NO-GO"

    reason_value = reason_codes_value(normalized_reason_codes)
    policy_status = "verified" if status == "pass" else "violation"
    promotion_reason_code = "none" if status == "pass" else normalized_reason_codes[0]
    promotion_reason_mapping_status = (
        "verified"
        if (
            (status == "pass" and promotion_reason_code == "none")
            or (
                status == "fail"
                and promotion_reason_code in normalized_reason_codes
                and promotion_reason_code != "none"
            )
        )
        else "violation"
    )

    report_payload = {
        "schema_version": POLICY_REPORT_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": POLICY_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": POLICY_REASON_CODES_CSV,
        "reason_codes": normalized_reason_codes,
        "reason_codes_value": reason_value,
        "sqlite_crash_restart_policy_status": policy_status,
        "sqlite_crash_restart_recovery_marker_status": marker_status,
        "sqlite_crash_restart_reason_taxonomy_status": reason_taxonomy_status,
        "sqlite_crash_restart_profile_contract_status": profile_contract_status,
        "sqlite_crash_restart_runbook_marker_parity_status": runbook_parity_status,
        "sqlite_crash_restart_runbook_reason_taxonomy_version": RUNBOOK_REASON_TAXONOMY_VERSION,
        "sqlite_crash_restart_runbook_reason_codes_csv": RUNBOOK_REASON_CODES_CSV,
        "sqlite_crash_restart_runbook_reason_code": runbook_reason_code,
        "sqlite_crash_restart_strategy_marker_parity_status": strategy_parity_status,
        "promotion_decision_reason_mapping_status": promotion_reason_mapping_status,
        "promotion_decision_reason_code": promotion_reason_code,
        "inputs": {
            "report_file": str(args.report_file),
            "expected_final_decision": args.expected_final_decision,
            "ci_fast_gate": args.ci_fast_gate,
            "runbook_file": str(args.runbook_file),
            "strategy_doc": str(args.strategy_doc),
        },
    }

    if args.output_json:
        args.output_json.parent.mkdir(parents=True, exist_ok=True)
        args.output_json.write_text(
            json.dumps(report_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={POLICY_REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={POLICY_REASON_CODES_CSV}")
    print(f"reason_codes_value={reason_value}")
    print(f"sqlite_crash_restart_policy_status={policy_status}")
    print(f"sqlite_crash_restart_recovery_marker_status={marker_status}")
    print(f"sqlite_crash_restart_reason_taxonomy_status={reason_taxonomy_status}")
    print(f"sqlite_crash_restart_profile_contract_status={profile_contract_status}")
    print(
        "sqlite_crash_restart_runbook_marker_parity_status="
        f"{runbook_parity_status}"
    )
    print(
        "sqlite_crash_restart_runbook_reason_taxonomy_version="
        f"{RUNBOOK_REASON_TAXONOMY_VERSION}"
    )
    print(
        "sqlite_crash_restart_runbook_reason_codes_csv="
        f"{RUNBOOK_REASON_CODES_CSV}"
    )
    print(f"sqlite_crash_restart_runbook_reason_code={runbook_reason_code}")
    print(
        "sqlite_crash_restart_strategy_marker_parity_status="
        f"{strategy_parity_status}"
    )
    print(
        "promotion_decision_reason_mapping_status="
        f"{promotion_reason_mapping_status}"
    )
    print(f"promotion_decision_reason_code={promotion_reason_code}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Sqlite crash-restart local-heavy lane runner contract."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
SOURCE_LANE = ROOT_DIR / "scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh"

REPORT_SCHEMA_VERSION = "kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1"
ARTIFACT_SCHEMA_VERSION = "kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1"
REASON_TAXONOMY_VERSION = "kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "crash_restart_profile_restart_status_mismatch,"
    "crash_restart_profile_corruption_status_mismatch,"
    "crash_restart_profile_combined_status_mismatch"
)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default="combined")
    parser.add_argument("--mode", default="dry-run")
    parser.add_argument("--ci-fast-gate", default="PASS")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_SQLITE_CRASH_RESTART_LOCAL_HEAVY_MAX_SECONDS", "240"),
    )
    parser.add_argument("--output-json", default="")
    return parser.parse_args(argv)


def parse_positive_int(raw_value: str) -> int:
    if not raw_value.isdigit():
        raise ValueError("max-seconds must be an integer")
    max_seconds = int(raw_value)
    if max_seconds <= 0:
        raise ValueError("max-seconds must be greater than zero")
    return max_seconds


def run_lane(argv: list[str]) -> int:
    args = parse_args(argv)

    profile = args.profile
    mode = args.mode
    ci_fast_gate = args.ci_fast_gate

    if profile not in {"restart", "corruption", "combined"}:
        print("profile must be restart, corruption, or combined", file=sys.stderr)
        return 1
    if mode not in {"dry-run", "run"}:
        print("mode must be dry-run or run", file=sys.stderr)
        return 1
    if ci_fast_gate not in {"PASS", "FAIL"}:
        print("ci-fast-gate must be PASS or FAIL", file=sys.stderr)
        return 1

    try:
        max_seconds = parse_positive_int(args.max_seconds)
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    if not SOURCE_LANE.exists() or not os.access(SOURCE_LANE, os.X_OK):
        print(f"expected required executable script '{SOURCE_LANE}'", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="sqlite-crash-restart-lane-") as temp_dir:
        temp_path = Path(temp_dir)
        source_report = temp_path / "sqlite-crash-restart-source-report.json"
        source_policy = temp_path / "sqlite-crash-restart-source-policy.json"
        source_summary = temp_path / "sqlite-crash-restart-source-summary.json"
        source_convergence = temp_path / "sqlite-crash-restart-source-convergence.json"

        completed = subprocess.run(
            [
                "bash",
                str(SOURCE_LANE),
                "--mode",
                mode,
                "--max-seconds",
                str(max_seconds),
                "--ci-fast-gate",
                ci_fast_gate,
                "--output-json",
                str(source_report),
                "--policy-output-json",
                str(source_policy),
                "--summary-output-json",
                str(source_summary),
                "--convergence-output-json",
                str(source_convergence),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if completed.returncode != 0:
            if completed.stderr:
                print(completed.stderr, end="", file=sys.stderr)
            elif completed.stdout:
                print(completed.stdout, end="", file=sys.stderr)
            return completed.returncode

        source_payload = json.loads(source_report.read_text(encoding="utf-8"))

    source_status = source_payload.get("status")
    source_final_decision = source_payload.get("final_decision")
    source_wal_append_status = source_payload.get("wal_append_status")
    source_wal_checkpoint_status = source_payload.get("wal_checkpoint_status")
    source_append_checkpoint_integrity_status = source_payload.get(
        "append_checkpoint_integrity_status"
    )
    source_journal_replay_status = source_payload.get(
        "journal_replay_drift_detection_status"
    )
    source_checkpoint_bypass_status = source_payload.get(
        "checkpoint_divergence_bypass_rejection_status"
    )
    source_readiness_progress_status = source_payload.get(
        "crash_recovery_readiness_progress_status"
    )
    source_snapshot_parity_status = source_payload.get("snapshot_parity_status")

    restart_markers_verified = all(
        value == "verified"
        for value in [
            source_journal_replay_status,
            source_checkpoint_bypass_status,
            source_readiness_progress_status,
            source_snapshot_parity_status,
        ]
    )
    corruption_markers_verified = all(
        value == "verified"
        for value in [
            source_wal_append_status,
            source_wal_checkpoint_status,
            source_append_checkpoint_integrity_status,
        ]
    )

    if profile == "restart":
        restart_drill_status = "verified" if restart_markers_verified else "failed"
        corruption_drill_status = "not_applicable"
        profile_status = "verified" if restart_drill_status == "verified" else "failed"
        reason_code = (
            "none"
            if profile_status == "verified"
            else "crash_restart_profile_restart_status_mismatch"
        )
    elif profile == "corruption":
        restart_drill_status = "not_applicable"
        corruption_drill_status = "verified" if corruption_markers_verified else "failed"
        profile_status = "verified" if corruption_drill_status == "verified" else "failed"
        reason_code = (
            "none"
            if profile_status == "verified"
            else "crash_restart_profile_corruption_status_mismatch"
        )
    else:
        restart_drill_status = "verified" if restart_markers_verified else "failed"
        corruption_drill_status = "verified" if corruption_markers_verified else "failed"
        profile_status = (
            "verified"
            if restart_drill_status == "verified"
            and corruption_drill_status == "verified"
            else "failed"
        )
        reason_code = (
            "none"
            if profile_status == "verified"
            else "crash_restart_profile_combined_status_mismatch"
        )

    status = (
        "pass"
        if profile_status == "verified"
        and source_status == "pass"
        and source_final_decision == "GO"
        else "fail"
    )
    final_decision = "GO" if status == "pass" else "NO-GO"
    if final_decision == "GO":
        reason_code = "none"

    report = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "artifact_schema_version": ARTIFACT_SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "status": status,
        "final_decision": final_decision,
        "lane_mode": mode,
        "profile": profile,
        "profile_status": profile_status,
        "reason_code": reason_code,
        "restart_drill_status": restart_drill_status,
        "corruption_drill_status": corruption_drill_status,
        "ci_fast_gate": ci_fast_gate,
        "source_report_schema_version": source_payload.get("schema_version", "missing"),
        "source_command_count": source_payload.get("command_count", 0),
        "source_policy_status": source_payload.get(
            "sqlite_crash_recovery_policy_status", "missing"
        ),
    }

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"status={report['status']}")
    print(f"final_decision={report['final_decision']}")
    print(f"lane_mode={report['lane_mode']}")
    print(f"profile={report['profile']}")
    print(f"profile_status={report['profile_status']}")
    print(f"reason_code={report['reason_code']}")
    print(f"restart_drill_status={report['restart_drill_status']}")
    print(f"corruption_drill_status={report['corruption_drill_status']}")
    print(f"schema_version={report['schema_version']}")
    print(f"artifact_schema_version={report['artifact_schema_version']}")
    print(f"reason_taxonomy_version={report['reason_taxonomy_version']}")
    print(f"reason_codes_csv={report['reason_codes_csv']}")
    print(f"source_report_schema_version={report['source_report_schema_version']}")
    print(f"source_command_count={report['source_command_count']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(run_lane(sys.argv[1:]))

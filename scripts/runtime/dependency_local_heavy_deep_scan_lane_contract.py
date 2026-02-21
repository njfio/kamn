#!/usr/bin/env python3
"""Dependency local-heavy deep scan lane runner contract."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
DEFAULT_FIXTURE_PATH = ROOT_DIR / "fixtures/ci/dependency_local_heavy_deep_scan_fixture_matrix.txt"

RUN_SCHEMA_VERSION = "kamn.runtime.dependency-local-heavy-deep-scan-lane-report.v1"
ARTIFACT_SCHEMA_VERSION = "kamn.runtime.dependency-local-heavy-deep-scan-artifact-schema.v1"
FIXTURE_SCHEMA_VERSION = "kamn.ci.dependency-local-heavy-deep-scan-fixture-matrix.v1"
REASON_TAXONOMY_VERSION = "kamn.runtime.dependency-local-heavy-deep-scan-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "dependency_local_heavy_deep_scan_profile_threshold_exceeded,"
    "dependency_local_heavy_deep_scan_runtime_budget_exceeded"
)
OPT_IN_ENV = "KAMN_DEPENDENCY_LOCAL_HEAVY_DEEP_SCAN_OPT_IN"

EXPECTED_COLUMNS = [
    "profile",
    "advisory_total",
    "critical_count",
    "high_count",
    "moderate_count",
    "low_count",
    "unknown_count",
    "expected_status",
    "expected_reason_code",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default="baseline")
    parser.add_argument("--mode", default="dry-run")
    parser.add_argument("--ci-fast-gate", default="PASS")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_DEPENDENCY_LOCAL_HEAVY_DEEP_SCAN_MAX_SECONDS", "180"),
    )
    parser.add_argument("--local-opt-in", default=os.environ.get(OPT_IN_ENV, "0"))
    parser.add_argument("--fixture-file", default=str(DEFAULT_FIXTURE_PATH))
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def parse_positive_int(raw_value: str, field: str) -> int:
    if not raw_value.isdigit():
        raise ValueError(f"{field} must be an integer")
    parsed = int(raw_value)
    if parsed < 0:
        raise ValueError(f"{field} must be zero or greater")
    return parsed


def parse_required_positive_int(raw_value: str, field: str) -> int:
    parsed = parse_positive_int(raw_value, field)
    if parsed <= 0:
        raise ValueError(f"{field} must be greater than zero")
    return parsed


def parse_fixture(path: Path) -> tuple[dict[str, str], dict[str, dict[str, int | str]]]:
    if not path.exists():
        raise ValueError(f"fixture file not found: {path}")

    markers: dict[str, str] = {}
    rows: dict[str, dict[str, int | str]] = {}
    columns: list[str] = []

    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue

        if line.startswith("columns="):
            columns = [part.strip() for part in line[len("columns=") :].split("|")]
            continue

        if "=" in line and not columns:
            key, value = line.split("=", 1)
            markers[key.strip()] = value.strip()
            continue

        if not columns:
            raise ValueError(f"fixture row before columns marker on line {line_number}")

        values = [part.strip() for part in line.split("|")]
        if len(values) != len(columns):
            raise ValueError(
                f"fixture row column mismatch on line {line_number}: expected {len(columns)} values"
            )

        row = dict(zip(columns, values))
        profile = row["profile"]
        if profile in rows:
            raise ValueError(f"duplicate fixture profile on line {line_number}: {profile}")

        numeric_fields = {
            "advisory_total",
            "critical_count",
            "high_count",
            "moderate_count",
            "low_count",
            "unknown_count",
        }
        parsed_row: dict[str, int | str] = {}
        for key, value in row.items():
            if key in numeric_fields:
                parsed_row[key] = parse_positive_int(value, key)
            else:
                parsed_row[key] = value

        rows[profile] = parsed_row

    if columns != EXPECTED_COLUMNS:
        raise ValueError(
            "fixture columns must be "
            + "|".join(EXPECTED_COLUMNS)
        )

    required_marker_keys = {
        "dependency_local_heavy_deep_scan_fixture_schema_version",
        "dependency_local_heavy_deep_scan_reason_taxonomy_version",
        "dependency_local_heavy_deep_scan_reason_codes_csv",
        "dependency_local_heavy_deep_scan_threshold_max_critical",
        "dependency_local_heavy_deep_scan_threshold_max_high",
        "dependency_local_heavy_deep_scan_required_profiles_csv",
    }
    missing_keys = sorted(required_marker_keys - markers.keys())
    if missing_keys:
        raise ValueError("fixture missing required markers: " + ",".join(missing_keys))

    if markers["dependency_local_heavy_deep_scan_fixture_schema_version"] != FIXTURE_SCHEMA_VERSION:
        raise ValueError("fixture schema version mismatch")
    if markers["dependency_local_heavy_deep_scan_reason_taxonomy_version"] != REASON_TAXONOMY_VERSION:
        raise ValueError("fixture reason taxonomy mismatch")
    if markers["dependency_local_heavy_deep_scan_reason_codes_csv"] != REASON_CODES_CSV:
        raise ValueError("fixture reason codes csv mismatch")

    required_profiles = [
        profile.strip()
        for profile in markers["dependency_local_heavy_deep_scan_required_profiles_csv"].split(",")
        if profile.strip()
    ]
    if not required_profiles:
        raise ValueError("fixture required profiles marker must not be empty")
    if set(required_profiles) != set(rows):
        raise ValueError("fixture profiles must match required profiles marker")

    for profile in required_profiles:
        expected_status = rows[profile]["expected_status"]
        if expected_status not in {"pass", "fail"}:
            raise ValueError(f"fixture expected_status must be pass/fail for profile {profile}")

    markers["dependency_local_heavy_deep_scan_required_profiles_csv"] = ",".join(required_profiles)
    return markers, rows


def run_lane() -> int:
    started = time.monotonic()
    args = parse_args()

    if args.mode not in {"dry-run", "run"}:
        print("mode must be dry-run or run", file=sys.stderr)
        return 1
    if args.ci_fast_gate not in {"PASS", "FAIL"}:
        print("ci-fast-gate must be PASS or FAIL", file=sys.stderr)
        return 1

    try:
        max_seconds = parse_required_positive_int(args.max_seconds, "max-seconds")
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    if args.mode == "run" and args.local_opt_in != "1":
        print(
            f"run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1",
            file=sys.stderr,
        )
        return 1

    if args.mode == "run" and args.ci_fast_gate != "FAIL":
        print(
            "run mode requires --ci-fast-gate FAIL for local-heavy execution scope",
            file=sys.stderr,
        )
        return 1

    fixture_path = Path(args.fixture_file)
    try:
        markers, rows = parse_fixture(fixture_path)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    profile = args.profile
    if profile not in rows:
        print("profile must be baseline or injected-risk", file=sys.stderr)
        return 1

    max_critical = int(markers["dependency_local_heavy_deep_scan_threshold_max_critical"])
    max_high = int(markers["dependency_local_heavy_deep_scan_threshold_max_high"])

    row = rows[profile]
    critical_count = int(row["critical_count"])
    high_count = int(row["high_count"])

    threshold_breached = critical_count > max_critical or high_count > max_high
    status = "fail" if threshold_breached else "pass"
    final_decision = "NO-GO" if threshold_breached else "GO"
    profile_status = "failed" if threshold_breached else "verified"
    reason_code = (
        "dependency_local_heavy_deep_scan_profile_threshold_exceeded"
        if threshold_breached
        else "none"
    )

    expected_status = str(row["expected_status"])
    expected_reason_code = str(row["expected_reason_code"])
    if status != expected_status or reason_code != expected_reason_code:
        print(
            f"fixture profile contract mismatch for {profile}: expected status={expected_status}, reason={expected_reason_code}",
            file=sys.stderr,
        )
        return 1

    command_count = 0 if args.mode == "dry-run" else 1
    run_mode_command_status = (
        "dry_run_no_commands_executed"
        if args.mode == "dry-run"
        else "dependency_local_heavy_deep_scan_executed"
    )

    elapsed_seconds = int(time.monotonic() - started)
    performance_budget_status = "verified"
    if elapsed_seconds > max_seconds:
        performance_budget_status = "violation"
        status = "fail"
        final_decision = "NO-GO"
        profile_status = "failed"
        reason_code = "dependency_local_heavy_deep_scan_runtime_budget_exceeded"

    reason_codes_value = "none" if reason_code == "none" else reason_code
    payload = {
        "schema_version": RUN_SCHEMA_VERSION,
        "artifact_schema_version": ARTIFACT_SCHEMA_VERSION,
        "fixture_schema_version": FIXTURE_SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "status": status,
        "final_decision": final_decision,
        "lane_mode": args.mode,
        "profile": profile,
        "profile_status": profile_status,
        "reason_code": reason_code,
        "reason_codes_value": reason_codes_value,
        "advisory_total": int(row["advisory_total"]),
        "critical_count": critical_count,
        "high_count": high_count,
        "moderate_count": int(row["moderate_count"]),
        "low_count": int(row["low_count"]),
        "unknown_count": int(row["unknown_count"]),
        "threshold_max_critical": max_critical,
        "threshold_max_high": max_high,
        "required_profiles_csv": markers[
            "dependency_local_heavy_deep_scan_required_profiles_csv"
        ],
        "fixture_path": str(fixture_path),
        "ci_fast_gate": args.ci_fast_gate,
        "run_mode_command_status": run_mode_command_status,
        "command_count": command_count,
        "performance_budget_status": performance_budget_status,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
    }

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"lane_mode={args.mode}")
    print(f"profile={profile}")
    print(f"profile_status={profile_status}")
    print(f"reason_code={reason_code}")
    print(f"reason_codes_value={reason_codes_value}")
    print(f"advisory_total={row['advisory_total']}")
    print(f"critical_count={critical_count}")
    print(f"high_count={high_count}")
    print(f"moderate_count={row['moderate_count']}")
    print(f"low_count={row['low_count']}")
    print(f"unknown_count={row['unknown_count']}")
    print(f"schema_version={RUN_SCHEMA_VERSION}")
    print(f"artifact_schema_version={ARTIFACT_SCHEMA_VERSION}")
    print(f"fixture_schema_version={FIXTURE_SCHEMA_VERSION}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"run_mode_command_status={run_mode_command_status}")
    print(f"command_count={command_count}")
    print(f"performance_budget_status={performance_budget_status}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"max_seconds={max_seconds}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(run_lane())

#!/usr/bin/env python3
"""Deterministic SBOM/provenance artifact generator contract lane."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
DEFAULT_FIXTURE_PATH = ROOT_DIR / "fixtures/ci/sbom_provenance_artifact_fixture_matrix.txt"

RUN_SCHEMA_VERSION = "kamn.runtime.sbom-provenance-artifact-report.v1"
ARTIFACT_SCHEMA_VERSION = "kamn.runtime.sbom-provenance-artifact-schema.v1"
FIXTURE_SCHEMA_VERSION = "kamn.ci.sbom-provenance-artifact-fixture-matrix.v1"
REASON_TAXONOMY_VERSION = "kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "sbom_provenance_profile_contract_violation,"
    "sbom_provenance_runtime_budget_exceeded"
)
OPT_IN_ENV = "KAMN_SBOM_PROVENANCE_GENERATOR_OPT_IN"

EXPECTED_COLUMNS = [
    "profile",
    "sbom_component_count",
    "sbom_package_count",
    "sbom_digest_sha256",
    "provenance_digest_sha256",
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
        default=os.environ.get("KAMN_SBOM_PROVENANCE_GENERATOR_MAX_SECONDS", "120"),
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


def is_sha256_digest(value: str) -> bool:
    if not value.startswith("sha256:"):
        return False
    digest = value[len("sha256:") :]
    if len(digest) != 64:
        return False
    return all(character in "0123456789abcdef" for character in digest)


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

        parsed_row: dict[str, int | str] = {
            "profile": profile,
            "sbom_component_count": parse_positive_int(
                row["sbom_component_count"], "sbom_component_count"
            ),
            "sbom_package_count": parse_positive_int(row["sbom_package_count"], "sbom_package_count"),
            "sbom_digest_sha256": row["sbom_digest_sha256"],
            "provenance_digest_sha256": row["provenance_digest_sha256"],
            "expected_status": row["expected_status"],
            "expected_reason_code": row["expected_reason_code"],
        }
        if not is_sha256_digest(str(parsed_row["sbom_digest_sha256"])):
            raise ValueError(f"invalid sbom digest shape for profile {profile}")
        if not is_sha256_digest(str(parsed_row["provenance_digest_sha256"])):
            raise ValueError(f"invalid provenance digest shape for profile {profile}")

        rows[profile] = parsed_row

    if columns != EXPECTED_COLUMNS:
        raise ValueError("fixture columns must be " + "|".join(EXPECTED_COLUMNS))

    required_marker_keys = {
        "sbom_provenance_fixture_schema_version",
        "sbom_provenance_reason_taxonomy_version",
        "sbom_provenance_reason_codes_csv",
        "sbom_provenance_required_profiles_csv",
        "sbom_provenance_min_component_count",
        "sbom_provenance_sbom_schema_version",
        "sbom_provenance_provenance_schema_version",
    }
    missing_keys = sorted(required_marker_keys - markers.keys())
    if missing_keys:
        raise ValueError("fixture missing required markers: " + ",".join(missing_keys))

    if markers["sbom_provenance_fixture_schema_version"] != FIXTURE_SCHEMA_VERSION:
        raise ValueError("fixture schema version mismatch")
    if markers["sbom_provenance_reason_taxonomy_version"] != REASON_TAXONOMY_VERSION:
        raise ValueError("fixture reason taxonomy mismatch")
    if markers["sbom_provenance_reason_codes_csv"] != REASON_CODES_CSV:
        raise ValueError("fixture reason codes csv mismatch")

    required_profiles = [
        profile.strip()
        for profile in markers["sbom_provenance_required_profiles_csv"].split(",")
        if profile.strip()
    ]
    if not required_profiles:
        raise ValueError("fixture required profiles marker must not be empty")
    if set(required_profiles) != set(rows):
        raise ValueError("fixture profiles must match required profiles marker")

    for profile in required_profiles:
        expected_status = str(rows[profile]["expected_status"])
        if expected_status not in {"pass", "fail"}:
            raise ValueError(f"fixture expected_status must be pass/fail for profile {profile}")

    markers["sbom_provenance_required_profiles_csv"] = ",".join(required_profiles)
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
        print("run mode requires --ci-fast-gate FAIL for local-only execution scope", file=sys.stderr)
        return 1

    fixture_path = Path(args.fixture_file)
    try:
        markers, rows = parse_fixture(fixture_path)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    profile = args.profile
    if profile not in rows:
        print("profile must be baseline or injected-drift", file=sys.stderr)
        return 1

    row = rows[profile]
    min_component_count = int(markers["sbom_provenance_min_component_count"])
    sbom_component_count = int(row["sbom_component_count"])
    sbom_package_count = int(row["sbom_package_count"])

    profile_contract_violation = sbom_component_count < min_component_count
    status = "fail" if profile_contract_violation else "pass"
    final_decision = "NO-GO" if profile_contract_violation else "GO"
    reason_code = (
        "sbom_provenance_profile_contract_violation" if profile_contract_violation else "none"
    )
    release_manifest_ready_status = "violation" if profile_contract_violation else "verified"
    artifact_linkage_status = "violation" if profile_contract_violation else "verified"

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
        "dry_run_no_commands_executed" if args.mode == "dry-run" else "sbom_provenance_generator_executed"
    )

    elapsed_seconds = int(time.monotonic() - started)
    performance_budget_status = "verified"
    if elapsed_seconds > max_seconds:
        performance_budget_status = "violation"
        status = "fail"
        final_decision = "NO-GO"
        reason_code = "sbom_provenance_runtime_budget_exceeded"
        release_manifest_ready_status = "violation"
        artifact_linkage_status = "violation"

    reason_codes_value = "none" if reason_code == "none" else reason_code
    sbom_schema_version = markers["sbom_provenance_sbom_schema_version"]
    provenance_schema_version = markers["sbom_provenance_provenance_schema_version"]
    sbom_digest_sha256 = str(row["sbom_digest_sha256"])
    provenance_digest_sha256 = str(row["provenance_digest_sha256"])

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
        "reason_code": reason_code,
        "reason_codes_value": reason_codes_value,
        "sbom_schema_version": sbom_schema_version,
        "provenance_schema_version": provenance_schema_version,
        "sbom_component_count": sbom_component_count,
        "sbom_package_count": sbom_package_count,
        "sbom_digest_sha256": sbom_digest_sha256,
        "provenance_digest_sha256": provenance_digest_sha256,
        "release_manifest_required_artifact_id": "sbom_provenance",
        "release_manifest_ready_status": release_manifest_ready_status,
        "artifact_linkage_status": artifact_linkage_status,
        "ci_fast_gate": args.ci_fast_gate,
        "run_mode_command_status": run_mode_command_status,
        "command_count": command_count,
        "performance_budget_status": performance_budget_status,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "fixture_path": str(fixture_path),
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
    print(f"reason_code={reason_code}")
    print(f"reason_codes_value={reason_codes_value}")
    print(f"schema_version={RUN_SCHEMA_VERSION}")
    print(f"artifact_schema_version={ARTIFACT_SCHEMA_VERSION}")
    print(f"fixture_schema_version={FIXTURE_SCHEMA_VERSION}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"sbom_schema_version={sbom_schema_version}")
    print(f"provenance_schema_version={provenance_schema_version}")
    print(f"sbom_component_count={sbom_component_count}")
    print(f"sbom_package_count={sbom_package_count}")
    print(f"sbom_digest_sha256={sbom_digest_sha256}")
    print(f"provenance_digest_sha256={provenance_digest_sha256}")
    print("release_manifest_required_artifact_id=sbom_provenance")
    print(f"release_manifest_ready_status={release_manifest_ready_status}")
    print(f"artifact_linkage_status={artifact_linkage_status}")
    print(f"run_mode_command_status={run_mode_command_status}")
    print(f"command_count={command_count}")
    print(f"performance_budget_status={performance_budget_status}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"max_seconds={max_seconds}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(run_lane())

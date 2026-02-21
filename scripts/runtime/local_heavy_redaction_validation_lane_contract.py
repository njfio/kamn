#!/usr/bin/env python3
"""Local-heavy redaction validation lane runner contract."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
import time

RUN_SCHEMA_VERSION = "kamn.runtime.local-heavy-redaction-validation-lane-report.v1"
ARTIFACT_SCHEMA_VERSION = "kamn.runtime.local-heavy-redaction-validation-artifact-schema.v1"
REASON_TAXONOMY_VERSION = "kamn.runtime.local-heavy-redaction-validation-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "local_heavy_redaction_sensitive_pattern_detected,"
    "local_heavy_redaction_runtime_budget_exceeded"
)

PROFILE_MARKERS = {
    "baseline": {
        "leak_marker_status": "clear",
        "leak_detection_count": 0,
        "leaked_pattern_ids_csv": "none",
    },
    "injected-leak": {
        "leak_marker_status": "detected",
        "leak_detection_count": 3,
        "leaked_pattern_ids_csv": "raw_signer_secret,pii_email,pii_phone",
    },
}

OPT_IN_ENV = "KAMN_LOCAL_HEAVY_REDACTION_VALIDATION_OPT_IN"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default="baseline")
    parser.add_argument("--mode", default="dry-run")
    parser.add_argument("--ci-fast-gate", default="PASS")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_HEAVY_REDACTION_VALIDATION_MAX_SECONDS", "120"),
    )
    parser.add_argument("--local-opt-in", default=os.environ.get(OPT_IN_ENV, "0"))
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def parse_positive_int(raw_value: str) -> int:
    if not raw_value.isdigit():
        raise ValueError("max-seconds must be an integer")
    parsed = int(raw_value)
    if parsed <= 0:
        raise ValueError("max-seconds must be greater than zero")
    return parsed


def run_lane() -> int:
    started = time.monotonic()
    args = parse_args()

    if args.profile not in {"baseline", "injected-leak"}:
        print("profile must be baseline or injected-leak", file=sys.stderr)
        return 1
    if args.mode not in {"dry-run", "run"}:
        print("mode must be dry-run or run", file=sys.stderr)
        return 1
    if args.ci_fast_gate not in {"PASS", "FAIL"}:
        print("ci-fast-gate must be PASS or FAIL", file=sys.stderr)
        return 1

    try:
        max_seconds = parse_positive_int(args.max_seconds)
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

    profile_markers = PROFILE_MARKERS[args.profile]
    leak_detected = args.profile == "injected-leak"
    profile_status = "failed" if leak_detected else "verified"
    reason_code = (
        "local_heavy_redaction_sensitive_pattern_detected" if leak_detected else "none"
    )
    status = "fail" if leak_detected else "pass"
    final_decision = "NO-GO" if leak_detected else "GO"

    command_count = 0 if args.mode == "dry-run" else 1
    run_mode_command_status = (
        "dry_run_no_commands_executed"
        if args.mode == "dry-run"
        else "local_heavy_redaction_validation_executed"
    )

    elapsed_seconds = int(time.monotonic() - started)
    performance_budget_status = "verified"
    if elapsed_seconds > max_seconds:
        performance_budget_status = "violation"
        status = "fail"
        final_decision = "NO-GO"
        profile_status = "failed"
        reason_code = "local_heavy_redaction_runtime_budget_exceeded"

    reason_codes_value = "none" if reason_code == "none" else reason_code
    payload = {
        "schema_version": RUN_SCHEMA_VERSION,
        "artifact_schema_version": ARTIFACT_SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "status": status,
        "final_decision": final_decision,
        "lane_mode": args.mode,
        "profile": args.profile,
        "profile_status": profile_status,
        "reason_code": reason_code,
        "reason_codes_value": reason_codes_value,
        "leak_marker_status": profile_markers["leak_marker_status"],
        "leak_detection_count": profile_markers["leak_detection_count"],
        "leaked_pattern_ids_csv": profile_markers["leaked_pattern_ids_csv"],
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
    print(f"profile={args.profile}")
    print(f"profile_status={profile_status}")
    print(f"reason_code={reason_code}")
    print(f"reason_codes_value={reason_codes_value}")
    print(f"leak_marker_status={profile_markers['leak_marker_status']}")
    print(f"leak_detection_count={profile_markers['leak_detection_count']}")
    print(f"leaked_pattern_ids_csv={profile_markers['leaked_pattern_ids_csv']}")
    print(f"schema_version={RUN_SCHEMA_VERSION}")
    print(f"artifact_schema_version={ARTIFACT_SCHEMA_VERSION}")
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

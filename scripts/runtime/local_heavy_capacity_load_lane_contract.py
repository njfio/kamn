#!/usr/bin/env python3
"""Local-heavy capacity/load lane runner contract."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
import time

RUN_SCHEMA_VERSION = "kamn.runtime.local-heavy-capacity-load-lane-report.v1"
ARTIFACT_SCHEMA_VERSION = "kamn.runtime.local-heavy-capacity-load-artifact-schema.v1"
REASON_TAXONOMY_VERSION = "kamn.runtime.local-heavy-capacity-load-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "local_heavy_capacity_load_profile_threshold_breach,"
    "local_heavy_capacity_load_runtime_budget_exceeded"
)

PROFILE_METRICS = {
    "baseline": {
        "throughput_tps": 11250,
        "latency_p50_ms": 92,
        "latency_p99_ms": 360,
        "error_rate_bps": 18,
    },
    "fault": {
        "throughput_tps": 8400,
        "latency_p50_ms": 168,
        "latency_p99_ms": 620,
        "error_rate_bps": 145,
    },
}

THRESHOLDS = {
    "min_throughput_tps": 10000,
    "max_latency_p50_ms": 120,
    "max_latency_p99_ms": 500,
    "max_error_rate_bps": 50,
}

OPT_IN_ENV = "KAMN_LOCAL_HEAVY_CAPACITY_LOAD_OPT_IN"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default="baseline")
    parser.add_argument("--mode", default="dry-run")
    parser.add_argument("--ci-fast-gate", default="PASS")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_HEAVY_CAPACITY_LOAD_MAX_SECONDS", "120"),
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

    if args.profile not in {"baseline", "fault"}:
        print("profile must be baseline or fault", file=sys.stderr)
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

    metrics = PROFILE_METRICS[args.profile]
    threshold_breached = (
        metrics["throughput_tps"] < THRESHOLDS["min_throughput_tps"]
        or metrics["latency_p50_ms"] > THRESHOLDS["max_latency_p50_ms"]
        or metrics["latency_p99_ms"] > THRESHOLDS["max_latency_p99_ms"]
        or metrics["error_rate_bps"] > THRESHOLDS["max_error_rate_bps"]
    )

    profile_status = "failed" if threshold_breached else "verified"
    reason_code = (
        "local_heavy_capacity_load_profile_threshold_breach"
        if threshold_breached
        else "none"
    )
    status = "fail" if threshold_breached else "pass"
    final_decision = "NO-GO" if threshold_breached else "GO"

    command_count = 0 if args.mode == "dry-run" else 1
    run_mode_command_status = (
        "dry_run_no_commands_executed"
        if args.mode == "dry-run"
        else "local_heavy_projection_executed"
    )

    elapsed_seconds = int(time.monotonic() - started)
    performance_budget_status = "verified"
    if elapsed_seconds > max_seconds:
        performance_budget_status = "violation"
        status = "fail"
        final_decision = "NO-GO"
        profile_status = "failed"
        reason_code = "local_heavy_capacity_load_runtime_budget_exceeded"

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
        "throughput_tps": metrics["throughput_tps"],
        "latency_p50_ms": metrics["latency_p50_ms"],
        "latency_p99_ms": metrics["latency_p99_ms"],
        "error_rate_bps": metrics["error_rate_bps"],
        "threshold_min_throughput_tps": THRESHOLDS["min_throughput_tps"],
        "threshold_max_latency_p50_ms": THRESHOLDS["max_latency_p50_ms"],
        "threshold_max_latency_p99_ms": THRESHOLDS["max_latency_p99_ms"],
        "threshold_max_error_rate_bps": THRESHOLDS["max_error_rate_bps"],
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
    print(f"throughput_tps={metrics['throughput_tps']}")
    print(f"latency_p50_ms={metrics['latency_p50_ms']}")
    print(f"latency_p99_ms={metrics['latency_p99_ms']}")
    print(f"error_rate_bps={metrics['error_rate_bps']}")
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

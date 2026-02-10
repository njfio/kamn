#!/usr/bin/env python3
"""Generate and validate fast-gate runtime/cost delta reports."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import date, datetime, timezone
import json
from pathlib import Path
import sys


SCHEMA_VERSION = "kamn.ci.fast-gate-budget-delta-report.v1"


@dataclass(frozen=True)
class DeltaConfig:
    baseline_elapsed_seconds: int
    baseline_runner_minutes: int
    max_elapsed_delta_pct: float
    max_runner_delta_pct: float


def fail(message: str) -> None:
    raise SystemExit(message)


def load_json(path: Path) -> dict:
    if not path.is_file():
        fail(f"file not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path}: {exc}")
    if not isinstance(payload, dict):
        fail(f"JSON object expected in {path}")
    return payload


def load_env(path: Path) -> dict[str, str]:
    if not path.is_file():
        fail(f"config file not found: {path}")
    values: dict[str, str] = {}
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            fail(f"invalid config line at {path}:{line_number}")
        key, raw_value = line.split("=", 1)
        key = key.strip()
        value = raw_value.strip()
        if not key:
            fail(f"empty config key at {path}:{line_number}")
        values[key] = value
    return values


def require_int(values: dict[str, str], key: str) -> int:
    raw = values.get(key)
    if raw is None or not raw.isdigit():
        fail(f"{key} must be a non-negative integer")
    value = int(raw)
    if value <= 0:
        fail(f"{key} must be greater than zero")
    return value


def require_float(values: dict[str, str], key: str) -> float:
    raw = values.get(key)
    if raw is None:
        fail(f"{key} is required")
    try:
        value = float(raw)
    except ValueError:
        fail(f"{key} must be a numeric value")
    if value <= 0:
        fail(f"{key} must be greater than zero")
    return value


def load_delta_config(path: Path) -> DeltaConfig:
    values = load_env(path)
    return DeltaConfig(
        baseline_elapsed_seconds=require_int(values, "FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS"),
        baseline_runner_minutes=require_int(values, "FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES"),
        max_elapsed_delta_pct=require_float(values, "FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT"),
        max_runner_delta_pct=require_float(values, "FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT"),
    )


def require_payload_int(payload: dict, key: str) -> int:
    value = payload.get(key)
    if not isinstance(value, int):
        fail(f"{key} must be an integer")
    if value < 0:
        fail(f"{key} must be a non-negative integer")
    return value


def compute_delta(current: int, baseline: int) -> tuple[int, float]:
    delta = current - baseline
    delta_pct = round((delta / baseline) * 100.0, 2)
    return delta, delta_pct


def command_generate(args: argparse.Namespace) -> int:
    current_path = Path(args.current_json)
    output_path = Path(args.output_json)
    baseline_path = Path(args.baseline_file)
    lane = args.lane

    current_payload = load_json(current_path)
    current_lane = current_payload.get("lane")
    if current_lane != lane:
        fail(f"current budget lane mismatch: expected {lane}, found {current_lane}")

    current_elapsed = require_payload_int(current_payload, "elapsed_seconds")
    current_runner_minutes = require_payload_int(current_payload, "runner_minutes")

    config = load_delta_config(baseline_path)
    elapsed_delta, elapsed_delta_pct = compute_delta(
        current=current_elapsed,
        baseline=config.baseline_elapsed_seconds,
    )
    runner_delta, runner_delta_pct = compute_delta(
        current=current_runner_minutes,
        baseline=config.baseline_runner_minutes,
    )

    report_payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "lane": lane,
        "baseline_source": str(baseline_path),
        "current_budget_source": str(current_path),
        "current_budget_status": current_payload.get("status", "unknown"),
        "baseline": {
            "elapsed_seconds": config.baseline_elapsed_seconds,
            "runner_minutes": config.baseline_runner_minutes,
        },
        "current": {
            "elapsed_seconds": current_elapsed,
            "runner_minutes": current_runner_minutes,
        },
        "variance": {
            "elapsed_seconds_delta": elapsed_delta,
            "elapsed_seconds_delta_pct": elapsed_delta_pct,
            "runner_minutes_delta": runner_delta,
            "runner_minutes_delta_pct": runner_delta_pct,
        },
        "thresholds": {
            "max_elapsed_delta_pct": config.max_elapsed_delta_pct,
            "max_runner_minutes_delta_pct": config.max_runner_delta_pct,
        },
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print("status=generated")
    print(f"lane={lane}")
    print(f"elapsed_seconds_delta={elapsed_delta}")
    print(f"elapsed_seconds_delta_pct={elapsed_delta_pct}")
    print(f"runner_minutes_delta={runner_delta}")
    print(f"runner_minutes_delta_pct={runner_delta_pct}")
    return 0


def parse_waiver(waiver_path: Path, violations: list[str]) -> tuple[bool, str]:
    if not waiver_path.is_file():
        return False, "waiver file not found"

    payload = load_json(waiver_path)
    reason = payload.get("reason")
    expires_on = payload.get("expires_on")
    allow_metrics = payload.get("allow_metrics")

    if not isinstance(reason, str) or not reason.strip():
        fail("waiver reason must be a non-empty string")
    if not isinstance(expires_on, str) or not expires_on:
        fail("waiver expires_on must be a non-empty YYYY-MM-DD value")
    if not isinstance(allow_metrics, list) or not all(isinstance(metric, str) for metric in allow_metrics):
        fail("waiver allow_metrics must be a string list")
    if not allow_metrics:
        fail("waiver allow_metrics must not be empty")

    try:
        expires_date = date.fromisoformat(expires_on)
    except ValueError:
        fail("waiver expires_on must be in YYYY-MM-DD format")

    if expires_date < date.today():
        fail(f"waiver expired on {expires_on}")

    missing_metrics = sorted(set(violations).difference(allow_metrics))
    if missing_metrics:
        return False, f"waiver does not allow metrics: {','.join(missing_metrics)}"

    return True, reason.strip()


def extract_section(payload: dict, key: str) -> dict:
    section = payload.get(key)
    if not isinstance(section, dict):
        fail(f"{key} must be an object")
    return section


def extract_float(section: dict, key: str) -> float:
    value = section.get(key)
    if not isinstance(value, (int, float)):
        fail(f"{key} must be numeric")
    return float(value)


def command_check(args: argparse.Namespace) -> int:
    report_path = Path(args.report_json)
    threshold_path = Path(args.threshold_file)
    waiver_path = Path(args.waiver_file)

    report = load_json(report_path)
    if report.get("schema_version") != SCHEMA_VERSION:
        fail("unexpected schema_version in fast-gate delta report")
    if report.get("lane") != "fast-gate":
        fail("delta report lane must be fast-gate")

    variance = extract_section(report, "variance")
    elapsed_delta = extract_float(variance, "elapsed_seconds_delta")
    elapsed_delta_pct = extract_float(variance, "elapsed_seconds_delta_pct")
    runner_delta = extract_float(variance, "runner_minutes_delta")
    runner_delta_pct = extract_float(variance, "runner_minutes_delta_pct")

    config = load_delta_config(threshold_path)
    violations: list[str] = []
    if elapsed_delta > 0 and elapsed_delta_pct > config.max_elapsed_delta_pct:
        violations.append("elapsed_seconds_delta_pct")
    if runner_delta > 0 and runner_delta_pct > config.max_runner_delta_pct:
        violations.append("runner_minutes_delta_pct")

    if not violations:
        print("status=pass")
        print("waived=false")
        print("violations=none")
        return 0

    waiver_applied, waiver_reason = parse_waiver(waiver_path, violations)
    if waiver_applied:
        print("status=pass")
        print("waived=true")
        print(f"violations={','.join(violations)}")
        print(f"waiver_reason={waiver_reason}")
        return 0

    print("status=fail")
    print("waived=false")
    print(f"violations={','.join(violations)}")
    print(f"failure_reason={waiver_reason}")
    return 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Fast-gate budget delta tooling.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate", help="Generate a fast-gate delta report.")
    generate.add_argument("--current-json", required=True)
    generate.add_argument("--output-json", required=True)
    generate.add_argument("--baseline-file", default=".ci/fast-gate-budget-delta.env")
    generate.add_argument("--lane", default="fast-gate")
    generate.set_defaults(handler=command_generate)

    check = subparsers.add_parser("check", help="Validate a fast-gate delta report against thresholds.")
    check.add_argument("--report-json", required=True)
    check.add_argument("--threshold-file", default=".ci/fast-gate-budget-delta.env")
    check.add_argument("--waiver-file", default=".ci/fast-gate-budget-delta-waiver.json")
    check.set_defaults(handler=command_check)

    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except SystemExit as exc:
        if isinstance(exc.code, str):
            print(exc.code, file=sys.stderr)
            raise SystemExit(1)
        raise

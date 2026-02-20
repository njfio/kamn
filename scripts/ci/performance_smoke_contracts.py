#!/usr/bin/env python3
"""Shared performance smoke report generation/check contracts."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_FIXTURE_FILE = REPO_ROOT / "fixtures/ci/performance_hot_path_fixture_matrix.json"
DEFAULT_PROFILE_FILE = REPO_ROOT / ".ci/performance-targets.env"
FIXTURE_SCHEMA_VERSION = "kamn.ci.performance-hot-path-matrix.v1"


class ContractError(RuntimeError):
    """Raised when contract inputs are invalid."""


def fail(message: str) -> None:
    raise ContractError(message)


def is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def format_number(value: float) -> str:
    if float(value).is_integer():
        return str(int(value))
    return f"{value:g}"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Performance smoke generation/check contract helper."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser(
        "generate",
        help=(
            "Generate report from fixture matrix. "
            "Usage: generate --output-json <path> [--lane <smoke|deep>] "
            "[--workload <runtime|signing|transport>] [--fixture-file <path>]"
        ),
    )
    generate.add_argument("--output-json", required=True)
    generate.add_argument("--lane", default="smoke")
    generate.add_argument("--workload", default="runtime")
    generate.add_argument("--fixture-file", default=str(DEFAULT_FIXTURE_FILE))

    check = subparsers.add_parser(
        "check",
        help=(
            "Check report against profile thresholds. "
            "Usage: check --report-json <path> [--profile-file <path>] "
            "[--lane <smoke|deep>]"
        ),
    )
    check.add_argument("--report-json", required=True)
    check.add_argument("--profile-file", default=str(DEFAULT_PROFILE_FILE))
    check.add_argument("--lane", default="smoke")

    return parser.parse_args(argv)


def load_json(path: Path, *, parse_error_prefix: str) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover - exact message parity path
        fail(f"{parse_error_prefix}: {exc}")


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def generate_report(args: argparse.Namespace) -> int:
    output_json = Path(args.output_json)
    fixture_path = Path(args.fixture_file)
    lane = args.lane
    workload = args.workload

    if not fixture_path.is_file():
        fail(f"fixture file not found: {fixture_path}")

    payload = load_json(fixture_path, parse_error_prefix="failed to parse fixture matrix")

    schema_version = payload.get("schema_version")
    if schema_version != FIXTURE_SCHEMA_VERSION:
        fail(
            "fixture schema version mismatch: "
            f"expected {FIXTURE_SCHEMA_VERSION}, got {schema_version}"
        )

    provenance = payload.get("baseline_provenance")
    if not isinstance(provenance, dict):
        fail("fixture matrix must include baseline_provenance object")

    provenance_required_fields = (
        "artifact_version",
        "source_commit",
        "source_run_id",
        "generated_at_utc",
        "generator",
    )
    for field in provenance_required_fields:
        value = provenance.get(field)
        if not isinstance(value, str) or not value.strip():
            fail(f"baseline_provenance.{field} must be non-empty string")

    fixtures = payload.get("fixtures")
    if not isinstance(fixtures, list) or not fixtures:
        fail("fixture matrix must include non-empty fixtures array")

    known_workloads = sorted(
        {
            item.get("workload")
            for item in fixtures
            if isinstance(item, dict) and isinstance(item.get("workload"), str)
        }
    )
    if workload not in known_workloads:
        fail(f"Unknown workload: {workload}")

    matching: dict[str, Any] | None = None
    for item in fixtures:
        if not isinstance(item, dict):
            continue
        if item.get("workload") == workload and item.get("lane") == lane:
            matching = item
            break
    if matching is None:
        fail(f"Unsupported lane for workload {workload}: {lane}")

    required_fields = (
        "latency_p50_ms",
        "latency_p99_ms",
        "throughput_tps",
        "availability_pct",
    )
    for field in required_fields:
        value = matching.get(field)
        if not is_number(value):
            fail(f"fixture field {field} must be numeric")

    if matching["latency_p50_ms"] < 0 or matching["latency_p99_ms"] < 0:
        fail("latency fields must be non-negative")
    if matching["throughput_tps"] <= 0:
        fail("throughput_tps must be > 0")
    if matching["availability_pct"] <= 0 or matching["availability_pct"] > 100:
        fail("availability_pct must be within (0, 100]")

    drift_seed_id = matching.get("drift_threshold_seed_id")
    if not isinstance(drift_seed_id, str) or not drift_seed_id.strip():
        fail("fixture field drift_threshold_seed_id must be non-empty string")

    drift_seed = matching.get("drift_threshold_seed")
    if not isinstance(drift_seed, dict):
        fail("fixture field drift_threshold_seed must be an object")

    drift_seed_required_fields = (
        "max_latency_p50_ms",
        "max_latency_p99_ms",
        "min_throughput_tps",
        "min_availability_pct",
    )
    for field in drift_seed_required_fields:
        value = drift_seed.get(field)
        if not is_number(value):
            fail(f"drift_threshold_seed.{field} must be numeric")

    if drift_seed["max_latency_p50_ms"] <= 0 or drift_seed["max_latency_p99_ms"] <= 0:
        fail("drift threshold max latency values must be > 0")
    if drift_seed["min_throughput_tps"] <= 0:
        fail("drift threshold min throughput must be > 0")
    if drift_seed["min_availability_pct"] <= 0 or drift_seed["min_availability_pct"] > 100:
        fail("drift threshold min availability must be within (0, 100]")

    profile = matching.get("profile")
    if not isinstance(profile, str) or not profile.strip():
        profile = f"prd-13.2-ci-{workload}-{lane}"

    report = {
        "profile": profile,
        "lane": lane,
        "workload": workload,
        "latency_p50_ms": matching["latency_p50_ms"],
        "latency_p99_ms": matching["latency_p99_ms"],
        "throughput_tps": matching["throughput_tps"],
        "availability_pct": matching["availability_pct"],
        "baseline_provenance_artifact_version": provenance["artifact_version"],
        "baseline_provenance_source_commit": provenance["source_commit"],
        "baseline_provenance_source_run_id": provenance["source_run_id"],
        "baseline_provenance_generated_at_utc": provenance["generated_at_utc"],
        "baseline_provenance_generator": provenance["generator"],
        "drift_threshold_seed_id": drift_seed_id,
        "drift_threshold_seed_max_latency_p50_ms": drift_seed["max_latency_p50_ms"],
        "drift_threshold_seed_max_latency_p99_ms": drift_seed["max_latency_p99_ms"],
        "drift_threshold_seed_min_throughput_tps": drift_seed["min_throughput_tps"],
        "drift_threshold_seed_min_availability_pct": drift_seed["min_availability_pct"],
    }
    write_json(output_json, report)

    print(
        "generated performance report: "
        f"workload={workload}; lane={lane}; output={output_json}"
    )
    return 0


def parse_env_file(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def extract_metric(report: dict[str, Any], key: str) -> float:
    value = report.get(key)
    if not is_number(value):
        fail(f"missing required metric: {key}")
    return float(value)


def extract_string_marker(report: dict[str, Any], key: str) -> str:
    value = report.get(key)
    if not isinstance(value, str) or not value:
        fail(f"missing required baseline marker: {key}")
    return value


def check_report(args: argparse.Namespace) -> int:
    report_path = Path(args.report_json)
    profile_path = Path(args.profile_file)
    lane = args.lane

    if not report_path.is_file():
        fail(f"report file not found: {report_path}")
    if not profile_path.is_file():
        fail(f"profile file not found: {profile_path}")

    profile_values = parse_env_file(profile_path)
    if lane == "smoke":
        threshold_keys = {
            "max_p50": "PERF_SMOKE_MAX_LATENCY_P50_MS",
            "max_p99": "PERF_SMOKE_MAX_LATENCY_P99_MS",
            "min_throughput": "PERF_SMOKE_MIN_THROUGHPUT_TPS",
            "min_availability": "PERF_SMOKE_MIN_AVAILABILITY_PCT",
        }
    elif lane == "deep":
        threshold_keys = {
            "max_p50": "PERF_DEEP_MAX_LATENCY_P50_MS",
            "max_p99": "PERF_DEEP_MAX_LATENCY_P99_MS",
            "min_throughput": "PERF_DEEP_MIN_THROUGHPUT_TPS",
            "min_availability": "PERF_DEEP_MIN_AVAILABILITY_PCT",
        }
    else:
        fail(f"Unsupported lane: {lane}")

    raw_thresholds: dict[str, str] = {}
    thresholds: dict[str, float] = {}
    for alias, key in threshold_keys.items():
        raw_value = profile_values.get(key)
        if raw_value is None:
            fail(f"missing profile threshold: {key}")
        try:
            thresholds[alias] = float(raw_value)
        except ValueError:
            fail(f"invalid numeric threshold: {key}")
        raw_thresholds[alias] = raw_value

    report = load_json(report_path, parse_error_prefix="failed to parse report JSON")
    if not isinstance(report, dict):
        fail("report JSON must be an object")

    latency_p50 = extract_metric(report, "latency_p50_ms")
    latency_p99 = extract_metric(report, "latency_p99_ms")
    throughput = extract_metric(report, "throughput_tps")
    availability = extract_metric(report, "availability_pct")
    baseline_version = extract_string_marker(report, "baseline_provenance_artifact_version")
    baseline_commit = extract_string_marker(report, "baseline_provenance_source_commit")
    baseline_run_id = extract_string_marker(report, "baseline_provenance_source_run_id")
    baseline_generated = extract_string_marker(report, "baseline_provenance_generated_at_utc")
    baseline_generator = extract_string_marker(report, "baseline_provenance_generator")
    drift_seed_id = extract_string_marker(report, "drift_threshold_seed_id")
    drift_seed_max_p50 = extract_metric(report, "drift_threshold_seed_max_latency_p50_ms")
    drift_seed_max_p99 = extract_metric(report, "drift_threshold_seed_max_latency_p99_ms")
    drift_seed_min_throughput = extract_metric(report, "drift_threshold_seed_min_throughput_tps")
    drift_seed_min_availability = extract_metric(
        report,
        "drift_threshold_seed_min_availability_pct",
    )

    failures: list[str] = []
    if not (latency_p50 < thresholds["max_p50"]):
        failures.append(f"latency_p50_ms>={raw_thresholds['max_p50']}")
    if not (latency_p99 < thresholds["max_p99"]):
        failures.append(f"latency_p99_ms>={raw_thresholds['max_p99']}")
    if not (throughput >= thresholds["min_throughput"]):
        failures.append(f"throughput_tps<{raw_thresholds['min_throughput']}")
    if not (availability >= thresholds["min_availability"]):
        failures.append(f"availability_pct<{raw_thresholds['min_availability']}")
    if not (0 < drift_seed_max_p50):
        failures.append("drift_threshold_seed_max_latency_p50_ms<=0")
    if not (0 < drift_seed_max_p99):
        failures.append("drift_threshold_seed_max_latency_p99_ms<=0")
    if not (0 < drift_seed_min_throughput):
        failures.append("drift_threshold_seed_min_throughput_tps<=0")
    if not (drift_seed_min_availability >= 0):
        failures.append("drift_threshold_seed_min_availability_pct<0")
    if not (drift_seed_min_availability < 101):
        failures.append("drift_threshold_seed_min_availability_pct>100")

    if failures:
        print(f"status=fail; lane={lane}; failures={','.join(failures)}")
        return 1

    print(
        f"status=pass; lane={lane}; "
        f"latency_p50_ms={format_number(latency_p50)}; "
        f"latency_p99_ms={format_number(latency_p99)}; "
        f"throughput_tps={format_number(throughput)}; "
        f"availability_pct={format_number(availability)}; "
        f"baseline_version={baseline_version}; "
        f"baseline_commit={baseline_commit}; "
        f"baseline_run_id={baseline_run_id}; "
        f"baseline_generated_at_utc={baseline_generated}; "
        f"baseline_generator={baseline_generator}; "
        f"drift_threshold_seed_id={drift_seed_id}"
    )
    return 0


def main(argv: list[str]) -> int:
    try:
        args = parse_args(argv)
        if args.command == "generate":
            return generate_report(args)
        if args.command == "check":
            return check_report(args)
        fail(f"Unknown command: {args.command}")
    except ContractError as exc:
        print(str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

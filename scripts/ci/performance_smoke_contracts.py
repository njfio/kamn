#!/usr/bin/env python3
"""Shared performance smoke report generation/check contracts."""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_FIXTURE_FILE = REPO_ROOT / "fixtures/ci/performance_hot_path_fixture_matrix.json"
DEFAULT_PROFILE_FILE = REPO_ROOT / ".ci/performance-targets.env"
DEFAULT_CI_TOOLS_FILE = REPO_ROOT / "scripts/ci/test_ci_tools.sh"
DEFAULT_WORKFLOW_FILE = REPO_ROOT / ".github/workflows/ci-fast-gate.yml"
DEFAULT_STRATEGY_DOC = REPO_ROOT / "docs/ci/strategy.md"
FIXTURE_SCHEMA_VERSION = "kamn.ci.performance-hot-path-matrix.v1"
REPORT_SCHEMA_VERSION = "kamn.ci.performance-ci-smoke-governance-report.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.performance-ci-smoke-threshold-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "performance_ci_smoke_argument_invalid,"
    "performance_ci_smoke_threshold_contract_violation,"
    "performance_ci_smoke_report_contract_violation,"
    "performance_ci_smoke_latency_p50_threshold_exceeded,"
    "performance_ci_smoke_latency_p99_threshold_exceeded,"
    "performance_ci_smoke_throughput_threshold_below_minimum,"
    "performance_ci_smoke_availability_threshold_below_minimum,"
    "performance_ci_smoke_selector_missing_checker_entry,"
    "performance_ci_smoke_selector_forbidden_entry_present,"
    "performance_ci_smoke_workflow_missing_checker_step,"
    "performance_ci_smoke_workflow_forbidden_entry_present,"
    "performance_ci_smoke_docs_marker_parity_drift,"
    "performance_ci_smoke_docs_remediation_marker_missing,"
    "performance_ci_smoke_runtime_budget_exceeded"
)
REASON_CODES_ORDER = tuple(REASON_CODES_CSV.split(","))
DEFAULT_MAX_SECONDS = 120
CI_TOOLS_REQUIRED_ENTRY = (
    'cargo test -p kamn-core --test performance_ci_smoke_governance_contract -- --nocapture'
)
CI_TOOLS_FORBIDDEN_ENTRY = (
    'bash "$ROOT_DIR/scripts/ci/check_performance_thresholds.sh" --lane deep'
)
WORKFLOW_REQUIRED_ENTRY = (
    "bash scripts/ci/check_performance_thresholds.sh --lane smoke --report-json "
    "performance-smoke-report.json --profile-file .ci/performance-targets.env"
)
WORKFLOW_FORBIDDEN_ENTRY = "bash scripts/ci/check_performance_thresholds.sh --lane deep"


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
            "Check report against profile thresholds and selector/workflow contracts. "
            "Usage: check --report-json <path> [--profile-file <path>] "
            "[--lane <smoke|deep>] [--ci-tools-file <path>] [--workflow-file <path>] "
            "[--strategy-doc <path>] [--max-seconds <int>]"
        ),
    )
    check.add_argument("--report-json", required=True)
    check.add_argument("--profile-file", default=str(DEFAULT_PROFILE_FILE))
    check.add_argument("--lane", default="smoke")
    check.add_argument("--ci-tools-file", default=str(DEFAULT_CI_TOOLS_FILE))
    check.add_argument("--workflow-file", default=str(DEFAULT_WORKFLOW_FILE))
    check.add_argument("--strategy-doc", default=str(DEFAULT_STRATEGY_DOC))
    check.add_argument("--max-seconds", default=str(DEFAULT_MAX_SECONDS))

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


def try_parse_positive_int(raw_value: str) -> int | None:
    try:
        parsed = int(raw_value)
    except ValueError:
        return None
    if parsed <= 0:
        return None
    return parsed


def extract_fast_mode_block(text: str) -> str:
    match = re.search(
        r'if \[ "\$\{KAMN_CI_TOOLS_FAST_MODE:-false\}" = "true" \]; then(?P<body>.*?)\n\s*echo "Fast-mode CI tool regression tests passed\."\n\s*exit 0\nfi',
        text,
        flags=re.DOTALL,
    )
    return "" if not match else match.group("body")


def normalize_reason_codes(reason_codes: list[str]) -> list[str]:
    observed = set(reason_codes)
    return [code for code in REASON_CODES_ORDER if code in observed]


def reason_codes_value(reason_codes: list[str]) -> str:
    return "none" if not reason_codes else ",".join(reason_codes)


def add_reason(reason_codes: list[str], reason_code: str) -> None:
    if reason_code not in reason_codes:
        reason_codes.append(reason_code)


def is_valid_report_metric(report: dict[str, Any], key: str) -> bool:
    value = report.get(key)
    return is_number(value)


def has_non_empty_string_marker(report: dict[str, Any], key: str) -> bool:
    value = report.get(key)
    return isinstance(value, str) and bool(value.strip())


def doc_required_markers() -> tuple[list[str], tuple[str, ...]]:
    markers = [
        "## Performance CI Smoke Threshold Governance Contract",
        (
            "bash scripts/ci/check_performance_thresholds.sh --lane smoke --report-json "
            "/tmp/performance-smoke-report.json --profile-file .ci/performance-targets.env "
            "--ci-tools-file scripts/ci/test_ci_tools.sh --workflow-file "
            ".github/workflows/ci-fast-gate.yml --strategy-doc docs/ci/strategy.md "
            "--max-seconds 120"
        ),
        (
            "cargo test -p kamn-core --test performance_ci_smoke_governance_contract "
            "-- --nocapture"
        ),
        f"performance_ci_smoke_reason_taxonomy_version={REASON_TAXONOMY_VERSION}",
        f"performance_ci_smoke_reason_codes_csv={REASON_CODES_CSV}",
        "performance_ci_smoke_reason_codes_value=none|<csv>",
        "performance_ci_smoke_max_seconds=120",
        "performance_ci_smoke_docs_status=verified|violation",
        "performance_ci_smoke_docs_remediation_status=verified|violation",
        "performance_ci_smoke_remediation_map_version=v1",
        f"performance_ci_smoke_fast_mode_required_entry={CI_TOOLS_REQUIRED_ENTRY}",
        f"performance_ci_smoke_fast_mode_forbidden_entry={CI_TOOLS_FORBIDDEN_ENTRY}",
        f"performance_ci_smoke_workflow_required_entry={WORKFLOW_REQUIRED_ENTRY}",
        f"performance_ci_smoke_workflow_forbidden_entry={WORKFLOW_FORBIDDEN_ENTRY}",
        "Regression: #4002, #4003",
    ]
    return markers, REASON_CODES_ORDER


def check_report(args: argparse.Namespace) -> int:
    started = time.monotonic()
    report_path = Path(args.report_json)
    profile_path = Path(args.profile_file)
    ci_tools_path = Path(args.ci_tools_file)
    workflow_path = Path(args.workflow_file)
    strategy_doc_path = Path(args.strategy_doc)
    lane = args.lane

    threshold_status = "verified"
    report_status = "verified"
    selector_status = "verified"
    workflow_status = "verified"
    docs_status = "verified"
    docs_remediation_status = "verified"
    raw_reason_codes: list[str] = []

    max_seconds = try_parse_positive_int(args.max_seconds)
    if max_seconds is None:
        max_seconds = DEFAULT_MAX_SECONDS
        add_reason(raw_reason_codes, "performance_ci_smoke_argument_invalid")

    if not report_path.is_file():
        add_reason(raw_reason_codes, "performance_ci_smoke_argument_invalid")
        report_status = "violation"
    if not profile_path.is_file():
        add_reason(raw_reason_codes, "performance_ci_smoke_argument_invalid")
        threshold_status = "violation"
    if not ci_tools_path.is_file():
        add_reason(raw_reason_codes, "performance_ci_smoke_argument_invalid")
        selector_status = "violation"
    if not workflow_path.is_file():
        add_reason(raw_reason_codes, "performance_ci_smoke_argument_invalid")
        workflow_status = "violation"

    profile_values: dict[str, str] = {}
    if profile_path.is_file():
        try:
            profile_values = parse_env_file(profile_path)
        except Exception:
            add_reason(raw_reason_codes, "performance_ci_smoke_threshold_contract_violation")
            threshold_status = "violation"

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
        threshold_keys = {}
        add_reason(raw_reason_codes, "performance_ci_smoke_threshold_contract_violation")
        threshold_status = "violation"

    thresholds: dict[str, float] = {}
    for alias, key in threshold_keys.items():
        raw_value = profile_values.get(key)
        if raw_value is None:
            add_reason(raw_reason_codes, "performance_ci_smoke_threshold_contract_violation")
            threshold_status = "violation"
            continue
        try:
            thresholds[alias] = float(raw_value)
        except ValueError:
            add_reason(raw_reason_codes, "performance_ci_smoke_threshold_contract_violation")
            threshold_status = "violation"

    report: dict[str, Any] | None = None
    if report_path.is_file():
        try:
            payload = json.loads(report_path.read_text(encoding="utf-8"))
            if not isinstance(payload, dict):
                raise ValueError("report JSON must be an object")
            report = payload
        except Exception:
            add_reason(raw_reason_codes, "performance_ci_smoke_report_contract_violation")
            report_status = "violation"

    latency_p50 = 0.0
    latency_p99 = 0.0
    throughput = 0.0
    availability = 0.0
    baseline_version = ""
    baseline_commit = ""
    baseline_run_id = ""
    baseline_generated = ""
    baseline_generator = ""
    drift_seed_id = ""
    metrics_ready = False

    if report is not None:
        required_metric_fields = (
            "latency_p50_ms",
            "latency_p99_ms",
            "throughput_tps",
            "availability_pct",
            "drift_threshold_seed_max_latency_p50_ms",
            "drift_threshold_seed_max_latency_p99_ms",
            "drift_threshold_seed_min_throughput_tps",
            "drift_threshold_seed_min_availability_pct",
        )
        required_string_fields = (
            "baseline_provenance_artifact_version",
            "baseline_provenance_source_commit",
            "baseline_provenance_source_run_id",
            "baseline_provenance_generated_at_utc",
            "baseline_provenance_generator",
            "drift_threshold_seed_id",
        )

        metrics_ok = all(is_valid_report_metric(report, field) for field in required_metric_fields)
        strings_ok = all(
            has_non_empty_string_marker(report, field) for field in required_string_fields
        )
        if not metrics_ok or not strings_ok:
            add_reason(raw_reason_codes, "performance_ci_smoke_report_contract_violation")
            report_status = "violation"
        else:
            latency_p50 = float(report["latency_p50_ms"])
            latency_p99 = float(report["latency_p99_ms"])
            throughput = float(report["throughput_tps"])
            availability = float(report["availability_pct"])
            baseline_version = str(report["baseline_provenance_artifact_version"])
            baseline_commit = str(report["baseline_provenance_source_commit"])
            baseline_run_id = str(report["baseline_provenance_source_run_id"])
            baseline_generated = str(report["baseline_provenance_generated_at_utc"])
            baseline_generator = str(report["baseline_provenance_generator"])
            drift_seed_id = str(report["drift_threshold_seed_id"])

            drift_seed_max_p50 = float(report["drift_threshold_seed_max_latency_p50_ms"])
            drift_seed_max_p99 = float(report["drift_threshold_seed_max_latency_p99_ms"])
            drift_seed_min_throughput = float(report["drift_threshold_seed_min_throughput_tps"])
            drift_seed_min_availability = float(report["drift_threshold_seed_min_availability_pct"])

            if drift_seed_max_p50 <= 0 or drift_seed_max_p99 <= 0:
                add_reason(raw_reason_codes, "performance_ci_smoke_report_contract_violation")
                report_status = "violation"
            if drift_seed_min_throughput <= 0:
                add_reason(raw_reason_codes, "performance_ci_smoke_report_contract_violation")
                report_status = "violation"
            if drift_seed_min_availability < 0 or drift_seed_min_availability > 100:
                add_reason(raw_reason_codes, "performance_ci_smoke_report_contract_violation")
                report_status = "violation"

            metrics_ready = True

    if metrics_ready and {"max_p50", "max_p99", "min_throughput", "min_availability"} <= set(
        thresholds
    ):
        if not (latency_p50 < thresholds["max_p50"]):
            add_reason(raw_reason_codes, "performance_ci_smoke_latency_p50_threshold_exceeded")
        if not (latency_p99 < thresholds["max_p99"]):
            add_reason(raw_reason_codes, "performance_ci_smoke_latency_p99_threshold_exceeded")
        if not (throughput >= thresholds["min_throughput"]):
            add_reason(raw_reason_codes, "performance_ci_smoke_throughput_threshold_below_minimum")
        if not (availability >= thresholds["min_availability"]):
            add_reason(
                raw_reason_codes,
                "performance_ci_smoke_availability_threshold_below_minimum",
            )

    if ci_tools_path.is_file():
        try:
            ci_tools_text = ci_tools_path.read_text(encoding="utf-8")
            fast_mode_block = extract_fast_mode_block(ci_tools_text)
            if not fast_mode_block or CI_TOOLS_REQUIRED_ENTRY not in fast_mode_block:
                add_reason(raw_reason_codes, "performance_ci_smoke_selector_missing_checker_entry")
                selector_status = "violation"
            if CI_TOOLS_FORBIDDEN_ENTRY in fast_mode_block:
                add_reason(raw_reason_codes, "performance_ci_smoke_selector_forbidden_entry_present")
                selector_status = "violation"
        except Exception:
            add_reason(raw_reason_codes, "performance_ci_smoke_selector_missing_checker_entry")
            selector_status = "violation"

    if workflow_path.is_file():
        try:
            workflow_text = workflow_path.read_text(encoding="utf-8")
            if WORKFLOW_REQUIRED_ENTRY not in workflow_text:
                add_reason(raw_reason_codes, "performance_ci_smoke_workflow_missing_checker_step")
                workflow_status = "violation"
            if WORKFLOW_FORBIDDEN_ENTRY in workflow_text:
                add_reason(raw_reason_codes, "performance_ci_smoke_workflow_forbidden_entry_present")
                workflow_status = "violation"
        except Exception:
            add_reason(raw_reason_codes, "performance_ci_smoke_workflow_missing_checker_step")
            workflow_status = "violation"

    if strategy_doc_path.is_file():
        try:
            strategy_text = strategy_doc_path.read_text(encoding="utf-8")
            required_markers, reason_codes = doc_required_markers()
            if any(marker not in strategy_text for marker in required_markers):
                add_reason(raw_reason_codes, "performance_ci_smoke_docs_marker_parity_drift")
                docs_status = "violation"
            for reason_code in reason_codes:
                if f"performance_ci_smoke_remediation.{reason_code}=" not in strategy_text:
                    add_reason(
                        raw_reason_codes,
                        "performance_ci_smoke_docs_remediation_marker_missing",
                    )
                    docs_remediation_status = "violation"
                    break
        except Exception:
            add_reason(raw_reason_codes, "performance_ci_smoke_docs_marker_parity_drift")
            docs_status = "violation"
            docs_remediation_status = "violation"
    else:
        add_reason(raw_reason_codes, "performance_ci_smoke_docs_marker_parity_drift")
        docs_status = "violation"
        docs_remediation_status = "violation"

    elapsed_seconds = int(time.monotonic() - started)
    if elapsed_seconds > max_seconds:
        add_reason(raw_reason_codes, "performance_ci_smoke_runtime_budget_exceeded")

    normalized_reasons = normalize_reason_codes(raw_reason_codes)
    reason_value = reason_codes_value(normalized_reasons)
    status = "pass" if not normalized_reasons else "fail"
    final_decision = "GO" if status == "pass" else "NO-GO"
    contract_status = "verified" if status == "pass" else "violation"

    payload = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "lane": lane,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes": normalized_reasons,
        "reason_codes_value": reason_value,
        "performance_ci_smoke_contract_status": contract_status,
        "performance_ci_smoke_threshold_status": threshold_status,
        "performance_ci_smoke_report_status": report_status,
        "performance_ci_smoke_selector_status": selector_status,
        "performance_ci_smoke_workflow_status": workflow_status,
        "performance_ci_smoke_docs_status": docs_status,
        "performance_ci_smoke_docs_remediation_status": docs_remediation_status,
        "performance_ci_smoke_max_seconds": max_seconds,
        "performance_ci_smoke_elapsed_seconds": elapsed_seconds,
        "inputs": {
            "report_json": str(report_path),
            "profile_file": str(profile_path),
            "ci_tools_file": str(ci_tools_path),
            "workflow_file": str(workflow_path),
            "strategy_doc": str(strategy_doc_path),
        },
    }

    if metrics_ready:
        payload.update(
            {
                "latency_p50_ms": latency_p50,
                "latency_p99_ms": latency_p99,
                "throughput_tps": throughput,
                "availability_pct": availability,
                "baseline_version": baseline_version,
                "baseline_commit": baseline_commit,
                "baseline_run_id": baseline_run_id,
                "baseline_generated_at_utc": baseline_generated,
                "baseline_generator": baseline_generator,
                "drift_threshold_seed_id": drift_seed_id,
            }
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"lane={lane}")
    print(f"performance_ci_smoke_reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"performance_ci_smoke_reason_codes_csv={REASON_CODES_CSV}")
    print(f"performance_ci_smoke_reason_codes_value={reason_value}")
    print(f"performance_ci_smoke_contract_status={contract_status}")
    print(f"performance_ci_smoke_threshold_status={threshold_status}")
    print(f"performance_ci_smoke_report_status={report_status}")
    print(f"performance_ci_smoke_selector_status={selector_status}")
    print(f"performance_ci_smoke_workflow_status={workflow_status}")
    print(f"performance_ci_smoke_docs_status={docs_status}")
    print(f"performance_ci_smoke_docs_remediation_status={docs_remediation_status}")
    print(f"performance_ci_smoke_max_seconds={max_seconds}")
    print(f"performance_ci_smoke_elapsed_seconds={elapsed_seconds}")

    if status == "pass" and metrics_ready:
        print(f"latency_p50_ms={format_number(latency_p50)}")
        print(f"latency_p99_ms={format_number(latency_p99)}")
        print(f"throughput_tps={format_number(throughput)}")
        print(f"availability_pct={format_number(availability)}")
        print(f"baseline_version={baseline_version}")
        print(f"baseline_commit={baseline_commit}")
        print(f"baseline_run_id={baseline_run_id}")
        print(f"baseline_generated_at_utc={baseline_generated}")
        print(f"baseline_generator={baseline_generator}")
        print(f"drift_threshold_seed_id={drift_seed_id}")

    return 0 if status == "pass" else 1


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

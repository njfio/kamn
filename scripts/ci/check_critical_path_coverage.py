#!/usr/bin/env python3
"""Fail-closed coverage policy for critical runtime/security paths."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "kamn.ci.critical-path-coverage-policy-report.v1"
THRESHOLD_SCHEMA_VERSION = "kamn.ci.critical-path-coverage-thresholds.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.critical-path-coverage-reason-taxonomy.v1"
ORDERED_REASON_CODES = (
    "critical_path_coverage_threshold_file_missing",
    "critical_path_coverage_threshold_file_invalid",
    "critical_path_coverage_threshold_schema_invalid",
    "critical_path_coverage_target_threshold_invalid",
    "critical_path_coverage_report_missing",
    "critical_path_coverage_report_invalid",
    "critical_path_coverage_report_schema_invalid",
    "critical_path_coverage_target_missing",
    "critical_path_coverage_line_below_threshold",
    "critical_path_coverage_function_below_threshold",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--core-coverage-json", default="ci-critical-path-core-coverage.json")
    parser.add_argument("--node-coverage-json", default="ci-critical-path-node-coverage.json")
    parser.add_argument(
        "--threshold-file",
        default=".ci/critical-path-coverage-thresholds.json",
    )
    parser.add_argument(
        "--output-json",
        default="ci-critical-path-coverage-policy.json",
    )
    return parser.parse_args()


def load_json(path: Path, missing_code: str, invalid_code: str, reason_codes: set[str], violations: list[str]) -> Any:
    if not path.exists():
        reason_codes.add(missing_code)
        violations.append(f"missing file: {path}")
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        reason_codes.add(invalid_code)
        violations.append(f"invalid JSON {path}: {error}")
        return None


def normalize_filename(path: str) -> str:
    return path.replace("\\", "/")


def parse_threshold_targets(payload: Any, reason_codes: set[str], violations: list[str]) -> list[dict[str, float | str]]:
    if not isinstance(payload, dict):
        reason_codes.add("critical_path_coverage_threshold_schema_invalid")
        violations.append("threshold payload must be an object")
        return []
    if payload.get("schema_version") != THRESHOLD_SCHEMA_VERSION:
        reason_codes.add("critical_path_coverage_threshold_schema_invalid")
        violations.append("threshold schema_version mismatch")
        return []
    raw_targets = payload.get("targets")
    if not isinstance(raw_targets, list):
        reason_codes.add("critical_path_coverage_threshold_schema_invalid")
        violations.append("threshold payload must include targets[]")
        return []

    targets: list[dict[str, float | str]] = []
    for idx, row in enumerate(raw_targets):
        if not isinstance(row, dict):
            reason_codes.add("critical_path_coverage_target_threshold_invalid")
            violations.append(f"targets[{idx}] must be an object")
            continue
        path = row.get("path")
        line_percent_min = row.get("line_percent_min")
        function_percent_min = row.get("function_percent_min")
        if not isinstance(path, str) or not path.strip():
            reason_codes.add("critical_path_coverage_target_threshold_invalid")
            violations.append(f"targets[{idx}] path must be non-empty string")
            continue
        if not isinstance(line_percent_min, (float, int)) or not isinstance(function_percent_min, (float, int)):
            reason_codes.add("critical_path_coverage_target_threshold_invalid")
            violations.append(f"targets[{idx}] threshold values must be numbers")
            continue
        targets.append(
            {
                "path": normalize_filename(path.strip()),
                "line_percent_min": float(line_percent_min),
                "function_percent_min": float(function_percent_min),
            }
        )
    return targets


def parse_coverage_report(path: Path, reason_codes: set[str], violations: list[str]) -> dict[str, dict[str, float]]:
    payload = load_json(
        path,
        missing_code="critical_path_coverage_report_missing",
        invalid_code="critical_path_coverage_report_invalid",
        reason_codes=reason_codes,
        violations=violations,
    )
    if payload is None:
        return {}
    if not isinstance(payload, dict):
        reason_codes.add("critical_path_coverage_report_schema_invalid")
        violations.append(f"coverage report is not an object: {path}")
        return {}
    data = payload.get("data")
    if not isinstance(data, list) or not data:
        reason_codes.add("critical_path_coverage_report_schema_invalid")
        violations.append(f"coverage report missing data[]: {path}")
        return {}
    first = data[0]
    if not isinstance(first, dict):
        reason_codes.add("critical_path_coverage_report_schema_invalid")
        violations.append(f"coverage report data[0] invalid: {path}")
        return {}
    files = first.get("files")
    if not isinstance(files, list):
        reason_codes.add("critical_path_coverage_report_schema_invalid")
        violations.append(f"coverage report missing files[]: {path}")
        return {}

    coverage_by_file: dict[str, dict[str, float]] = {}
    for row in files:
        if not isinstance(row, dict):
            continue
        filename = row.get("filename")
        summary = row.get("summary")
        if not isinstance(filename, str) or not isinstance(summary, dict):
            continue
        lines = summary.get("lines")
        functions = summary.get("functions")
        if not isinstance(lines, dict) or not isinstance(functions, dict):
            continue
        line_percent = lines.get("percent")
        function_percent = functions.get("percent")
        if not isinstance(line_percent, (float, int)) or not isinstance(function_percent, (float, int)):
            continue
        coverage_by_file[normalize_filename(filename)] = {
            "line_percent": float(line_percent),
            "function_percent": float(function_percent),
        }
    return coverage_by_file


def ordered_reason_codes(reason_codes: set[str]) -> list[str]:
    order = {code: idx for idx, code in enumerate(ORDERED_REASON_CODES)}
    return sorted(reason_codes, key=lambda code: (order.get(code, len(order)), code))


def find_target_metrics(target_path: str, coverage_by_file: dict[str, dict[str, float]]) -> dict[str, float] | None:
    for filename, metrics in coverage_by_file.items():
        if filename.endswith(target_path):
            return metrics
    return None


def merge_coverage_metrics(
    coverage_by_file: dict[str, dict[str, float]],
    new_metrics_by_file: dict[str, dict[str, float]],
) -> None:
    for filename, metrics in new_metrics_by_file.items():
        existing = coverage_by_file.get(filename)
        if existing is None:
            coverage_by_file[filename] = metrics
            continue
        coverage_by_file[filename] = {
            "line_percent": max(existing["line_percent"], metrics["line_percent"]),
            "function_percent": max(existing["function_percent"], metrics["function_percent"]),
        }


def main() -> int:
    args = parse_args()
    reason_codes: set[str] = set()
    violations: list[str] = []

    threshold_payload = load_json(
        Path(args.threshold_file),
        missing_code="critical_path_coverage_threshold_file_missing",
        invalid_code="critical_path_coverage_threshold_file_invalid",
        reason_codes=reason_codes,
        violations=violations,
    )
    targets = parse_threshold_targets(threshold_payload, reason_codes, violations) if threshold_payload is not None else []

    coverage_by_file: dict[str, dict[str, float]] = {}
    for coverage_path in (Path(args.core_coverage_json), Path(args.node_coverage_json)):
        merge_coverage_metrics(
            coverage_by_file,
            parse_coverage_report(coverage_path, reason_codes, violations),
        )

    target_reports: list[dict[str, Any]] = []
    line_failures = 0
    function_failures = 0
    missing_targets = 0

    for target in targets:
        target_path = str(target["path"])
        line_percent_min = float(target["line_percent_min"])
        function_percent_min = float(target["function_percent_min"])
        metrics = find_target_metrics(target_path, coverage_by_file)
        target_report: dict[str, Any] = {
            "path": target_path,
            "line_percent_min": line_percent_min,
            "function_percent_min": function_percent_min,
            "line_percent_actual": None,
            "function_percent_actual": None,
            "status": "missing",
        }
        if metrics is None:
            reason_codes.add("critical_path_coverage_target_missing")
            missing_targets += 1
            target_reports.append(target_report)
            continue
        line_percent_actual = float(metrics["line_percent"])
        function_percent_actual = float(metrics["function_percent"])
        target_report["line_percent_actual"] = line_percent_actual
        target_report["function_percent_actual"] = function_percent_actual
        target_report["status"] = "ok"
        if line_percent_actual < line_percent_min:
            reason_codes.add("critical_path_coverage_line_below_threshold")
            line_failures += 1
            target_report["status"] = "fail"
        if function_percent_actual < function_percent_min:
            reason_codes.add("critical_path_coverage_function_below_threshold")
            function_failures += 1
            target_report["status"] = "fail"
        target_reports.append(target_report)

    ordered = ordered_reason_codes(reason_codes)
    final_decision = "GO" if not ordered else "NO-GO"
    status = "ok" if final_decision == "GO" else "fail"
    failed_targets = sum(1 for row in target_reports if row["status"] != "ok")

    report = {
        "schema_version": SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": ordered,
        "reason_codes_csv": ",".join(ordered) if ordered else "none",
        "threshold_file": args.threshold_file,
        "coverage_reports": [args.core_coverage_json, args.node_coverage_json],
        "target_count": len(target_reports),
        "failed_targets": failed_targets,
        "missing_targets": missing_targets,
        "line_failures": line_failures,
        "function_failures": function_failures,
        "targets": target_reports,
        "violations": violations,
    }

    Path(args.output_json).write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={report['reason_codes_csv']}")
    print(f"target_count={len(target_reports)}")
    print(f"failed_targets={failed_targets}")
    print(f"missing_targets={missing_targets}")
    print(f"line_failures={line_failures}")
    print(f"function_failures={function_failures}")
    print(f"coverage_policy_report_file={args.output_json}")

    return 0 if status == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())

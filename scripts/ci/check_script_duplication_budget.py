#!/usr/bin/env python3
"""Enforce script-surface and duplication budgets with optional waivers."""

from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import dataclass
from datetime import date
import hashlib
import json
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class BudgetThresholds:
    script_count_max: int
    shell_line_total_max: int
    duplicate_basename_max: int
    duplicate_content_max: int


@dataclass(frozen=True)
class ScriptMetrics:
    script_count: int
    shell_line_total: int
    duplicate_basename: int
    duplicate_content: int


def _parse_key_value_budget_file(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line == "" or line.startswith("#"):
            continue
        if "=" not in line:
            raise ValueError(f"invalid budget line (expected KEY=VALUE): {raw_line}")
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key == "" or value == "":
            raise ValueError(f"invalid budget line (empty key/value): {raw_line}")
        values[key] = value
    return values


def _to_positive_int(raw_value: str, key: str) -> int:
    if not raw_value.isdigit():
        raise ValueError(f"{key} must be a non-negative integer")
    return int(raw_value)


def load_thresholds(path: Path) -> BudgetThresholds:
    values = _parse_key_value_budget_file(path)
    required_keys = (
        "SCRIPT_COUNT_MAX",
        "SHELL_LINE_TOTAL_MAX",
        "DUPLICATE_BASENAME_MAX",
        "DUPLICATE_CONTENT_MAX",
    )
    for key in required_keys:
        if key not in values:
            raise ValueError(f"missing required budget key: {key}")

    return BudgetThresholds(
        script_count_max=_to_positive_int(values["SCRIPT_COUNT_MAX"], "SCRIPT_COUNT_MAX"),
        shell_line_total_max=_to_positive_int(values["SHELL_LINE_TOTAL_MAX"], "SHELL_LINE_TOTAL_MAX"),
        duplicate_basename_max=_to_positive_int(
            values["DUPLICATE_BASENAME_MAX"], "DUPLICATE_BASENAME_MAX"
        ),
        duplicate_content_max=_to_positive_int(
            values["DUPLICATE_CONTENT_MAX"], "DUPLICATE_CONTENT_MAX"
        ),
    )


def load_baseline_metrics(path: Path) -> ScriptMetrics:
    values = _parse_key_value_budget_file(path)
    required_keys = (
        "SCRIPT_COUNT_BASELINE",
        "SHELL_LINE_TOTAL_BASELINE",
        "DUPLICATE_BASENAME_BASELINE",
        "DUPLICATE_CONTENT_BASELINE",
    )
    for key in required_keys:
        if key not in values:
            raise ValueError(f"missing required baseline key: {key}")

    return ScriptMetrics(
        script_count=_to_positive_int(values["SCRIPT_COUNT_BASELINE"], "SCRIPT_COUNT_BASELINE"),
        shell_line_total=_to_positive_int(
            values["SHELL_LINE_TOTAL_BASELINE"], "SHELL_LINE_TOTAL_BASELINE"
        ),
        duplicate_basename=_to_positive_int(
            values["DUPLICATE_BASENAME_BASELINE"], "DUPLICATE_BASENAME_BASELINE"
        ),
        duplicate_content=_to_positive_int(
            values["DUPLICATE_CONTENT_BASELINE"], "DUPLICATE_CONTENT_BASELINE"
        ),
    )


def compute_metrics(scripts_root: Path) -> ScriptMetrics:
    def include_in_budget(path: Path) -> bool:
        return path.is_file() and not path.name.startswith("test_")

    scripts = sorted(
        path for path in scripts_root.rglob("*.sh") if include_in_budget(path)
    )
    script_count = len(scripts)
    def shell_lines_for_budget(path: Path) -> int:
        # Symlink wrappers represent command-surface entries, not full copies
        # of the target implementation body.
        if path.is_symlink():
            return 1
        return sum(1 for _ in path.open("r", encoding="utf-8", errors="ignore"))

    shell_line_total = sum(shell_lines_for_budget(path) for path in scripts)

    basename_counts = Counter(path.name for path in scripts)
    duplicate_basename = sum(
        count for count in basename_counts.values() if count > 1
    )

    # Treat symlink wrappers as intentional command-surface entries and only
    # enforce duplicate-content policy across regular files.
    content_counts = Counter(
        hashlib.sha256(path.read_bytes()).hexdigest()
        for path in scripts
        if not path.is_symlink()
    )
    duplicate_content = sum(
        count for count in content_counts.values() if count > 1
    )

    return ScriptMetrics(
        script_count=script_count,
        shell_line_total=shell_line_total,
        duplicate_basename=duplicate_basename,
        duplicate_content=duplicate_content,
    )


def compute_metric_deltas(metrics: ScriptMetrics, baseline: ScriptMetrics) -> ScriptMetrics:
    return ScriptMetrics(
        script_count=metrics.script_count - baseline.script_count,
        shell_line_total=metrics.shell_line_total - baseline.shell_line_total,
        duplicate_basename=metrics.duplicate_basename - baseline.duplicate_basename,
        duplicate_content=metrics.duplicate_content - baseline.duplicate_content,
    )


def evaluate_violations(metrics: ScriptMetrics, thresholds: BudgetThresholds) -> list[str]:
    violations: list[str] = []
    if metrics.script_count > thresholds.script_count_max:
        violations.append("script_count")
    if metrics.shell_line_total > thresholds.shell_line_total_max:
        violations.append("shell_line_total")
    if metrics.duplicate_basename > thresholds.duplicate_basename_max:
        violations.append("duplicate_basename")
    if metrics.duplicate_content > thresholds.duplicate_content_max:
        violations.append("duplicate_content")
    return violations


def parse_waiver(path: Path, today: date) -> set[str]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("waiver file must contain an object")

    reason = payload.get("reason")
    if not isinstance(reason, str) or reason.strip() == "":
        raise ValueError("waiver reason is required")

    expires_on = payload.get("expires_on")
    if not isinstance(expires_on, str):
        raise ValueError("waiver expires_on is required")
    try:
        expiry_date = date.fromisoformat(expires_on)
    except ValueError as exc:
        raise ValueError("waiver expires_on must use YYYY-MM-DD") from exc

    if expiry_date < today:
        raise ValueError("waiver has expired")

    allow_metrics = payload.get("allow_metrics")
    if not isinstance(allow_metrics, list) or len(allow_metrics) == 0:
        raise ValueError("waiver allow_metrics must be a non-empty list")

    allowed: set[str] = set()
    for item in allow_metrics:
        if not isinstance(item, str) or item.strip() == "":
            raise ValueError("waiver allow_metrics entries must be non-empty strings")
        allowed.add(item.strip())
    return allowed


def build_remediation_guidance(pending: list[str], waiver_error: str, budget_file: Path, waiver_file: Path) -> str:
    if not pending and waiver_error == "":
        return "none"

    if pending:
        metrics = ",".join(sorted(pending))
        return (
            f"reduce metrics ({metrics}) under thresholds in {budget_file} "
            f"or add temporary waiver in {waiver_file} with reason/expires_on/allow_metrics"
        )

    return (
        f"fix waiver in {waiver_file} (reason, expires_on=YYYY-MM-DD, allow_metrics) "
        f"or remove it to enforce thresholds in {budget_file}"
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check script-surface duplication budgets."
    )
    parser.add_argument(
        "--scripts-root",
        default="scripts",
        help="Root directory to scan for .sh scripts (default: scripts).",
    )
    parser.add_argument(
        "--budget-file",
        default=".ci/script-surface-budget.env",
        help="Budget config file path.",
    )
    parser.add_argument(
        "--baseline-file",
        default=".ci/script-surface-baseline.env",
        help="Baseline metric file path for delta reporting.",
    )
    parser.add_argument(
        "--waiver-file",
        default=".ci/script-surface-budget-waiver.json",
        help="Optional waiver JSON path.",
    )
    parser.add_argument(
        "--today",
        default="",
        help="Override current date (YYYY-MM-DD), for deterministic tests.",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output JSON report path.",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    scripts_root = Path(args.scripts_root)
    budget_file = Path(args.budget_file)
    baseline_file = Path(args.baseline_file)
    waiver_file = Path(args.waiver_file)

    if not scripts_root.is_dir():
        print("status=fail")
        print(f"error=scripts root not found: {scripts_root}")
        return 1
    if not budget_file.is_file():
        print("status=fail")
        print(f"error=budget file not found: {budget_file}")
        return 1
    if not baseline_file.is_file():
        print("status=fail")
        print(f"error=baseline file not found: {baseline_file}")
        return 1

    try:
        today = date.fromisoformat(args.today) if args.today else date.today()
    except ValueError:
        print("status=fail")
        print("error=--today must be YYYY-MM-DD")
        return 1

    try:
        thresholds = load_thresholds(budget_file)
        baseline_metrics = load_baseline_metrics(baseline_file)
        metrics = compute_metrics(scripts_root)
        deltas = compute_metric_deltas(metrics, baseline_metrics)
        violations = evaluate_violations(metrics, thresholds)
    except ValueError as exc:
        print("status=fail")
        print(f"error={exc}")
        return 1

    waived: list[str] = []
    pending = violations.copy()
    waiver_error = ""
    if violations and waiver_file.is_file():
        try:
            allowed = parse_waiver(waiver_file, today)
            waived = sorted(metric for metric in violations if metric in allowed)
            pending = [metric for metric in violations if metric not in allowed]
        except ValueError as exc:
            waiver_error = str(exc)

    status = "pass" if not pending and waiver_error == "" else "fail"
    violations_csv = "none" if not violations else ",".join(sorted(violations))
    waived_csv = "none" if not waived else ",".join(waived)
    pending_csv = "none" if not pending else ",".join(sorted(pending))
    remediation = build_remediation_guidance(pending, waiver_error, budget_file, waiver_file)

    print(f"status={status}")
    print(f"script_count={metrics.script_count}")
    print(f"shell_line_total={metrics.shell_line_total}")
    print(f"duplicate_basename={metrics.duplicate_basename}")
    print(f"duplicate_content={metrics.duplicate_content}")
    print(f"delta_script_count={deltas.script_count}")
    print(f"delta_shell_line_total={deltas.shell_line_total}")
    print(f"delta_duplicate_basename={deltas.duplicate_basename}")
    print(f"delta_duplicate_content={deltas.duplicate_content}")
    print(f"violations={violations_csv}")
    print(f"waived={waived_csv}")
    print(f"pending={pending_csv}")
    print(f"remediation={remediation}")
    if waiver_error:
        print(f"waiver_error={waiver_error}")

    if args.output_json:
        report = {
            "schema_version": "kamn.ci.script-surface-budget-report.v1",
            "status": status,
            "scripts_root": str(scripts_root),
            "budget_file": str(budget_file),
            "baseline_file": str(baseline_file),
            "waiver_file": str(waiver_file),
            "metrics": {
                "script_count": metrics.script_count,
                "shell_line_total": metrics.shell_line_total,
                "duplicate_basename": metrics.duplicate_basename,
                "duplicate_content": metrics.duplicate_content,
            },
            "baseline_metrics": {
                "script_count": baseline_metrics.script_count,
                "shell_line_total": baseline_metrics.shell_line_total,
                "duplicate_basename": baseline_metrics.duplicate_basename,
                "duplicate_content": baseline_metrics.duplicate_content,
            },
            "deltas": {
                "script_count": deltas.script_count,
                "shell_line_total": deltas.shell_line_total,
                "duplicate_basename": deltas.duplicate_basename,
                "duplicate_content": deltas.duplicate_content,
            },
            "thresholds": {
                "script_count_max": thresholds.script_count_max,
                "shell_line_total_max": thresholds.shell_line_total_max,
                "duplicate_basename_max": thresholds.duplicate_basename_max,
                "duplicate_content_max": thresholds.duplicate_content_max,
            },
            "violations": sorted(violations),
            "waived": waived,
            "pending": sorted(pending),
            "remediation": remediation,
            "waiver_error": waiver_error,
        }
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
        print(f"report_file={output_path}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main(__import__("sys").argv[1:]))

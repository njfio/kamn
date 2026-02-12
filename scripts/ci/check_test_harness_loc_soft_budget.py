#!/usr/bin/env python3
"""Evaluate test-harness LOC growth against non-blocking soft budgets."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys


INPUT_SCHEMA = "kamn.ci.test-harness-loc-report.v1"
OUTPUT_SCHEMA = "kamn.ci.test-harness-loc-soft-budget-report.v1"


def parse_key_value_budget_file(path: Path) -> dict[str, str]:
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


def to_non_negative_int(raw_value: str, key: str) -> int:
    if not raw_value.isdigit():
        raise ValueError(f"{key} must be a non-negative integer")
    return int(raw_value)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check test-harness LOC soft budgets."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument(
        "--budget-file",
        default=".ci/test-harness-loc-soft-budget.env",
        help="Soft budget configuration file path.",
    )
    parser.add_argument(
        "--baseline-file",
        default=".ci/test-harness-loc-baseline.env",
        help="Baseline configuration file path.",
    )
    parser.add_argument("--output-json", default="")
    return parser.parse_args(argv)


def fail(error: str) -> int:
    print("status=fail")
    print(f"error={error}")
    return 1


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    report_file = Path(args.report_file)
    budget_file = Path(args.budget_file)
    baseline_file = Path(args.baseline_file)

    if not report_file.is_file():
        return fail(f"report file not found: {report_file}")
    if not budget_file.is_file():
        return fail(f"budget file not found: {budget_file}")
    if not baseline_file.is_file():
        return fail(f"baseline file not found: {baseline_file}")

    try:
        report = json.loads(report_file.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        return fail(f"report file is not valid JSON: {error}")

    if report.get("schema_version") != INPUT_SCHEMA:
        return fail(f"unexpected report schema: {report.get('schema_version')}")

    harness_script_count = report.get("harness_script_count")
    if not isinstance(harness_script_count, int) or harness_script_count < 0:
        return fail("report harness_script_count must be a non-negative integer")

    harness_shell_line_total = report.get("harness_shell_line_total")
    if not isinstance(harness_shell_line_total, int) or harness_shell_line_total < 0:
        return fail("report harness_shell_line_total must be a non-negative integer")

    try:
        budget_values = parse_key_value_budget_file(budget_file)
        soft_max_script_count = to_non_negative_int(
            budget_values["TEST_HARNESS_SCRIPT_COUNT_SOFT_MAX"],
            "TEST_HARNESS_SCRIPT_COUNT_SOFT_MAX",
        )
        soft_max_shell_line_total = to_non_negative_int(
            budget_values["TEST_HARNESS_SHELL_LINE_TOTAL_SOFT_MAX"],
            "TEST_HARNESS_SHELL_LINE_TOTAL_SOFT_MAX",
        )
    except KeyError as error:
        return fail(f"missing required budget key: {error.args[0]}")
    except ValueError as error:
        return fail(str(error))

    try:
        baseline_values = parse_key_value_budget_file(baseline_file)
        baseline_script_count = to_non_negative_int(
            baseline_values["TEST_HARNESS_SCRIPT_COUNT_BASELINE"],
            "TEST_HARNESS_SCRIPT_COUNT_BASELINE",
        )
        baseline_shell_line_total = to_non_negative_int(
            baseline_values["TEST_HARNESS_SHELL_LINE_TOTAL_BASELINE"],
            "TEST_HARNESS_SHELL_LINE_TOTAL_BASELINE",
        )
    except KeyError as error:
        return fail(f"missing required baseline key: {error.args[0]}")
    except ValueError as error:
        return fail(str(error))

    exceeded_metrics: list[str] = []
    if harness_script_count > soft_max_script_count:
        exceeded_metrics.append("harness_script_count")
    if harness_shell_line_total > soft_max_shell_line_total:
        exceeded_metrics.append("harness_shell_line_total")

    soft_budget_status = "within" if not exceeded_metrics else "exceeded"
    review_required = bool(exceeded_metrics)
    delta_harness_script_count = harness_script_count - baseline_script_count
    delta_harness_shell_line_total = harness_shell_line_total - baseline_shell_line_total

    output = {
        "schema_version": OUTPUT_SCHEMA,
        "report_file": str(report_file.resolve()),
        "budget_file": str(budget_file.resolve()),
        "baseline_file": str(baseline_file.resolve()),
        "budget_mode": "soft",
        "status": "ok",
        "soft_budget_status": soft_budget_status,
        "review_required": review_required,
        "exceeded_metrics": exceeded_metrics,
        "harness_script_count": harness_script_count,
        "harness_shell_line_total": harness_shell_line_total,
        "soft_max_harness_script_count": soft_max_script_count,
        "soft_max_harness_shell_line_total": soft_max_shell_line_total,
        "baseline_harness_script_count": baseline_script_count,
        "baseline_harness_shell_line_total": baseline_shell_line_total,
        "delta_harness_script_count": delta_harness_script_count,
        "delta_harness_shell_line_total": delta_harness_shell_line_total,
    }

    output_path: Path | None = None
    if args.output_json:
        output_path = Path(args.output_json).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(
            json.dumps(output, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    exceeded_marker = "none" if not exceeded_metrics else ",".join(exceeded_metrics)
    print("status=ok")
    print("budget_mode=soft")
    print(f"soft_budget_status={soft_budget_status}")
    print(f"review_required={'true' if review_required else 'false'}")
    print(f"exceeded_metrics={exceeded_marker}")
    print(f"harness_script_count={harness_script_count}")
    print(f"harness_shell_line_total={harness_shell_line_total}")
    print(f"delta_harness_script_count={delta_harness_script_count}")
    print(f"delta_harness_shell_line_total={delta_harness_shell_line_total}")
    if output_path is not None:
        print(f"report_file={output_path}")

    # Soft budgets are advisory only: threshold breaches do not fail CI.
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

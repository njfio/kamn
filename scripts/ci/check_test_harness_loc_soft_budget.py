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
    parser.add_argument(
        "--trend-threshold-file",
        default="",
        help=(
            "Optional trend threshold file path. When set, warning/fail trend states "
            "are evaluated against baseline deltas."
        ),
    )
    parser.add_argument(
        "--enforce-trend-fail",
        action="store_true",
        help="Return non-zero when trend_status=fail.",
    )
    parser.add_argument("--output-json", default="")
    return parser.parse_args(argv)


def fail(error: str, reason_code: str) -> int:
    print("status=fail")
    print(f"reason_codes={reason_code}")
    print(f"error={error}")
    return 1


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    report_file = Path(args.report_file)
    budget_file = Path(args.budget_file)
    baseline_file = Path(args.baseline_file)
    trend_threshold_file: Path | None = (
        Path(args.trend_threshold_file) if args.trend_threshold_file else None
    )

    if not report_file.is_file():
        return fail(f"report file not found: {report_file}", "report_file_not_found")
    if not budget_file.is_file():
        return fail(f"budget file not found: {budget_file}", "budget_file_not_found")
    if not baseline_file.is_file():
        return fail(f"baseline file not found: {baseline_file}", "baseline_file_not_found")
    if trend_threshold_file is not None and not trend_threshold_file.is_file():
        return fail(
            f"trend threshold file not found: {trend_threshold_file}",
            "trend_threshold_file_not_found",
        )

    try:
        report = json.loads(report_file.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        return fail(f"report file is not valid JSON: {error}", "report_json_invalid")

    if report.get("schema_version") != INPUT_SCHEMA:
        return fail(
            f"unexpected report schema: {report.get('schema_version')}",
            "report_schema_mismatch",
        )

    harness_script_count = report.get("harness_script_count")
    if not isinstance(harness_script_count, int) or harness_script_count < 0:
        return fail(
            "report harness_script_count must be a non-negative integer",
            "report_harness_script_count_invalid",
        )

    harness_shell_line_total = report.get("harness_shell_line_total")
    if not isinstance(harness_shell_line_total, int) or harness_shell_line_total < 0:
        return fail(
            "report harness_shell_line_total must be a non-negative integer",
            "report_harness_shell_line_total_invalid",
        )

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
        return fail(
            f"missing required budget key: {error.args[0]}",
            "budget_key_missing",
        )
    except ValueError as error:
        return fail(str(error), "budget_value_invalid")

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
        return fail(
            f"missing required baseline key: {error.args[0]}",
            "baseline_key_missing",
        )
    except ValueError as error:
        return fail(str(error), "baseline_value_invalid")

    trend_thresholds_enabled = trend_threshold_file is not None
    warn_delta_script_count = 0
    fail_delta_script_count = 0
    warn_delta_shell_line_total = 0
    fail_delta_shell_line_total = 0
    if trend_thresholds_enabled:
        try:
            trend_values = parse_key_value_budget_file(trend_threshold_file)
            warn_delta_script_count = to_non_negative_int(
                trend_values["TEST_HARNESS_SCRIPT_COUNT_WARN_DELTA_MAX"],
                "TEST_HARNESS_SCRIPT_COUNT_WARN_DELTA_MAX",
            )
            fail_delta_script_count = to_non_negative_int(
                trend_values["TEST_HARNESS_SCRIPT_COUNT_FAIL_DELTA_MAX"],
                "TEST_HARNESS_SCRIPT_COUNT_FAIL_DELTA_MAX",
            )
            warn_delta_shell_line_total = to_non_negative_int(
                trend_values["TEST_HARNESS_SHELL_LINE_TOTAL_WARN_DELTA_MAX"],
                "TEST_HARNESS_SHELL_LINE_TOTAL_WARN_DELTA_MAX",
            )
            fail_delta_shell_line_total = to_non_negative_int(
                trend_values["TEST_HARNESS_SHELL_LINE_TOTAL_FAIL_DELTA_MAX"],
                "TEST_HARNESS_SHELL_LINE_TOTAL_FAIL_DELTA_MAX",
            )
        except KeyError as error:
            return fail(
                f"missing required trend threshold key: {error.args[0]}",
                "trend_threshold_key_missing",
            )
        except ValueError as error:
            return fail(str(error), "trend_threshold_value_invalid")

        if fail_delta_script_count < warn_delta_script_count:
            return fail(
                "TEST_HARNESS_SCRIPT_COUNT_FAIL_DELTA_MAX must be >= TEST_HARNESS_SCRIPT_COUNT_WARN_DELTA_MAX",
                "trend_threshold_order_invalid",
            )
        if fail_delta_shell_line_total < warn_delta_shell_line_total:
            return fail(
                "TEST_HARNESS_SHELL_LINE_TOTAL_FAIL_DELTA_MAX must be >= TEST_HARNESS_SHELL_LINE_TOTAL_WARN_DELTA_MAX",
                "trend_threshold_order_invalid",
            )

    exceeded_metrics: list[str] = []
    soft_budget_reason_codes: list[str] = []
    if harness_script_count > soft_max_script_count:
        exceeded_metrics.append("harness_script_count")
        soft_budget_reason_codes.append("harness_script_count_soft_max_exceeded")
    if harness_shell_line_total > soft_max_shell_line_total:
        exceeded_metrics.append("harness_shell_line_total")
        soft_budget_reason_codes.append("harness_shell_line_total_soft_max_exceeded")

    soft_budget_status = "within" if not exceeded_metrics else "exceeded"
    delta_harness_script_count = harness_script_count - baseline_script_count
    delta_harness_shell_line_total = harness_shell_line_total - baseline_shell_line_total

    trend_warning_metrics: list[str] = []
    trend_fail_metrics: list[str] = []
    trend_reason_codes: list[str] = []
    if trend_thresholds_enabled:
        if delta_harness_script_count > fail_delta_script_count:
            trend_fail_metrics.append("harness_script_count")
            trend_reason_codes.append("harness_script_count_trend_fail_delta_exceeded")
        elif delta_harness_script_count > warn_delta_script_count:
            trend_warning_metrics.append("harness_script_count")
            trend_reason_codes.append("harness_script_count_trend_warn_delta_exceeded")

        if delta_harness_shell_line_total > fail_delta_shell_line_total:
            trend_fail_metrics.append("harness_shell_line_total")
            trend_reason_codes.append("harness_shell_line_total_trend_fail_delta_exceeded")
        elif delta_harness_shell_line_total > warn_delta_shell_line_total:
            trend_warning_metrics.append("harness_shell_line_total")
            trend_reason_codes.append("harness_shell_line_total_trend_warn_delta_exceeded")

    if not trend_thresholds_enabled:
        trend_status = "not_configured"
    elif trend_fail_metrics:
        trend_status = "fail"
    elif trend_warning_metrics:
        trend_status = "warn"
    else:
        trend_status = "within"

    if trend_status == "fail":
        policy_decision = "NO-GO"
    elif trend_status == "warn" or soft_budget_status == "exceeded":
        policy_decision = "WARN"
    else:
        policy_decision = "GO"

    combined_reason_codes = soft_budget_reason_codes + trend_reason_codes
    review_required = bool(exceeded_metrics or trend_warning_metrics or trend_fail_metrics)
    status = "ok"
    exit_code = 0
    if args.enforce_trend_fail and trend_status == "fail":
        status = "fail"
        exit_code = 1

    output = {
        "schema_version": OUTPUT_SCHEMA,
        "report_file": str(report_file.resolve()),
        "budget_file": str(budget_file.resolve()),
        "baseline_file": str(baseline_file.resolve()),
        "trend_threshold_file": (
            str(trend_threshold_file.resolve()) if trend_threshold_file is not None else ""
        ),
        "budget_mode": "soft",
        "status": status,
        "soft_budget_status": soft_budget_status,
        "trend_thresholds_enabled": trend_thresholds_enabled,
        "trend_status": trend_status,
        "policy_decision": policy_decision,
        "review_required": review_required,
        "exceeded_metrics": exceeded_metrics,
        "trend_warning_metrics": trend_warning_metrics,
        "trend_fail_metrics": trend_fail_metrics,
        "soft_budget_reason_codes": soft_budget_reason_codes,
        "trend_reason_codes": trend_reason_codes,
        "reason_codes": combined_reason_codes,
        "harness_script_count": harness_script_count,
        "harness_shell_line_total": harness_shell_line_total,
        "soft_max_harness_script_count": soft_max_script_count,
        "soft_max_harness_shell_line_total": soft_max_shell_line_total,
        "baseline_harness_script_count": baseline_script_count,
        "baseline_harness_shell_line_total": baseline_shell_line_total,
        "delta_harness_script_count": delta_harness_script_count,
        "delta_harness_shell_line_total": delta_harness_shell_line_total,
        "warn_delta_harness_script_count_max": warn_delta_script_count,
        "fail_delta_harness_script_count_max": fail_delta_script_count,
        "warn_delta_harness_shell_line_total_max": warn_delta_shell_line_total,
        "fail_delta_harness_shell_line_total_max": fail_delta_shell_line_total,
        "enforce_trend_fail": args.enforce_trend_fail,
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
    print(f"status={status}")
    print("budget_mode=soft")
    print(f"soft_budget_status={soft_budget_status}")
    print(f"trend_status={trend_status}")
    print(f"policy_decision={policy_decision}")
    print(f"review_required={'true' if review_required else 'false'}")
    print(f"exceeded_metrics={exceeded_marker}")
    print(
        f"trend_warning_metrics={'none' if not trend_warning_metrics else ','.join(trend_warning_metrics)}"
    )
    print(
        f"trend_fail_metrics={'none' if not trend_fail_metrics else ','.join(trend_fail_metrics)}"
    )
    print(
        f"reason_codes={'none' if not combined_reason_codes else ','.join(combined_reason_codes)}"
    )
    print(f"harness_script_count={harness_script_count}")
    print(f"harness_shell_line_total={harness_shell_line_total}")
    print(f"delta_harness_script_count={delta_harness_script_count}")
    print(f"delta_harness_shell_line_total={delta_harness_shell_line_total}")
    if output_path is not None:
        print(f"report_file={output_path}")

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

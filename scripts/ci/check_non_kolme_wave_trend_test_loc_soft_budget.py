#!/usr/bin/env python3
"""Check non-Kolme wave trend-test script LOC against soft budget thresholds."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

BASELINE_SCHEMA = "kamn.ci.non-kolme-wave-trend-test-loc-baseline.v1"
THRESHOLDS_SCHEMA = "kamn.ci.non-kolme-wave-trend-test-loc-thresholds.v1"
OUTPUT_SCHEMA = "kamn.ci.non-kolme-wave-trend-test-loc-soft-budget-report.v1"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check non-Kolme wave trend-test LOC soft budgets."
    )
    parser.add_argument(
        "--baseline-file",
        default="fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_baseline.json",
        help="Baseline fixture path.",
    )
    parser.add_argument(
        "--threshold-file",
        default="fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_thresholds.json",
        help="Trend threshold fixture path.",
    )
    parser.add_argument("--output-json", default="", help="Optional JSON output path.")
    return parser.parse_args(argv)


def read_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(reason_code: str, error: str) -> int:
    print("status=fail")
    print(f"reason_codes={reason_code}")
    print(f"error={error}")
    return 1


def collect_current_scripts(root_dir: Path) -> tuple[list[str], int]:
    wrapper_paths = sorted(
        root_dir.glob("scripts/ci/test_check_non_kolme_wave*_wrapper_family_budget_trend.sh")
    )
    impl_path = root_dir / "scripts/ci/test_check_non_kolme_wave_wrapper_family_budget_trend_impl.sh"

    current_paths: list[Path] = [path for path in wrapper_paths if path.is_file()]
    if impl_path.is_file():
        current_paths.append(impl_path)

    relative_paths = [path.relative_to(root_dir).as_posix() for path in current_paths]
    total_shell_loc = 0
    for path in current_paths:
        with path.open("r", encoding="utf-8") as handle:
            total_shell_loc += sum(1 for _ in handle)

    return relative_paths, total_shell_loc


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    root_dir = Path(__file__).resolve().parents[2]

    baseline_file = (root_dir / args.baseline_file).resolve()
    threshold_file = (root_dir / args.threshold_file).resolve()

    if not baseline_file.is_file():
        return fail("baseline_file_not_found", f"baseline file not found: {baseline_file}")
    if not threshold_file.is_file():
        return fail("threshold_file_not_found", f"threshold file not found: {threshold_file}")

    try:
        baseline = read_json(baseline_file)
    except json.JSONDecodeError as error:
        return fail("baseline_json_invalid", f"baseline JSON invalid: {error}")

    try:
        thresholds = read_json(threshold_file)
    except json.JSONDecodeError as error:
        return fail("threshold_json_invalid", f"threshold JSON invalid: {error}")

    if baseline.get("schema_version") != BASELINE_SCHEMA:
        return fail(
            "baseline_schema_mismatch",
            f"unexpected baseline schema: {baseline.get('schema_version')}",
        )
    if thresholds.get("schema_version") != THRESHOLDS_SCHEMA:
        return fail(
            "threshold_schema_mismatch",
            f"unexpected threshold schema: {thresholds.get('schema_version')}",
        )

    baseline_script_count = baseline.get("script_count")
    baseline_total_shell_loc = baseline.get("total_shell_loc")
    baseline_script_files = baseline.get("script_files")

    if not isinstance(baseline_script_count, int) or baseline_script_count < 0:
        return fail("baseline_script_count_invalid", "baseline script_count must be non-negative int")
    if not isinstance(baseline_total_shell_loc, int) or baseline_total_shell_loc < 0:
        return fail(
            "baseline_total_shell_loc_invalid",
            "baseline total_shell_loc must be non-negative int",
        )
    if not isinstance(baseline_script_files, list) or not all(
        isinstance(item, str) for item in baseline_script_files
    ):
        return fail("baseline_script_files_invalid", "baseline script_files must be string list")

    if len(baseline_script_files) != baseline_script_count:
        return fail(
            "baseline_script_count_mismatch",
            "baseline script_count does not match script_files length",
        )

    max_script_count_increase = thresholds.get("max_script_count_increase")
    max_total_shell_loc_increase = thresholds.get("max_total_shell_loc_increase")

    if not isinstance(max_script_count_increase, int) or max_script_count_increase < 0:
        return fail(
            "threshold_script_count_invalid",
            "max_script_count_increase must be non-negative int",
        )
    if not isinstance(max_total_shell_loc_increase, int) or max_total_shell_loc_increase < 0:
        return fail(
            "threshold_total_shell_loc_invalid",
            "max_total_shell_loc_increase must be non-negative int",
        )

    current_script_files, current_total_shell_loc = collect_current_scripts(root_dir)
    current_script_count = len(current_script_files)

    script_count_delta = current_script_count - baseline_script_count
    total_shell_loc_delta = current_total_shell_loc - baseline_total_shell_loc

    current_script_set = set(current_script_files)
    missing_baseline_scripts = [
        script for script in baseline_script_files if script not in current_script_set
    ]

    reason_codes: list[str] = []
    if missing_baseline_scripts:
        reason_codes.append("missing_baseline_scripts")
    if script_count_delta > max_script_count_increase:
        reason_codes.append("script_count_delta_threshold_exceeded")
    if total_shell_loc_delta > max_total_shell_loc_increase:
        reason_codes.append("total_shell_loc_delta_threshold_exceeded")

    status = "pass" if not reason_codes else "fail"
    output = {
        "schema_version": OUTPUT_SCHEMA,
        "status": status,
        "baseline_file": str(baseline_file),
        "threshold_file": str(threshold_file),
        "current_script_count": current_script_count,
        "baseline_script_count": baseline_script_count,
        "script_count_delta": script_count_delta,
        "current_total_shell_loc": current_total_shell_loc,
        "baseline_total_shell_loc": baseline_total_shell_loc,
        "total_shell_loc_delta": total_shell_loc_delta,
        "max_script_count_increase": max_script_count_increase,
        "max_total_shell_loc_increase": max_total_shell_loc_increase,
        "missing_baseline_scripts": missing_baseline_scripts,
        "reason_codes": reason_codes,
        "violation_count": len(reason_codes),
    }

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"status={status}")
    print(f"current_script_count={current_script_count}")
    print(f"baseline_script_count={baseline_script_count}")
    print(f"script_count_delta={script_count_delta}")
    print(f"current_total_shell_loc={current_total_shell_loc}")
    print(f"baseline_total_shell_loc={baseline_total_shell_loc}")
    print(f"total_shell_loc_delta={total_shell_loc_delta}")
    print(f"violation_count={len(reason_codes)}")
    print(
        "reason_codes=" + (",".join(reason_codes) if reason_codes else "none")
    )

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""Check non-Kolme wave trend-test script LOC against soft budget thresholds."""

from __future__ import annotations

import argparse
import datetime as dt
import json
from pathlib import Path
import sys

BASELINE_SCHEMA = "kamn.ci.non-kolme-wave-trend-test-loc-baseline.v1"
THRESHOLDS_SCHEMA = "kamn.ci.non-kolme-wave-trend-test-loc-thresholds.v1"
OUTPUT_SCHEMA = "kamn.ci.non-kolme-wave-trend-test-loc-soft-budget-report.v1"
WAIVER_SCHEMA = "kamn.ci.non-kolme-wave-trend-test-loc-soft-budget-waiver.v1"
WAIVER_SCOPE = "non_kolme_wave_trend_test_loc_soft_budget"


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
    parser.add_argument(
        "--waiver-file",
        default="",
        help="Optional waiver JSON path.",
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


def fail_with_output(
    reason_code: str,
    error: str,
    *,
    output_json: str,
    baseline_file: Path,
    threshold_file: Path,
    waiver_file: Path | None,
) -> int:
    output = {
        "schema_version": OUTPUT_SCHEMA,
        "status": "fail",
        "baseline_file": str(baseline_file),
        "threshold_file": str(threshold_file),
        "waiver_file": str(waiver_file) if waiver_file else "",
        "soft_overrun_status": "within",
        "waiver_status": "none",
        "review_required": False,
        "waived_reason_codes": [],
        "reason_codes": [reason_code],
        "error": error,
        "remediation": "Fix invalid waiver metadata or checker inputs and rerun.",
    }
    if output_json:
        output_path = Path(output_json)
        output_path.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print("status=fail")
    print("soft_overrun_status=within")
    print("waiver_status=none")
    print("review_required=false")
    print("waived_reason_codes=none")
    print(f"reason_codes={reason_code}")
    print(f"error={error}")
    print("remediation=Fix invalid waiver metadata or checker inputs and rerun.")
    return 1


def parse_waiver(
    waiver_file: Path,
    triggered_threshold_reasons: list[str],
) -> tuple[str, list[str], list[str], str] | tuple[None, None, None, None]:
    try:
        waiver_payload = read_json(waiver_file)
    except json.JSONDecodeError:
        return None, None, None, "waiver_file_json_invalid"

    if waiver_payload.get("schema_version") != WAIVER_SCHEMA:
        return None, None, None, "waiver_file_schema_mismatch"

    scope = waiver_payload.get("scope")
    if scope != WAIVER_SCOPE:
        return None, None, None, "waiver_scope_mismatch"

    expires_on = waiver_payload.get("expires_on")
    if not isinstance(expires_on, str):
        return None, None, None, "waiver_expiry_invalid"
    try:
        expiry_date = dt.date.fromisoformat(expires_on)
    except ValueError:
        return None, None, None, "waiver_expiry_invalid"
    if expiry_date < dt.date.today():
        return None, None, None, "waiver_expired"

    allowed_reason_codes = waiver_payload.get("allowed_reason_codes", [])
    if not isinstance(allowed_reason_codes, list) or not all(
        isinstance(value, str) for value in allowed_reason_codes
    ):
        return None, None, None, "waiver_allowed_reason_codes_invalid"

    waived_reason_codes = [
        reason for reason in triggered_threshold_reasons if reason in set(allowed_reason_codes)
    ]
    unwaived_reason_codes = [
        reason for reason in triggered_threshold_reasons if reason not in set(allowed_reason_codes)
    ]
    return "applied", waived_reason_codes, unwaived_reason_codes, ""


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
    waiver_file = (root_dir / args.waiver_file).resolve() if args.waiver_file else None

    if not baseline_file.is_file():
        return fail("baseline_file_not_found", f"baseline file not found: {baseline_file}")
    if not threshold_file.is_file():
        return fail("threshold_file_not_found", f"threshold file not found: {threshold_file}")
    if waiver_file and not waiver_file.is_file():
        return fail_with_output(
            "waiver_file_not_found",
            f"waiver file not found: {waiver_file}",
            output_json=args.output_json,
            baseline_file=baseline_file,
            threshold_file=threshold_file,
            waiver_file=waiver_file,
        )

    try:
        baseline = read_json(baseline_file)
    except json.JSONDecodeError as error:
        return fail("baseline_json_invalid", f"baseline JSON invalid: {error}")

    try:
        thresholds = read_json(threshold_file)
    except json.JSONDecodeError as error:
        return fail("threshold_json_invalid", f"threshold JSON invalid: {error}")

    if baseline.get("schema_version") != BASELINE_SCHEMA:
        return fail_with_output(
            "baseline_schema_mismatch",
            f"unexpected baseline schema: {baseline.get('schema_version')}",
            output_json=args.output_json,
            baseline_file=baseline_file,
            threshold_file=threshold_file,
            waiver_file=waiver_file,
        )
    if thresholds.get("schema_version") != THRESHOLDS_SCHEMA:
        return fail_with_output(
            "threshold_schema_mismatch",
            f"unexpected threshold schema: {thresholds.get('schema_version')}",
            output_json=args.output_json,
            baseline_file=baseline_file,
            threshold_file=threshold_file,
            waiver_file=waiver_file,
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

    baseline_script_set = set(baseline_script_files)
    current_script_set = set(current_script_files)
    missing_baseline_scripts = [
        script for script in baseline_script_files if script not in current_script_set
    ]
    unexpected_current_scripts = [
        script for script in current_script_files if script not in baseline_script_set
    ]

    reason_codes: list[str] = []
    threshold_reason_codes: list[str] = []
    if missing_baseline_scripts:
        reason_codes.append("missing_baseline_scripts")
    if unexpected_current_scripts:
        reason_codes.append("unexpected_current_scripts")
    if script_count_delta > max_script_count_increase:
        threshold_reason_codes.append("script_count_delta_threshold_exceeded")
    if total_shell_loc_delta > max_total_shell_loc_increase:
        threshold_reason_codes.append("total_shell_loc_delta_threshold_exceeded")

    reason_codes.extend(threshold_reason_codes)
    soft_overrun_status = "exceeded" if threshold_reason_codes else "within"
    review_required = soft_overrun_status == "exceeded"
    waiver_status = "none"
    waived_reason_codes: list[str] = []

    if threshold_reason_codes:
        if waiver_file is None:
            reason_codes.append("delta_threshold_violation_unwaived")
        else:
            parsed_waiver = parse_waiver(waiver_file, threshold_reason_codes)
            if parsed_waiver[0] is None:
                return fail_with_output(
                    parsed_waiver[3],
                    "waiver metadata failed validation",
                    output_json=args.output_json,
                    baseline_file=baseline_file,
                    threshold_file=threshold_file,
                    waiver_file=waiver_file,
                )
            waiver_status, waived_reason_codes, unwaived_reason_codes, _ = parsed_waiver
            if unwaived_reason_codes:
                reason_codes.append("delta_threshold_violation_unwaived")
            else:
                reason_codes.append("delta_threshold_waiver_applied")

    status = "pass"
    blocking_reason_codes = {
        "missing_baseline_scripts",
        "unexpected_current_scripts",
        "delta_threshold_violation_unwaived",
    }
    if any(reason in blocking_reason_codes for reason in reason_codes):
        status = "fail"

    remediation = "No action required."
    if status == "fail":
        remediation = "Update baseline/threshold metadata or provide a valid unexpired waiver."
    elif soft_overrun_status == "exceeded":
        remediation = "Trend delta waived; keep reviewer follow-up ticket linked until baseline refresh."

    output = {
        "schema_version": OUTPUT_SCHEMA,
        "status": status,
        "baseline_file": str(baseline_file),
        "threshold_file": str(threshold_file),
        "waiver_file": str(waiver_file) if waiver_file else "",
        "current_script_count": current_script_count,
        "baseline_script_count": baseline_script_count,
        "script_count_delta": script_count_delta,
        "current_total_shell_loc": current_total_shell_loc,
        "baseline_total_shell_loc": baseline_total_shell_loc,
        "total_shell_loc_delta": total_shell_loc_delta,
        "max_script_count_increase": max_script_count_increase,
        "max_total_shell_loc_increase": max_total_shell_loc_increase,
        "missing_baseline_scripts": missing_baseline_scripts,
        "unexpected_current_scripts": unexpected_current_scripts,
        "reason_codes": reason_codes,
        "violation_count": len(reason_codes),
        "soft_overrun_status": soft_overrun_status,
        "waiver_status": waiver_status,
        "review_required": review_required,
        "waived_reason_codes": waived_reason_codes,
        "remediation": remediation,
    }

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"status={status}")
    print(f"soft_overrun_status={soft_overrun_status}")
    print(f"waiver_status={waiver_status}")
    print(f"review_required={'true' if review_required else 'false'}")
    print("waived_reason_codes=" + (",".join(waived_reason_codes) if waived_reason_codes else "none"))
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
    print(f"remediation={remediation}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

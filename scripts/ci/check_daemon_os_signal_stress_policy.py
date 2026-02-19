#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Dict, List

REASON_TAXONOMY_VERSION = "kamn.ci.daemon-os-signal-stress-policy-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "overload_policy_argument_invalid,"
    "overload_policy_ci_tools_fast_mode_heavy_run_leaked,"
    "overload_policy_ci_tools_fast_mode_missing_overload_test,"
    "overload_policy_ci_tools_script_missing,"
    "overload_policy_expected_decision_mismatch,"
    "overload_policy_output_json_required,"
    "overload_policy_reason_code_unknown,"
    "overload_policy_report_file_missing,"
    "overload_policy_report_json_invalid,"
    "overload_policy_report_schema_mismatch,"
    "overload_policy_runtime_budget_exceeded,"
    "overload_policy_threshold_file_missing,"
    "overload_policy_threshold_key_missing,"
    "overload_policy_threshold_value_invalid"
)


def parse_thresholds(path: Path) -> Dict[str, str]:
    values: Dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise ValueError("invalid-threshold-line")
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def emit(
    *,
    output_json: Path,
    status: str,
    final_decision: str,
    reason_codes: str,
    report_schema_version: str,
    runtime_seconds: str,
    max_runtime_seconds: str,
    allowed_reason_codes_csv: str,
    report_file: Path,
    threshold_file: Path,
    ci_tools_script: Path,
    expected_final_decision: str,
    errors: List[str] | None = None,
) -> None:
    payload = {
        "schema_version": "kamn.ci.daemon-os-signal-stress-policy-report.v1",
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": reason_codes,
        "reason_codes_csv": REASON_CODES_CSV,
        "report_schema_version": report_schema_version,
        "runtime_seconds": runtime_seconds,
        "max_runtime_seconds": max_runtime_seconds,
        "allowed_reason_codes_csv": allowed_reason_codes_csv,
        "errors": errors or [],
        "report_file": str(report_file),
        "threshold_file": str(threshold_file),
        "ci_tools_script": str(ci_tools_script),
        "expected_final_decision": expected_final_decision,
    }
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes={reason_codes}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"report_schema_version={report_schema_version}")
    print(f"runtime_seconds={runtime_seconds}")
    print(f"max_runtime_seconds={max_runtime_seconds}")
    print(f"allowed_reason_codes_csv={allowed_reason_codes_csv}")
    if errors:
        for error in errors:
            print(f"overload dry-run policy failed: {error}")


def fail(
    *,
    output_json: Path,
    reason_codes: List[str],
    errors: List[str],
    report_schema_version: str,
    runtime_seconds: str,
    max_runtime_seconds: str,
    allowed_reason_codes_csv: str,
    report_file: Path,
    threshold_file: Path,
    ci_tools_script: Path,
    expected_final_decision: str,
) -> int:
    emit(
        output_json=output_json,
        status="fail",
        final_decision="NO-GO",
        reason_codes=",".join(reason_codes),
        report_schema_version=report_schema_version,
        runtime_seconds=runtime_seconds,
        max_runtime_seconds=max_runtime_seconds,
        allowed_reason_codes_csv=allowed_reason_codes_csv,
        report_file=report_file,
        threshold_file=threshold_file,
        ci_tools_script=ci_tools_script,
        expected_final_decision=expected_final_decision,
        errors=errors,
    )
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Check daemon OS-signal stress matrix dry-run policy")
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--threshold-file", required=True)
    parser.add_argument("--ci-tools-script", required=True)
    parser.add_argument("--expected-final-decision", default="GO")
    parser.add_argument("--output-json", required=True)
    args = parser.parse_args()

    report_file = Path(args.report_file)
    threshold_file = Path(args.threshold_file)
    ci_tools_script = Path(args.ci_tools_script)
    output_json = Path(args.output_json)
    expected_final_decision = args.expected_final_decision

    if not report_file.is_file():
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_report_file_missing"],
            errors=["report file missing"],
            report_schema_version="unknown",
            runtime_seconds="unknown",
            max_runtime_seconds="unknown",
            allowed_reason_codes_csv="unknown",
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    if not threshold_file.is_file():
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_threshold_file_missing"],
            errors=["threshold file missing"],
            report_schema_version="unknown",
            runtime_seconds="unknown",
            max_runtime_seconds="unknown",
            allowed_reason_codes_csv="unknown",
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    if not ci_tools_script.is_file():
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_ci_tools_script_missing"],
            errors=["ci tools script missing"],
            report_schema_version="unknown",
            runtime_seconds="unknown",
            max_runtime_seconds="unknown",
            allowed_reason_codes_csv="unknown",
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    try:
        thresholds = parse_thresholds(threshold_file)
    except Exception:
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_threshold_value_invalid"],
            errors=["threshold file parsing failed"],
            report_schema_version="unknown",
            runtime_seconds="unknown",
            max_runtime_seconds="unknown",
            allowed_reason_codes_csv="unknown",
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    required_keys = {
        "REPORT_SCHEMA_VERSION",
        "MAX_RUNTIME_SECONDS",
        "ALLOWED_REASON_CODES_CSV",
        "CI_TOOLS_REQUIRED_ENTRY",
        "CI_TOOLS_FORBIDDEN_ENTRY",
    }
    missing_keys = sorted(required_keys - set(thresholds.keys()))
    if missing_keys:
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_threshold_key_missing"],
            errors=[f"missing threshold keys: {','.join(missing_keys)}"],
            report_schema_version="unknown",
            runtime_seconds="unknown",
            max_runtime_seconds="unknown",
            allowed_reason_codes_csv="unknown",
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    report_schema_version = thresholds["REPORT_SCHEMA_VERSION"]
    allowed_reason_codes_csv = thresholds["ALLOWED_REASON_CODES_CSV"]
    allowed_reason_codes = [entry.strip() for entry in allowed_reason_codes_csv.split(",") if entry.strip()]
    if not allowed_reason_codes:
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_threshold_value_invalid"],
            errors=["ALLOWED_REASON_CODES_CSV must contain at least one code"],
            report_schema_version=report_schema_version,
            runtime_seconds="unknown",
            max_runtime_seconds="unknown",
            allowed_reason_codes_csv=allowed_reason_codes_csv,
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    try:
        max_runtime_seconds_value = int(thresholds["MAX_RUNTIME_SECONDS"])
    except Exception:
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_threshold_value_invalid"],
            errors=["MAX_RUNTIME_SECONDS must be an integer"],
            report_schema_version=report_schema_version,
            runtime_seconds="unknown",
            max_runtime_seconds="unknown",
            allowed_reason_codes_csv=allowed_reason_codes_csv,
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    if max_runtime_seconds_value < 0:
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_threshold_value_invalid"],
            errors=["MAX_RUNTIME_SECONDS must be non-negative"],
            report_schema_version=report_schema_version,
            runtime_seconds="unknown",
            max_runtime_seconds=str(max_runtime_seconds_value),
            allowed_reason_codes_csv=allowed_reason_codes_csv,
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    try:
        report_payload = json.loads(report_file.read_text(encoding="utf-8"))
    except Exception:
        return fail(
            output_json=output_json,
            reason_codes=["overload_policy_report_json_invalid"],
            errors=["report JSON is invalid"],
            report_schema_version=report_schema_version,
            runtime_seconds="unknown",
            max_runtime_seconds=str(max_runtime_seconds_value),
            allowed_reason_codes_csv=allowed_reason_codes_csv,
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    errors: List[str] = []
    reason_codes: List[str] = []

    actual_schema = report_payload.get("schema_version")
    if actual_schema != report_schema_version:
        reason_codes.append("overload_policy_report_schema_mismatch")
        errors.append("report schema_version mismatch")

    runtime_seconds = report_payload.get("runtime_seconds")
    if not isinstance(runtime_seconds, int) or runtime_seconds < 0:
        reason_codes.append("overload_policy_report_json_invalid")
        errors.append("runtime_seconds must be a non-negative integer")
        runtime_seconds_rendered = "unknown"
    else:
        runtime_seconds_rendered = str(runtime_seconds)
        if runtime_seconds > max_runtime_seconds_value:
            reason_codes.append("overload_policy_runtime_budget_exceeded")
            errors.append("runtime budget exceeded")

    actual_final_decision = report_payload.get("final_decision")
    if actual_final_decision != expected_final_decision:
        reason_codes.append("overload_policy_expected_decision_mismatch")
        errors.append("final_decision does not match expected value")

    actual_reason_code = report_payload.get("reason_code")
    if not isinstance(actual_reason_code, str) or not actual_reason_code:
        reason_codes.append("overload_policy_report_json_invalid")
        errors.append("reason_code must be a non-empty string")
    elif actual_reason_code not in allowed_reason_codes:
        reason_codes.append("overload_policy_reason_code_unknown")
        errors.append("reason_code is not in ALLOWED_REASON_CODES_CSV")

    ci_tools_content = ci_tools_script.read_text(encoding="utf-8", errors="ignore")
    required_entry = thresholds["CI_TOOLS_REQUIRED_ENTRY"]
    forbidden_entry = thresholds["CI_TOOLS_FORBIDDEN_ENTRY"]
    if required_entry not in ci_tools_content:
        reason_codes.append("overload_policy_ci_tools_fast_mode_missing_overload_test")
        errors.append("ci tools script is missing required overload test entry")
    if f"scripts/ci/{forbidden_entry}" in ci_tools_content:
        reason_codes.append("overload_policy_ci_tools_fast_mode_heavy_run_leaked")
        errors.append("ci tools script leaks direct overload heavy-run command")

    if reason_codes:
        return fail(
            output_json=output_json,
            reason_codes=sorted(dict.fromkeys(reason_codes)),
            errors=errors,
            report_schema_version=report_schema_version,
            runtime_seconds=runtime_seconds_rendered,
            max_runtime_seconds=str(max_runtime_seconds_value),
            allowed_reason_codes_csv=allowed_reason_codes_csv,
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            expected_final_decision=expected_final_decision,
        )

    emit(
        output_json=output_json,
        status="pass",
        final_decision=expected_final_decision,
        reason_codes="none",
        report_schema_version=report_schema_version,
        runtime_seconds=runtime_seconds_rendered,
        max_runtime_seconds=str(max_runtime_seconds_value),
        allowed_reason_codes_csv=allowed_reason_codes_csv,
        report_file=report_file,
        threshold_file=threshold_file,
        ci_tools_script=ci_tools_script,
        expected_final_decision=expected_final_decision,
        errors=None,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

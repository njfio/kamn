#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import stat
import subprocess
import tempfile
from pathlib import Path

MATRIX_REASON_TAXONOMY_VERSION = "kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1"
MATRIX_REASON_CODES_CSV = (
    "runtime_budget_exceeded,matrix_failure_threshold_exceeded,quarantine_registry_missing,"
    "quarantine_reference_present_without_followup,matrix_failures_within_threshold,"
    "stable_success_with_quarantine_followup,stable_success"
)


def require_file(path: Path, message: str) -> None:
    if not path.is_file():
        raise SystemExit(message)


def require_executable(path: Path, message: str) -> None:
    if not path.is_file():
        raise SystemExit(message)
    mode = path.stat().st_mode
    if mode & stat.S_IXUSR == 0:
        raise SystemExit(message)


def write_json(path: Path, payload: dict) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run(cmd: list[str], *, check: bool) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(cmd, text=True, capture_output=True)
    if check and result.returncode != 0:
        raise SystemExit(
            f"command failed ({result.returncode}): {' '.join(cmd)}\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
        )
    return result


def assert_contains(haystack: str, needle: str, message: str) -> None:
    if needle not in haystack:
        raise SystemExit(f"{message}: expected to find `{needle}`")


def matrix_report(
    *,
    reason_code: str,
    runtime_seconds: int,
    final_decision: str = "GO",
    reason_taxonomy_version: str = MATRIX_REASON_TAXONOMY_VERSION,
    reason_codes_csv: str = MATRIX_REASON_CODES_CSV,
) -> dict:
    return {
        "schema_version": "kamn.ci.daemon-os-signal-stress-matrix-report.v1",
        "reason_taxonomy_version": reason_taxonomy_version,
        "reason_codes_csv": reason_codes_csv,
        "status": "pass",
        "final_decision": final_decision,
        "reason_code": reason_code,
        "runtime_seconds": runtime_seconds,
    }


def run_policy(
    *,
    policy_script: Path,
    report_file: Path,
    threshold_file: Path,
    ci_tools_script: Path,
    output_json: Path,
    check: bool,
) -> subprocess.CompletedProcess[str]:
    return run(
        [
            "python3",
            str(policy_script),
            "--report-file",
            str(report_file),
            "--threshold-file",
            str(threshold_file),
            "--ci-tools-script",
            str(ci_tools_script),
            "--expected-final-decision",
            "GO",
            "--output-json",
            str(output_json),
        ],
        check=check,
    )


def assert_reason_failure(
    result: subprocess.CompletedProcess[str], *, expected_reason: str, context: str
) -> None:
    if result.returncode == 0:
        raise SystemExit(f"expected overload dry-run checker to fail on {context}")
    assert_contains(
        result.stdout + result.stderr,
        f"reason_codes={expected_reason}",
        f"expected {context} failure marker",
    )


def main() -> int:
    root_dir = Path(__file__).resolve().parents[2]
    policy_script = root_dir / "scripts/ci/check_daemon_os_signal_stress_policy.py"
    threshold_file = root_dir / "fixtures/ci/daemon_os_signal_stress_policy_thresholds.env"
    ci_tools_script = root_dir / "scripts/ci/test_ci_tools.sh"

    require_executable(policy_script, "expected overload dry-run policy checker to be executable")
    require_file(threshold_file, "expected overload dry-run threshold fixture to exist")

    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)

        report_file = tmp_dir / "daemon-stress-report.json"
        write_json(report_file, matrix_report(reason_code="stable_success", runtime_seconds=42))
        pass_result = run_policy(
            policy_script=policy_script,
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=ci_tools_script,
            output_json=tmp_dir / "policy-pass.json",
            check=True,
        )
        assert_contains(pass_result.stdout, "status=pass", "expected pass status marker")
        assert_contains(pass_result.stdout, "final_decision=GO", "expected GO final decision marker")
        assert_contains(pass_result.stdout, "reason_codes=none", "expected no reason codes marker")

        failure_cases = [
            (
                "runtime budget exceedance",
                matrix_report(reason_code="stable_success", runtime_seconds=601),
                "overload_policy_runtime_budget_exceeded",
            ),
            (
                "unknown reason code",
                matrix_report(reason_code="unexpected_reason_code", runtime_seconds=42),
                "overload_policy_reason_code_unknown",
            ),
            (
                "report reason taxonomy mismatch",
                matrix_report(
                    reason_code="stable_success",
                    runtime_seconds=42,
                    reason_taxonomy_version="kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v999",
                ),
                "overload_policy_report_reason_taxonomy_mismatch",
            ),
            (
                "report reason-csv mismatch",
                matrix_report(
                    reason_code="stable_success",
                    runtime_seconds=42,
                    reason_codes_csv="stable_success",
                ),
                "overload_policy_report_reason_codes_csv_mismatch",
            ),
        ]
        for idx, (context, payload, expected_reason) in enumerate(failure_cases, start=1):
            case_report = tmp_dir / f"daemon-stress-fail-case-{idx}.json"
            write_json(case_report, payload)
            failure_result = run_policy(
                policy_script=policy_script,
                report_file=case_report,
                threshold_file=threshold_file,
                ci_tools_script=ci_tools_script,
                output_json=tmp_dir / f"policy-fail-case-{idx}.json",
                check=False,
            )
            assert_reason_failure(
                failure_result, expected_reason=expected_reason, context=context
            )

        fake_ci_tools = tmp_dir / "test_ci_tools_fast_mode_violation.sh"
        fake_ci_tools.write_text(
            """#!/usr/bin/env bash
set -euo pipefail
if [ "${KAMN_CI_TOOLS_FAST_MODE:-false}" = "true" ]; then
  bash "$ROOT_DIR/scripts/ci/test_run_daemon_os_signal_stress_matrix.sh"
  bash "$ROOT_DIR/scripts/ci/run_daemon_os_signal_stress_matrix.sh"
  exit 0
fi
""",
            encoding="utf-8",
        )
        os.chmod(fake_ci_tools, 0o755)

        selector_fail = run_policy(
            policy_script=policy_script,
            report_file=report_file,
            threshold_file=threshold_file,
            ci_tools_script=fake_ci_tools,
            output_json=tmp_dir / "policy-selector-fail.json",
            check=False,
        )
        assert_reason_failure(
            selector_fail,
            expected_reason="overload_policy_ci_tools_fast_mode_heavy_run_leaked",
            context="heavy run leakage in fast mode",
        )

    print("daemon os-signal overload dry-run policy checker tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

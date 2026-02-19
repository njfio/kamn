#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import stat
import subprocess
import tempfile
from pathlib import Path


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


def main() -> int:
    root_dir = Path(__file__).resolve().parents[2]
    policy_script = root_dir / "scripts/ci/check_daemon_os_signal_stress_policy.py"
    threshold_file = root_dir / "fixtures/ci/daemon_os_signal_stress_policy_thresholds.env"

    require_executable(policy_script, "expected overload dry-run policy checker to be executable")
    require_file(threshold_file, "expected overload dry-run threshold fixture to exist")

    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)

        report_file = tmp_dir / "daemon-stress-report.json"
        write_json(
            report_file,
            {
                "schema_version": "kamn.ci.daemon-os-signal-stress-matrix-report.v1",
                "status": "pass",
                "final_decision": "GO",
                "reason_code": "stable_success",
                "runtime_seconds": 42,
            },
        )

        pass_result = run(
            [
                "python3",
                str(policy_script),
                "--report-file",
                str(report_file),
                "--threshold-file",
                str(threshold_file),
                "--ci-tools-script",
                str(root_dir / "scripts/ci/test_ci_tools.sh"),
                "--expected-final-decision",
                "GO",
                "--output-json",
                str(tmp_dir / "policy-pass.json"),
            ],
            check=True,
        )
        pass_output = pass_result.stdout
        assert_contains(pass_output, "status=pass", "expected pass status marker")
        assert_contains(pass_output, "final_decision=GO", "expected GO final decision marker")
        assert_contains(pass_output, "reason_codes=none", "expected no reason codes marker")

        runtime_fail_report = tmp_dir / "daemon-stress-runtime-fail-report.json"
        write_json(
            runtime_fail_report,
            {
                "schema_version": "kamn.ci.daemon-os-signal-stress-matrix-report.v1",
                "status": "pass",
                "final_decision": "GO",
                "reason_code": "stable_success",
                "runtime_seconds": 601,
            },
        )

        runtime_fail = run(
            [
                "python3",
                str(policy_script),
                "--report-file",
                str(runtime_fail_report),
                "--threshold-file",
                str(threshold_file),
                "--ci-tools-script",
                str(root_dir / "scripts/ci/test_ci_tools.sh"),
                "--expected-final-decision",
                "GO",
                "--output-json",
                str(tmp_dir / "policy-runtime-fail.json"),
            ],
            check=False,
        )
        if runtime_fail.returncode == 0:
            raise SystemExit("expected overload dry-run checker to fail on runtime threshold exceedance")
        assert_contains(
            runtime_fail.stdout + runtime_fail.stderr,
            "reason_codes=overload_policy_runtime_budget_exceeded",
            "expected runtime budget reason code",
        )

        bad_reason_report = tmp_dir / "daemon-stress-bad-reason-report.json"
        write_json(
            bad_reason_report,
            {
                "schema_version": "kamn.ci.daemon-os-signal-stress-matrix-report.v1",
                "status": "pass",
                "final_decision": "GO",
                "reason_code": "unexpected_reason_code",
                "runtime_seconds": 42,
            },
        )

        bad_reason = run(
            [
                "python3",
                str(policy_script),
                "--report-file",
                str(bad_reason_report),
                "--threshold-file",
                str(threshold_file),
                "--ci-tools-script",
                str(root_dir / "scripts/ci/test_ci_tools.sh"),
                "--expected-final-decision",
                "GO",
                "--output-json",
                str(tmp_dir / "policy-reason-fail.json"),
            ],
            check=False,
        )
        if bad_reason.returncode == 0:
            raise SystemExit("expected overload dry-run checker to fail on unknown reason code")
        assert_contains(
            bad_reason.stdout + bad_reason.stderr,
            "reason_codes=overload_policy_reason_code_unknown",
            "expected unknown reason-code failure marker",
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

        selector_fail = run(
            [
                "python3",
                str(policy_script),
                "--report-file",
                str(report_file),
                "--threshold-file",
                str(threshold_file),
                "--ci-tools-script",
                str(fake_ci_tools),
                "--expected-final-decision",
                "GO",
                "--output-json",
                str(tmp_dir / "policy-selector-fail.json"),
            ],
            check=False,
        )
        if selector_fail.returncode == 0:
            raise SystemExit(
                "expected overload dry-run checker to fail when heavy run leaks into fast mode"
            )
        assert_contains(
            selector_fail.stdout + selector_fail.stderr,
            "reason_codes=overload_policy_ci_tools_fast_mode_heavy_run_leaked",
            "expected heavy-run leakage marker",
        )

    print("daemon os-signal overload dry-run policy checker tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

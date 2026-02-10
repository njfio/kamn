#!/usr/bin/env python3
"""Frontend shell determinism matrix contract-lane runner."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def usage() -> None:
    """Print CLI usage."""
    print(
        "Usage:\n"
        "  bash scripts/frontend/run_dashboard_shell_determinism_matrix_contract_lane.sh \\\n"
        "    [--output-file <path>]"
    )


def fail(message: str) -> int:
    """Emit an error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def parse_args(argv: list[str]) -> tuple[int, str | None]:
    """Parse CLI arguments and return exit code/output file."""
    output_file: str | None = None
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--output-file":
            if index + 1 >= len(argv):
                return fail("unknown argument: --output-file"), None
            output_file = argv[index + 1]
            index += 2
            continue
        if argument in {"--help", "-h"}:
            usage()
            return 0, None
        return fail(f"unknown argument: {argument}"), None

    return 200, output_file


def run_and_capture(command: list[str], *, env: dict[str, str] | None = None) -> tuple[int, str]:
    """Run command and capture merged stdout/stderr."""
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def main(argv: list[str]) -> int:
    """Execute frontend shell determinism matrix contract lane."""
    parse_status, output_file_arg = parse_args(argv)
    if parse_status != 200:
        return parse_status

    root_dir = Path(__file__).resolve().parents[2]
    lane_script = root_dir / "scripts/frontend/run_dashboard_shell_determinism_matrix_lane.sh"
    checker = root_dir / "scripts/frontend/check_dashboard_shell_determinism_matrix_policy.sh"
    ui_doc = root_dir / "docs/foundation/operator-dashboard-ui-mvp.md"

    if not lane_script.is_file() or not os.access(lane_script, os.X_OK):
        return fail("expected dashboard shell matrix lane script to be executable")

    if not checker.is_file() or not os.access(checker, os.X_OK):
        return fail("expected dashboard shell matrix policy checker to be executable")

    if not ui_doc.is_file():
        return fail("expected operator dashboard UI doc to exist")

    max_contract_seconds_raw = os.getenv("KAMN_FRONTEND_SHELL_MATRIX_CONTRACT_MAX_SECONDS", "240")
    if not max_contract_seconds_raw.isdigit() or int(max_contract_seconds_raw) <= 0:
        return fail("KAMN_FRONTEND_SHELL_MATRIX_CONTRACT_MAX_SECONDS must be a positive integer")
    max_contract_seconds = int(max_contract_seconds_raw)

    start_epoch = int(time.time())

    with tempfile.TemporaryDirectory() as temp_dir:
        if output_file_arg:
            output_file = Path(output_file_arg)
        else:
            output_file = Path(temp_dir) / "dashboard-shell-matrix-contract-report.json"
        stale_critical_no_go_report = Path(temp_dir) / "dashboard-shell-matrix-no-go.json"

        env_go = os.environ.copy()
        env_go["KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS"] = str(max_contract_seconds)
        go_code, go_output = run_and_capture(
            ["bash", str(lane_script), "--output-json", str(output_file)],
            env=env_go,
        )
        if go_code != 0:
            return fail("expected dashboard shell matrix lane GO run to report pass status")

        if "status=pass" not in go_output:
            return fail("expected dashboard shell matrix lane GO run to report pass status")
        if "final_decision=GO" not in go_output:
            return fail("expected dashboard shell matrix lane GO run to report GO decision")
        if "reason_key=frontend_shell_matrix_reason_codes:GO:v1" not in go_output:
            return fail("expected dashboard shell matrix lane GO run to emit deterministic GO reason key")

        go_policy_code, go_policy_output = run_and_capture(
            ["bash", str(checker), "--report-file", str(output_file)]
        )
        if go_policy_code != 0:
            return fail("expected dashboard shell matrix policy checker status marker for GO report")
        if "status=ok" not in go_policy_output:
            return fail("expected dashboard shell matrix policy checker status marker for GO report")
        if "final_decision=GO" not in go_policy_output:
            return fail("expected dashboard shell matrix policy checker GO decision for GO report")
        if "failed_checks=none" not in go_policy_output:
            return fail("expected dashboard shell matrix policy checker no failed checks for GO report")

        env_no_go = os.environ.copy()
        env_no_go["KAMN_FRONTEND_SHELL_MATRIX_SKIP_COMMANDS"] = "true"
        env_no_go["KAMN_FRONTEND_SHELL_MATRIX_FORCE_STALE_CRITICAL_STATE_MISSING"] = "true"
        stale_no_go_code, stale_no_go_output = run_and_capture(
            ["bash", str(lane_script), "--output-json", str(stale_critical_no_go_report)],
            env=env_no_go,
        )

        if stale_no_go_code == 0:
            return fail("expected forced stale/critical missing dashboard shell matrix lane run to fail closed")
        if "stale_critical_state_missing" not in stale_no_go_output:
            return fail("expected forced stale/critical missing lane run to emit stale_critical_state_missing reason code")

        stale_policy_code, stale_policy_output = run_and_capture(
            ["bash", str(checker), "--report-file", str(stale_critical_no_go_report)]
        )
        if stale_policy_code != 0:
            return fail(
                "expected dashboard shell matrix policy checker NO-GO decision for stale/critical-missing report"
            )
        if "final_decision=NO-GO" not in stale_policy_output:
            return fail(
                "expected dashboard shell matrix policy checker NO-GO decision for stale/critical-missing report"
            )
        if "stale_critical_state_missing" not in stale_policy_output:
            return fail(
                "expected dashboard shell matrix policy checker failed checks to include stale_critical_state_missing"
            )

        doc_text = ui_doc.read_text(encoding="utf-8")
        required_doc_snippets = (
            "run_dashboard_shell_determinism_matrix_lane.sh",
            "dashboard_shell_determinism_matrix_lane_contract.py",
            "check_dashboard_shell_determinism_matrix_policy.sh",
            "dashboard_shell_determinism_matrix_policy_contract.py",
            "run_dashboard_shell_determinism_matrix_contract_lane.sh",
            "dashboard_shell_determinism_matrix_contract_lane_contract.py",
            "kamn.frontend.shell-matrix-report.v1",
            "Regression: #943",
            "Regression: #1210",
            "Regression: #1214",
            "Regression: #1218",
        )
        for snippet in required_doc_snippets:
            if snippet not in doc_text:
                if snippet == "run_dashboard_shell_determinism_matrix_lane.sh":
                    return fail(
                        "expected operator dashboard UI doc to reference dashboard shell matrix lane command"
                    )
                if snippet == "check_dashboard_shell_determinism_matrix_policy.sh":
                    return fail(
                        "expected operator dashboard UI doc to reference dashboard shell matrix policy checker command"
                    )
                if snippet == "run_dashboard_shell_determinism_matrix_contract_lane.sh":
                    return fail(
                        "expected operator dashboard UI doc to reference dashboard shell matrix contract lane command"
                    )
                if snippet == "kamn.frontend.shell-matrix-report.v1":
                    return fail(
                        "expected operator dashboard UI doc to reference dashboard shell matrix schema marker"
                    )
                return fail(
                    f"expected operator dashboard UI doc to reference {snippet}"
                )

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_contract_seconds:
            return fail(
                f"dashboard shell determinism matrix contract lane exceeded runtime budget: {elapsed_seconds}s"
            )

        print("status=ok")
        print(f"report_file={output_file}")
        print("final_decision=GO")
        print("dashboard shell determinism matrix contract lane tests passed.")
        return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

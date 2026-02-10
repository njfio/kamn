#!/usr/bin/env python3
"""Frontend shell determinism matrix lane contract runner."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import time


def usage() -> None:
    """Print CLI usage."""
    print(
        "Usage:\n"
        "  bash scripts/frontend/run_dashboard_shell_determinism_matrix_lane.sh \\\n"
        "    --output-json <path>"
    )


def fail(message: str) -> int:
    """Emit an error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def parse_args(argv: list[str]) -> tuple[int, str | None]:
    """Parse CLI arguments and return exit code/output path."""
    output_json: str | None = None
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--output-json":
            if index + 1 >= len(argv):
                return fail("unknown argument: --output-json"), None
            output_json = argv[index + 1]
            index += 2
            continue
        if argument in {"--help", "-h"}:
            usage()
            return 0, None
        return fail(f"unknown argument: {argument}"), None

    if not output_json:
        usage()
        return fail("--output-json is required"), None

    return 200, output_json


def main(argv: list[str]) -> int:
    """Execute lane checks and emit report JSON."""
    parse_status, output_json = parse_args(argv)
    if parse_status != 200:
        return parse_status

    root_dir = Path(__file__).resolve().parents[2]
    dashboard_test_script = root_dir / "scripts/frontend/test_dashboard_package.sh"
    ui_doc = root_dir / "docs/foundation/operator-dashboard-ui-mvp.md"

    if not dashboard_test_script.is_file() or not os.access(dashboard_test_script, os.X_OK):
        return fail("expected dashboard package test script to be executable")

    if not ui_doc.is_file():
        return fail("expected operator dashboard UI doc to exist")

    max_seconds_raw = os.getenv("KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS", "180")
    skip_commands = os.getenv("KAMN_FRONTEND_SHELL_MATRIX_SKIP_COMMANDS", "false")
    force_healthy_state_missing = os.getenv(
        "KAMN_FRONTEND_SHELL_MATRIX_FORCE_HEALTHY_STATE_MISSING", "false"
    )
    force_stale_critical_state_missing = os.getenv(
        "KAMN_FRONTEND_SHELL_MATRIX_FORCE_STALE_CRITICAL_STATE_MISSING", "false"
    )
    force_error_state_missing = os.getenv(
        "KAMN_FRONTEND_SHELL_MATRIX_FORCE_ERROR_STATE_MISSING", "false"
    )
    force_docs_contract_missing = os.getenv(
        "KAMN_FRONTEND_SHELL_MATRIX_FORCE_DOCS_CONTRACT_MISSING", "false"
    )
    force_lane_failure = os.getenv("KAMN_FRONTEND_SHELL_MATRIX_FORCE_LANE_FAILURE", "false")

    if not max_seconds_raw.isdigit():
        return fail("KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS must be a non-negative integer")
    max_seconds = int(max_seconds_raw)

    for name, value in (
        ("skip_commands", skip_commands),
        ("force_healthy_state_missing", force_healthy_state_missing),
        ("force_stale_critical_state_missing", force_stale_critical_state_missing),
        ("force_error_state_missing", force_error_state_missing),
        ("force_docs_contract_missing", force_docs_contract_missing),
        ("force_lane_failure", force_lane_failure),
    ):
        if value not in {"true", "false"}:
            return fail(f"invalid boolean for {name}: {value}")

    output_path = Path(output_json)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    start_epoch = int(time.time())

    commands: list[str] = []
    dashboard_output = ""
    dashboard_exit_code = 0
    frontend_lane_passed = True

    if force_lane_failure == "true":
        frontend_lane_passed = False
        dashboard_exit_code = 1
    elif skip_commands != "true":
        commands.append("bash scripts/frontend/test_dashboard_package.sh")
        result = subprocess.run(
            ["bash", str(dashboard_test_script)],
            capture_output=True,
            text=True,
            check=False,
        )
        dashboard_output = (result.stdout or "") + (result.stderr or "")
        dashboard_exit_code = result.returncode
        if dashboard_exit_code != 0:
            frontend_lane_passed = False

    healthy_state_passed = True
    stale_critical_state_passed = True
    error_state_passed = True

    if skip_commands != "true":
        if not frontend_lane_passed:
            healthy_state_passed = False
            stale_critical_state_passed = False
            error_state_passed = False
        else:
            if (
                "functional builds dashboard shell from live backend snapshot"
                not in dashboard_output
            ):
                healthy_state_passed = False
            if (
                "regression renders critical badge and stale banner together"
                not in dashboard_output
            ):
                stale_critical_state_passed = False
            if "integration renders explicit error state shell" not in dashboard_output:
                error_state_passed = False
            if (
                "regression renders error shell when live backend request fails"
                not in dashboard_output
            ):
                error_state_passed = False

    if force_healthy_state_missing == "true":
        healthy_state_passed = False
    if force_stale_critical_state_missing == "true":
        stale_critical_state_passed = False
    if force_error_state_missing == "true":
        error_state_passed = False

    docs_contract_passed = True
    required_doc_snippets = (
        "## Frontend Shell Determinism Matrix Contract",
        "run_dashboard_shell_determinism_matrix_lane.sh",
        "dashboard_shell_determinism_matrix_lane_contract.py",
        "check_dashboard_shell_determinism_matrix_policy.sh",
        "dashboard_shell_determinism_matrix_policy_contract.py",
        "run_dashboard_shell_determinism_matrix_contract_lane.sh",
        "kamn.frontend.shell-matrix-report.v1",
        "KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS",
        "KAMN_FRONTEND_SHELL_MATRIX_CONTRACT_MAX_SECONDS",
        "Regression: #943",
        "Regression: #1210",
        "Regression: #1214",
    )

    doc_text = ui_doc.read_text(encoding="utf-8")
    for snippet in required_doc_snippets:
        if snippet not in doc_text:
            docs_contract_passed = False
            break

    if force_docs_contract_missing == "true":
        docs_contract_passed = False

    elapsed_seconds = int(time.time()) - start_epoch

    reason_codes: list[str] = []
    if not frontend_lane_passed:
        reason_codes.append("frontend_lane_failed")
    if not healthy_state_passed:
        reason_codes.append("healthy_state_missing")
    if not stale_critical_state_passed:
        reason_codes.append("stale_critical_state_missing")
    if not error_state_passed:
        reason_codes.append("error_state_missing")
    if not docs_contract_passed:
        reason_codes.append("docs_contract_missing")
    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")

    if reason_codes:
        reason_codes = sorted(set(reason_codes))

    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"

    reason_key = f"frontend_shell_matrix_reason_codes:{final_decision}:v1"
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

    payload = {
        "schema_version": "kamn.frontend.shell-matrix-report.v1",
        "evidence_key": "frontend_shell_matrix:v1",
        "status": status,
        "final_decision": final_decision,
        "reason_key": reason_key,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "skip_commands": skip_commands == "true",
        "dashboard_package_exit_code": dashboard_exit_code,
        "command_count": len(commands),
        "commands": commands,
        "frontend_lane_passed": frontend_lane_passed,
        "healthy_state_passed": healthy_state_passed,
        "stale_critical_state_passed": stale_critical_state_passed,
        "error_state_passed": error_state_passed,
        "docs_contract_passed": docs_contract_passed,
        "reason_codes": [] if reason_codes_csv == "none" else reason_codes_csv.split(","),
    }

    output_path.write_text(
        json.dumps(payload, sort_keys=True, indent=2) + "\n",
        encoding="utf-8",
    )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"reason_codes={reason_codes_csv}")
    print(f"reason_key={reason_key}")
    print(f"report_file={output_path}")

    if status != "pass":
        return fail(f"dashboard shell determinism matrix lane failed closed: {reason_codes_csv}")

    print("dashboard shell determinism matrix lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

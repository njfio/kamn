#!/usr/bin/env python3
"""Dashboard stale/error budget contract-lane runner."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def usage() -> None:
    """Print usage text."""
    print(
        "Usage:\n"
        "  bash scripts/dashboard/run_dashboard_stale_error_budget_contract_lane.sh \\\n"
        "    [--output-file <path>]"
    )


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def run_capture(command: list[str], *, env: dict[str, str] | None = None) -> tuple[int, str]:
    """Run command and return exit code + merged output."""
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def parse_args(argv: list[str]) -> tuple[int, str]:
    """Parse CLI args and return status/output-file."""
    output_file = ""
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--output-file":
            if index + 1 >= len(argv):
                return fail("unknown argument: --output-file"), output_file
            output_file = argv[index + 1]
            index += 2
            continue
        if argument in {"--help", "-h"}:
            usage()
            return 0, output_file
        return fail(f"unknown argument: {argument}"), output_file
    return 200, output_file


def main(argv: list[str]) -> int:
    """Execute dashboard stale/error budget contract lane."""
    parse_status, output_file_arg = parse_args(argv)
    if parse_status != 200:
        return parse_status

    root_dir = Path(__file__).resolve().parents[2]
    lane_script = root_dir / "scripts/dashboard/run_dashboard_stale_error_budget_lane.sh"
    checker = root_dir / "scripts/dashboard/check_dashboard_stale_error_budget_policy.sh"
    observability_doc = root_dir / "docs/foundation/observability-slo-dashboards.md"

    if not lane_script.is_file() or not os.access(lane_script, os.X_OK):
        return fail("expected dashboard stale/error budget lane script to be executable")
    if not checker.is_file() or not os.access(checker, os.X_OK):
        return fail("expected dashboard stale/error budget policy checker to be executable")
    if not observability_doc.is_file():
        return fail("expected observability SLO dashboard doc to exist")

    max_contract_seconds_raw = os.getenv("KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS", "240")
    if not max_contract_seconds_raw.isdigit() or int(max_contract_seconds_raw) <= 0:
        return fail("KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS must be a positive integer")
    max_contract_seconds = int(max_contract_seconds_raw)
    require_frontend_contract_raw = os.getenv(
        "KAMN_DASHBOARD_STALE_ERROR_CONTRACT_REQUIRE_FRONTEND", "false"
    )
    if require_frontend_contract_raw not in {"true", "false"}:
        return fail("KAMN_DASHBOARD_STALE_ERROR_CONTRACT_REQUIRE_FRONTEND must be true or false")
    require_frontend_contract = require_frontend_contract_raw == "true"
    start_epoch = int(time.time())

    with tempfile.TemporaryDirectory() as temp_dir:
        if output_file_arg:
            output_file = Path(output_file_arg)
        else:
            output_file = Path(temp_dir) / "dashboard-stale-error-contract-report.json"

        go_env = os.environ.copy()
        go_env["KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS"] = str(max_contract_seconds)
        if not require_frontend_contract:
            # Keep CI contract-lane GO path deterministic and low-cost by default.
            go_env["KAMN_DASHBOARD_STALE_ERROR_SKIP_COMMANDS"] = "true"
        go_code, go_output = run_capture(
            ["bash", str(lane_script), "--output-json", str(output_file)],
            env=go_env,
        )
        if go_code != 0 or "status=pass" not in go_output:
            return fail("expected dashboard stale/error budget lane GO run to report pass status")
        if "final_decision=GO" not in go_output:
            return fail("expected dashboard stale/error budget lane GO run to report GO decision")
        if "reason_key=dashboard_stale_error_budget_reason_codes:GO:v1" not in go_output:
            return fail("expected dashboard stale/error budget lane GO run to emit deterministic GO reason key")

        go_policy_code, go_policy_output = run_capture(
            ["bash", str(checker), "--report-file", str(output_file)]
        )
        if go_policy_code != 0 or "status=ok" not in go_policy_output:
            return fail("expected dashboard stale/error budget policy checker status marker for GO report")
        if "final_decision=GO" not in go_policy_output:
            return fail("expected dashboard stale/error budget policy checker GO decision for GO report")
        if "failed_checks=none" not in go_policy_output:
            return fail("expected dashboard stale/error budget policy checker no failed checks for GO report")

        stale_no_go_report = Path(temp_dir) / "dashboard-stale-error-no-go.json"
        no_go_env = os.environ.copy()
        no_go_env["KAMN_DASHBOARD_STALE_ERROR_SKIP_COMMANDS"] = "true"
        no_go_env["KAMN_DASHBOARD_STALE_ERROR_FORCE_STALE_DATA_MISSING"] = "true"
        no_go_code, no_go_output = run_capture(
            ["bash", str(lane_script), "--output-json", str(stale_no_go_report)],
            env=no_go_env,
        )
        if no_go_code == 0:
            return fail("expected forced stale-data-missing dashboard stale/error lane run to fail closed")
        if "stale_data_threshold_missing" not in no_go_output:
            return fail("expected forced stale-data-missing lane run to emit stale_data_threshold_missing reason code")

        no_go_policy_code, no_go_policy_output = run_capture(
            ["bash", str(checker), "--report-file", str(stale_no_go_report)]
        )
        if no_go_policy_code != 0 or "final_decision=NO-GO" not in no_go_policy_output:
            return fail(
                "expected dashboard stale/error budget policy checker NO-GO decision for stale-data-missing report"
            )
        if "stale_data_threshold_missing" not in no_go_policy_output:
            return fail(
                "expected dashboard stale/error budget policy checker failed checks to include stale_data_threshold_missing"
            )

        doc_text = observability_doc.read_text(encoding="utf-8")
        required_doc_snippets = (
            "run_dashboard_stale_error_budget_lane.sh",
            "check_dashboard_stale_error_budget_policy.sh",
            "run_dashboard_stale_error_budget_contract_lane.sh",
            "stale_error_budget_contract_lane_contract.py",
            "kamn.dashboard.stale-error-budget-report.v1",
            "Regression: #942",
            "Regression: #1258",
        )
        for snippet in required_doc_snippets:
            if snippet not in doc_text:
                if snippet == "run_dashboard_stale_error_budget_lane.sh":
                    return fail("expected observability doc to reference dashboard stale/error lane command")
                if snippet == "check_dashboard_stale_error_budget_policy.sh":
                    return fail("expected observability doc to reference dashboard stale/error policy checker command")
                if snippet == "run_dashboard_stale_error_budget_contract_lane.sh":
                    return fail("expected observability doc to reference dashboard stale/error contract lane command")
                if snippet == "kamn.dashboard.stale-error-budget-report.v1":
                    return fail("expected observability doc to reference dashboard stale/error schema marker")
                if snippet == "Regression: #942":
                    return fail("expected observability doc to include Regression: #942 marker")
                return fail("expected observability doc to include Regression: #1258 marker")

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_contract_seconds:
            return fail(
                f"dashboard stale/error budget contract lane exceeded runtime budget: {elapsed_seconds}s"
            )

        print("status=ok")
        print(f"report_file={output_file}")
        print("final_decision=GO")
        print("dashboard stale/error budget contract lane tests passed.")
        return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

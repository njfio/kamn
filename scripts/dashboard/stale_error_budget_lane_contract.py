#!/usr/bin/env python3
"""Dashboard stale/error budget lane runner and report emitter."""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402

ROOT_DIR = SCRIPT_DIR.parent.parent
GENERATOR = ROOT_DIR / "scripts/canary/generate_post_cutover_slo_evidence_bundle.sh"
POLICY_CHECKER = ROOT_DIR / "scripts/canary/check_post_cutover_slo_policy.sh"
OBSERVABILITY_DOC = ROOT_DIR / "docs/foundation/observability-slo-dashboards.md"


def usage() -> None:
    print(
        "Usage:\n"
        "  bash scripts/dashboard/run_dashboard_stale_error_budget_lane.sh \\\n"
        "    --output-json <path>"
    )


def _parse_args(argv: list[str]) -> Path:
    output_json = ""
    index = 0
    while index < len(argv):
        arg = argv[index]
        if arg == "--output-json":
            if index + 1 >= len(argv):
                fail("unknown argument: --output-json")
            output_json = argv[index + 1]
            index += 2
            continue
        if arg in {"--help", "-h"}:
            usage()
            raise SystemExit(0)
        fail(f"unknown argument: {arg}")

    if output_json == "":
        usage()
        fail("--output-json is required")
    return Path(output_json)


def _parse_bool_env(name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"invalid boolean for {name}: {raw_value}")


def _run_command(command: list[str]) -> tuple[int, str]:
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    output = completed.stdout
    if completed.stderr:
        output = f"{output}{completed.stderr}"
    return completed.returncode, output


def _extract_value(output: str, key: str) -> str:
    for line in output.splitlines():
        if "=" not in line:
            continue
        candidate_key, value = line.split("=", 1)
        if candidate_key == key:
            return value
    return ""


def main(argv: list[str]) -> int:
    output_json = _parse_args(argv)

    if not GENERATOR.is_file() or not os.access(GENERATOR, os.X_OK):
        fail("expected post-cutover SLO evidence generator to be executable")
    if not POLICY_CHECKER.is_file() or not os.access(POLICY_CHECKER, os.X_OK):
        fail("expected post-cutover SLO policy checker to be executable")
    if not OBSERVABILITY_DOC.is_file():
        fail("expected observability SLO dashboard doc to exist")

    max_seconds_raw = os.getenv("KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS", "180")
    if re.fullmatch(r"[0-9]+", max_seconds_raw) is None:
        fail("KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS must be a non-negative integer")
    max_seconds = int(max_seconds_raw)

    skip_commands = _parse_bool_env(
        "skip_commands",
        os.getenv("KAMN_DASHBOARD_STALE_ERROR_SKIP_COMMANDS", "false"),
    )
    force_stale_data_missing = _parse_bool_env(
        "force_stale_data_missing",
        os.getenv("KAMN_DASHBOARD_STALE_ERROR_FORCE_STALE_DATA_MISSING", "false"),
    )
    force_error_budget_missing = _parse_bool_env(
        "force_error_budget_missing",
        os.getenv("KAMN_DASHBOARD_STALE_ERROR_FORCE_ERROR_BUDGET_MISSING", "false"),
    )
    force_docs_contract_missing = _parse_bool_env(
        "force_docs_contract_missing",
        os.getenv("KAMN_DASHBOARD_STALE_ERROR_FORCE_DOCS_CONTRACT_MISSING", "false"),
    )
    force_lane_failure = _parse_bool_env(
        "force_lane_failure",
        os.getenv("KAMN_DASHBOARD_STALE_ERROR_FORCE_LANE_FAILURE", "false"),
    )

    output_json.parent.mkdir(parents=True, exist_ok=True)
    start_epoch = int(time.time())

    commands: list[str] = []
    generator_output = ""
    policy_output = ""
    generator_exit_code = 0
    policy_exit_code = 0
    dashboard_lane_passed = True

    with tempfile.TemporaryDirectory() as temp_dir:
        bundle_file = Path(temp_dir) / "dashboard-stale-error-evidence.json"

        if force_lane_failure:
            dashboard_lane_passed = False
            generator_exit_code = 1
            policy_exit_code = 1
        elif not skip_commands:
            commands.append(
                "bash scripts/canary/generate_post_cutover_slo_evidence_bundle.sh "
                f"--output-file {bundle_file} "
                "--window-minutes 15 "
                "--p95-latency-ms 140 "
                "--max-p95-latency-ms 200 "
                "--error-rate-bps 18 "
                "--max-error-rate-bps 25 "
                "--delivery-success-bps 9992 "
                "--min-delivery-success-bps 9950 "
                "--snapshot-age-seconds 30 "
                "--max-snapshot-age-seconds 120 "
                "--evidence-complete true "
                "--ci-fast-gate PASS"
            )
            generator_exit_code, generator_output = _run_command(
                [
                    "bash",
                    str(GENERATOR),
                    "--output-file",
                    str(bundle_file),
                    "--window-minutes",
                    "15",
                    "--p95-latency-ms",
                    "140",
                    "--max-p95-latency-ms",
                    "200",
                    "--error-rate-bps",
                    "18",
                    "--max-error-rate-bps",
                    "25",
                    "--delivery-success-bps",
                    "9992",
                    "--min-delivery-success-bps",
                    "9950",
                    "--snapshot-age-seconds",
                    "30",
                    "--max-snapshot-age-seconds",
                    "120",
                    "--evidence-complete",
                    "true",
                    "--ci-fast-gate",
                    "PASS",
                ]
            )

            if generator_exit_code == 0:
                commands.append(
                    "bash scripts/canary/check_post_cutover_slo_policy.sh "
                    f"--bundle-file {bundle_file}"
                )
                policy_exit_code, policy_output = _run_command(
                    ["bash", str(POLICY_CHECKER), "--bundle-file", str(bundle_file)]
                )
            else:
                policy_exit_code = 1

            if generator_exit_code != 0 or policy_exit_code != 0:
                dashboard_lane_passed = False

        canary_bundle_final_decision = "unknown"
        if not skip_commands and generator_output:
            maybe_bundle_decision = _extract_value(generator_output, "final_decision")
            if maybe_bundle_decision:
                canary_bundle_final_decision = maybe_bundle_decision

        canary_policy_final_decision = "unknown"
        if not skip_commands and policy_output:
            maybe_policy_decision = _extract_value(policy_output, "final_decision")
            if maybe_policy_decision:
                canary_policy_final_decision = maybe_policy_decision

        stale_data_passed = True
        error_budget_passed = True
        if not skip_commands:
            if not dashboard_lane_passed:
                stale_data_passed = False
                error_budget_passed = False
            else:
                try:
                    payload = json.loads(bundle_file.read_text(encoding="utf-8"))
                    metrics = payload.get("metrics", {})
                    if metrics.get("snapshot_age_seconds", 10**9) > metrics.get(
                        "max_snapshot_age_seconds",
                        -1,
                    ):
                        stale_data_passed = False
                    if metrics.get("error_rate_bps", 10**9) > metrics.get(
                        "max_error_rate_bps",
                        -1,
                    ):
                        error_budget_passed = False
                except (OSError, json.JSONDecodeError):
                    stale_data_passed = False
                    error_budget_passed = False

                if canary_policy_final_decision != "GO":
                    if stale_data_passed and error_budget_passed:
                        stale_data_passed = False

        if force_stale_data_missing:
            stale_data_passed = False
        if force_error_budget_missing:
            error_budget_passed = False

        docs_contract_passed = True
        required_doc_snippets = (
            "## Dashboard Stale/Error Budget Policy Checker Contract",
            "stale_error_budget_policy_contract.py",
            "stale_error_budget_lane_contract.py",
            "run_dashboard_stale_error_budget_lane.sh",
            "check_dashboard_stale_error_budget_policy.sh",
            "run_dashboard_stale_error_budget_contract_lane.sh",
            "kamn.dashboard.stale-error-budget-report.v1",
            "KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS",
            "KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS",
            "Regression: #942",
        )
        observability_doc_text = OBSERVABILITY_DOC.read_text(encoding="utf-8")
        for snippet in required_doc_snippets:
            if snippet not in observability_doc_text:
                docs_contract_passed = False
                break
        if force_docs_contract_missing:
            docs_contract_passed = False

        elapsed_seconds = int(time.time()) - start_epoch

        reason_codes: list[str] = []
        if not dashboard_lane_passed:
            reason_codes.append("dashboard_lane_failed")
        if not stale_data_passed:
            reason_codes.append("stale_data_threshold_missing")
        if not error_budget_passed:
            reason_codes.append("error_budget_threshold_missing")
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
        reason_key = f"dashboard_stale_error_budget_reason_codes:{final_decision}:v1"
        reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

        payload: dict[str, Any] = {
            "schema_version": "kamn.dashboard.stale-error-budget-report.v1",
            "evidence_key": "dashboard_stale_error_budget:v1",
            "status": status,
            "final_decision": final_decision,
            "reason_key": reason_key,
            "elapsed_seconds": elapsed_seconds,
            "max_seconds": max_seconds,
            "skip_commands": skip_commands,
            "bundle_file": str(bundle_file),
            "generator_exit_code": generator_exit_code,
            "policy_exit_code": policy_exit_code,
            "canary_bundle_final_decision": canary_bundle_final_decision,
            "canary_policy_final_decision": canary_policy_final_decision,
            "dashboard_lane_passed": dashboard_lane_passed,
            "stale_data_passed": stale_data_passed,
            "error_budget_passed": error_budget_passed,
            "docs_contract_passed": docs_contract_passed,
            "command_count": len(commands),
            "commands": commands,
            "reason_codes": reason_codes,
        }
        write_json(output_json, payload)

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"reason_codes={reason_codes_csv}")
    print(f"reason_key={reason_key}")
    print(f"report_file={output_json}")

    if status != "pass":
        fail(f"dashboard stale/error budget lane failed closed: {reason_codes_csv}")

    print("dashboard stale/error budget lane tests passed.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

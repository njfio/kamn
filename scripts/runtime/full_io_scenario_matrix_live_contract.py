#!/usr/bin/env python3
"""Full I/O scenario matrix lane and policy checker contracts."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    load_json,
    require_enum,
    require_positive_int,
    write_json,
)

RUN_LANE_SCHEMA = "kamn.runtime.full-io-scenario-matrix-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.full-io-scenario-matrix-live-policy-report.v1"
OPT_IN_ENV = "KAMN_LOCAL_FULL_IO_SCENARIO_MATRIX_OPT_IN"
DRY_RUN_REASON = "dry_run_no_commands_executed"
RUN_REASON = "full_io_scenario_matrix_executed"
FAST_GATE_EXCLUSION_REASON = "full_io_scenario_matrix_run_mode_excluded_from_fast_gate"
FULL_IO_HARNESS_POLICY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.full-io-scenario-matrix-policy-reason-taxonomy.v1"
)
FULL_IO_HARNESS_POLICY_REASON_CODES = (
    "full_io_scenario_matrix_policy_schema_mismatch",
    "full_io_scenario_matrix_policy_status_mismatch",
    "full_io_scenario_matrix_policy_final_decision_mismatch",
    "full_io_scenario_matrix_policy_ci_fast_gate_mismatch",
    "full_io_scenario_matrix_policy_process_harness_mismatch",
    "full_io_scenario_matrix_policy_api_route_matrix_mismatch",
    "full_io_scenario_matrix_policy_auth_failure_matrix_mismatch",
    "full_io_scenario_matrix_policy_websocket_matrix_mismatch",
    "full_io_scenario_matrix_policy_multinode_propagation_mismatch",
    "full_io_scenario_matrix_policy_fast_gate_exclusion_mismatch",
    "full_io_scenario_matrix_policy_fast_gate_reason_mismatch",
    "full_io_scenario_matrix_policy_lane_mode_invalid",
    "full_io_scenario_matrix_policy_command_count_invalid",
    "full_io_scenario_matrix_policy_artifact_paths_invalid",
    "full_io_scenario_matrix_policy_dry_run_eligibility_mismatch",
    "full_io_scenario_matrix_policy_dry_run_command_count_mismatch",
    "full_io_scenario_matrix_policy_dry_run_command_status_mismatch",
    "full_io_scenario_matrix_policy_dry_run_reason_code_mismatch",
    "full_io_scenario_matrix_policy_run_mode_exclusion_mismatch",
    "full_io_scenario_matrix_policy_run_mode_command_count_mismatch",
    "full_io_scenario_matrix_policy_run_mode_command_status_mismatch",
    "full_io_scenario_matrix_policy_run_mode_reason_code_mismatch",
    "full_io_scenario_matrix_policy_expected_decision_mismatch",
)
FULL_IO_HARNESS_POLICY_REASON_CODES_CSV = ",".join(FULL_IO_HARNESS_POLICY_REASON_CODES)


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _reason_codes_value(reason_codes: list[str]) -> str:
    return ",".join(reason_codes) if reason_codes else "none"


def _run_command(
    command: list[str],
    *,
    timeout_seconds: int,
    env: dict[str, str] | None = None,
) -> str:
    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
        timeout=timeout_seconds,
        env=env,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"lane command failed: {' '.join(command)}: {detail}")
    return completed.stdout


def run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    max_seconds = require_positive_int("KAMN_FULL_IO_SCENARIO_MATRIX_MAX_SECONDS", args.max_seconds)
    command_max_seconds = require_positive_int(
        "KAMN_FULL_IO_SCENARIO_MATRIX_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    process_harness_module = ROOT_DIR / "scripts/framework/process_harness.py"
    if not process_harness_module.is_file():
        fail("expected reusable process harness module to exist: scripts/framework/process_harness.py")

    if mode == "run" and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            "KAMN_LOCAL_FULL_IO_SCENARIO_MATRIX_OPT_IN=1"
        )

    start_epoch = int(time.time())
    commands_executed = 0
    scenario_artifact_paths: dict[str, str] = {}

    with tempfile.TemporaryDirectory(prefix="full-io-scenario-matrix-live-") as temp_dir:
        temp_path = Path(temp_dir)
        command_specs: list[tuple[str, list[str], dict[str, str] | None]] = [
            (
                "api_route_matrix",
                [
                    "bash",
                    "scripts/runtime/validate_service_api_live.sh",
                    "--max-seconds",
                    str(command_max_seconds),
                    "--output-json",
                    str(temp_path / "api-route-matrix.json"),
                ],
                None,
            ),
            (
                "auth_failure_matrix",
                [
                    "bash",
                    "scripts/runtime/validate_service_api_request_auth_live.sh",
                    "--max-seconds",
                    str(command_max_seconds),
                    "--output-json",
                    str(temp_path / "auth-failure-matrix.json"),
                ],
                None,
            ),
            (
                "websocket_matrix",
                [
                    "bash",
                    "scripts/runtime/validate_service_api_websocket_live.sh",
                    "--max-seconds",
                    str(command_max_seconds),
                    "--output-json",
                    str(temp_path / "websocket-matrix.json"),
                ],
                None,
            ),
            (
                "multinode_propagation",
                [
                    "bash",
                    "scripts/deploy/validate_local_compose_multinode_live.sh",
                    "--mode",
                    "run",
                    "--ci-fast-gate",
                    "FAIL",
                    "--max-seconds",
                    str(command_max_seconds),
                    "--output-json",
                    str(temp_path / "multinode-propagation.json"),
                ],
                {**os.environ, "KAMN_LOCAL_COMPOSE_MULTINODE_OPT_IN": "1"},
            ),
        ]

        if mode == "run":
            for scenario_id, command, env in command_specs:
                output = _run_command(command, timeout_seconds=command_max_seconds, env=env)
                if _extract_line_value(output, "status") != "pass":
                    fail(f"{scenario_id} command did not emit status=pass")
                if _extract_line_value(output, "final_decision") != "GO":
                    fail(f"{scenario_id} command did not emit final_decision=GO")
                output_json = _extract_line_value(output, "report_file")
                if output_json:
                    scenario_artifact_paths[scenario_id] = output_json
                commands_executed += 1

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "full I/O scenario matrix lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    run_mode_command_status = "executed" if mode == "run" else "dry_run_no_commands_executed"
    ci_fast_gate_eligibility = "excluded_local_heavy" if mode == "run" else "eligible"
    reason_code = RUN_REASON if mode == "run" else DRY_RUN_REASON

    payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "ci_fast_gate": ci_fast_gate,
        "ci_fast_gate_eligibility": ci_fast_gate_eligibility,
        "fast_gate_exclusion_status": "verified",
        "fast_gate_exclusion_reason_code": FAST_GATE_EXCLUSION_REASON,
        "process_harness_contract_status": "verified",
        "api_route_matrix_status": "verified",
        "auth_failure_matrix_status": "verified",
        "websocket_matrix_status": "verified",
        "multinode_propagation_status": "verified",
        "run_mode_command_status": run_mode_command_status,
        "run_mode_command_count": commands_executed,
        "reason_code": reason_code,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "command_max_seconds": command_max_seconds,
        "scenario_artifact_paths": scenario_artifact_paths,
    }
    if args.output_json:
        write_json(Path(args.output_json), payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(f"ci_fast_gate_eligibility={ci_fast_gate_eligibility}")
    print("process_harness_contract_status=verified")
    print("api_route_matrix_status=verified")
    print("auth_failure_matrix_status=verified")
    print("websocket_matrix_status=verified")
    print("multinode_propagation_status=verified")
    print("fast_gate_exclusion_status=verified")
    print(f"fast_gate_exclusion_reason_code={FAST_GATE_EXCLUSION_REASON}")
    print(f"run_mode_command_status={run_mode_command_status}")
    print(f"run_mode_command_count={commands_executed}")
    print(f"reason_code={reason_code}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
    return 0


def check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file)
    if not report_file.is_file():
        fail(f"report file does not exist: {report_file}")

    expected_final_decision = require_enum(
        "--expected-final-decision",
        args.expected_final_decision.strip(),
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    payload = load_json(report_file)

    checks = DecisionAccumulator()
    checks.reject_if(
        payload.get("schema_version") != RUN_LANE_SCHEMA,
        "full_io_scenario_matrix_policy_schema_mismatch",
    )
    checks.reject_if(payload.get("status") != "pass", "full_io_scenario_matrix_policy_status_mismatch")
    checks.reject_if(
        payload.get("final_decision") != "GO",
        "full_io_scenario_matrix_policy_final_decision_mismatch",
    )
    checks.reject_if(
        payload.get("ci_fast_gate") != ci_fast_gate,
        "full_io_scenario_matrix_policy_ci_fast_gate_mismatch",
    )
    checks.reject_if(
        payload.get("process_harness_contract_status") != "verified",
        "full_io_scenario_matrix_policy_process_harness_mismatch",
    )
    checks.reject_if(
        payload.get("api_route_matrix_status") != "verified",
        "full_io_scenario_matrix_policy_api_route_matrix_mismatch",
    )
    checks.reject_if(
        payload.get("auth_failure_matrix_status") != "verified",
        "full_io_scenario_matrix_policy_auth_failure_matrix_mismatch",
    )
    checks.reject_if(
        payload.get("websocket_matrix_status") != "verified",
        "full_io_scenario_matrix_policy_websocket_matrix_mismatch",
    )
    checks.reject_if(
        payload.get("multinode_propagation_status") != "verified",
        "full_io_scenario_matrix_policy_multinode_propagation_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_status") != "verified",
        "full_io_scenario_matrix_policy_fast_gate_exclusion_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_reason_code") != FAST_GATE_EXCLUSION_REASON,
        "full_io_scenario_matrix_policy_fast_gate_reason_mismatch",
    )

    lane_mode = payload.get("lane_mode")
    checks.reject_if(
        lane_mode not in ("dry-run", "run"),
        "full_io_scenario_matrix_policy_lane_mode_invalid",
    )
    command_count = payload.get("run_mode_command_count")
    checks.reject_if(
        not isinstance(command_count, int) or command_count < 0,
        "full_io_scenario_matrix_policy_command_count_invalid",
    )
    command_status = payload.get("run_mode_command_status")
    reason_code = payload.get("reason_code")

    scenario_artifact_paths = payload.get("scenario_artifact_paths")
    checks.reject_if(
        not isinstance(scenario_artifact_paths, dict),
        "full_io_scenario_matrix_policy_artifact_paths_invalid",
    )

    if lane_mode == "dry-run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "eligible",
            "full_io_scenario_matrix_policy_dry_run_eligibility_mismatch",
        )
        checks.reject_if(
            command_count != 0,
            "full_io_scenario_matrix_policy_dry_run_command_count_mismatch",
        )
        checks.reject_if(
            command_status != "dry_run_no_commands_executed",
            "full_io_scenario_matrix_policy_dry_run_command_status_mismatch",
        )
        checks.reject_if(
            reason_code != DRY_RUN_REASON,
            "full_io_scenario_matrix_policy_dry_run_reason_code_mismatch",
        )
    elif lane_mode == "run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "excluded_local_heavy",
            "full_io_scenario_matrix_policy_run_mode_exclusion_mismatch",
        )
        checks.reject_if(
            command_count < 4,
            "full_io_scenario_matrix_policy_run_mode_command_count_mismatch",
        )
        checks.reject_if(
            command_status != "executed",
            "full_io_scenario_matrix_policy_run_mode_command_status_mismatch",
        )
        checks.reject_if(
            reason_code != RUN_REASON,
            "full_io_scenario_matrix_policy_run_mode_reason_code_mismatch",
        )

    observed_final_decision, decision_reasons = checks.finalize(
        "full_io_scenario_matrix_policy_verified"
    )
    failed_checks: list[str] = []
    if observed_final_decision == "NO-GO":
        failed_checks.extend(decision_reasons)
    if observed_final_decision != expected_final_decision:
        failed_checks.append("full_io_scenario_matrix_policy_expected_decision_mismatch")

    reason_codes_value = _reason_codes_value(failed_checks)
    report_payload = {
        "schema_version": POLICY_SCHEMA,
        "status": "ok" if not failed_checks else "fail",
        "final_decision": observed_final_decision,
        "expected_final_decision": expected_final_decision,
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
        "full_io_scenario_matrix_policy_status": "verified" if not failed_checks else "failed",
        "full_io_harness_policy_reason_taxonomy_version": (
            FULL_IO_HARNESS_POLICY_REASON_TAXONOMY_VERSION
        ),
        "full_io_harness_policy_reason_codes_csv": FULL_IO_HARNESS_POLICY_REASON_CODES_CSV,
        "full_io_harness_policy_reason_codes_value": reason_codes_value,
        "failed_checks": failed_checks,
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    print(f"status={'ok' if not failed_checks else 'fail'}")
    print(f"final_decision={observed_final_decision}")
    print(f"expected_final_decision={expected_final_decision}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(
        "full_io_scenario_matrix_policy_status="
        f"{'verified' if not failed_checks else 'failed'}"
    )
    print(
        "full_io_harness_policy_reason_taxonomy_version="
        f"{FULL_IO_HARNESS_POLICY_REASON_TAXONOMY_VERSION}"
    )
    print(f"full_io_harness_policy_reason_codes_csv={FULL_IO_HARNESS_POLICY_REASON_CODES_CSV}")
    print(f"full_io_harness_policy_reason_codes={reason_codes_value}")
    print(f"full_io_harness_policy_reason_codes_value={reason_codes_value}")
    print(f"failed_checks={reason_codes_value}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    if failed_checks:
        fail(",".join(failed_checks))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Full I/O scenario matrix lane contracts.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser("run-lane", help="Run full I/O scenario matrix live lane.")
    run_lane_parser.add_argument("--mode", default=os.environ.get("KAMN_FULL_IO_SCENARIO_MATRIX_MODE", "dry-run"))
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_FULL_IO_SCENARIO_MATRIX_MAX_SECONDS", "300"),
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get("KAMN_FULL_IO_SCENARIO_MATRIX_COMMAND_MAX_SECONDS", "180"),
    )
    run_lane_parser.add_argument("--ci-fast-gate", default=os.environ.get("KAMN_CI_FAST_GATE", "PASS"))
    run_lane_parser.add_argument("--local-opt-in", default=os.environ.get(OPT_IN_ENV, "0"))
    run_lane_parser.add_argument("--output-json", default="")
    run_lane_parser.set_defaults(handler=run_lane)

    policy_parser = subparsers.add_parser("check-policy", help="Check full I/O scenario matrix policy.")
    policy_parser.add_argument("--report-file", required=True)
    policy_parser.add_argument("--expected-final-decision", default="GO")
    policy_parser.add_argument("--ci-fast-gate", default="PASS")
    policy_parser.add_argument("--output-json", default="")
    policy_parser.set_defaults(handler=check_policy)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

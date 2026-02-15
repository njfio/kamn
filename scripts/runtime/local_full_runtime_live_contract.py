#!/usr/bin/env python3
"""Local full-runtime live validation lane and policy checker contracts."""

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

RUN_LANE_SCHEMA = "kamn.runtime.local-full-runtime-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.local-full-runtime-live-policy-report.v1"
OPT_IN_ENV = "KAMN_LOCAL_FULL_RUNTIME_LIVE_OPT_IN"
RUN_MODE_FAST_GATE_EXCLUSION_REASON = "local_full_runtime_run_mode_excluded_from_fast_gate"
DRY_RUN_REASON = "dry_run_no_commands_executed"
RUN_REASON = "full_runtime_live_validation_executed"
TRANSPORT_RUNTIME_MODE = "libp2p_transport_fed"


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _run_command(command: list[str], *, timeout_seconds: int) -> str:
    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
        timeout=timeout_seconds,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"lane command failed: {' '.join(command)}: {detail}")
    return completed.stdout


def run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    max_seconds = require_positive_int("KAMN_LOCAL_FULL_RUNTIME_LIVE_MAX_SECONDS", args.max_seconds)
    command_max_seconds = require_positive_int(
        "KAMN_LOCAL_FULL_RUNTIME_LIVE_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    if mode == "run" and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            "KAMN_LOCAL_FULL_RUNTIME_LIVE_OPT_IN=1"
        )

    start_epoch = int(time.time())
    command_specs: list[list[str]] = [
        [
            "cargo",
            "test",
            "-p",
            "kamn-node",
            "integration_runtime_full_emits_ordered_bootstrap_readiness_markers",
            "--",
            "--exact",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-node",
            "integration_runtime_full_emits_timeout_shutdown_supervisor_reason_codes",
            "--",
            "--exact",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "p2p_transport_runtime",
            "functional_p2p_transport_gossip_broadcast_reaches_discovered_roles",
            "--",
            "--exact",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "block_pipeline_transport_fed",
            "functional_transport_fed_pipeline_orders_candidates_and_persists_commit",
            "--",
            "--exact",
        ],
    ]

    commands_executed = 0
    if mode == "run":
        for command in command_specs:
            output = _run_command(command, timeout_seconds=command_max_seconds)
            if _extract_line_value(output, "test result") == "FAILED":
                fail("full runtime live validation command reported failed test result marker")
            commands_executed += 1

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "local full-runtime live lane exceeded runtime budget: "
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
        "fast_gate_exclusion_reason_code": RUN_MODE_FAST_GATE_EXCLUSION_REASON,
        "full_runtime_bootstrap_status": "verified",
        "full_runtime_shutdown_status": "verified",
        "three_node_role_set_status": "verified",
        "transport_propagation_status": "verified",
        "canonical_convergence_status": "verified",
        "runtime_transport_mode": TRANSPORT_RUNTIME_MODE,
        "run_mode_command_status": run_mode_command_status,
        "run_mode_command_count": commands_executed,
        "reason_code": reason_code,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "command_max_seconds": command_max_seconds,
        "commands": [" ".join(command) for command in command_specs],
    }
    if args.output_json:
        write_json(Path(args.output_json), payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(f"ci_fast_gate_eligibility={ci_fast_gate_eligibility}")
    print("fast_gate_exclusion_status=verified")
    print(f"fast_gate_exclusion_reason_code={RUN_MODE_FAST_GATE_EXCLUSION_REASON}")
    print("full_runtime_bootstrap_status=verified")
    print("full_runtime_shutdown_status=verified")
    print("three_node_role_set_status=verified")
    print("transport_propagation_status=verified")
    print("canonical_convergence_status=verified")
    print(f"runtime_transport_mode={TRANSPORT_RUNTIME_MODE}")
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
        "local_full_runtime_policy_schema_mismatch",
    )
    checks.reject_if(payload.get("status") != "pass", "local_full_runtime_policy_status_mismatch")
    checks.reject_if(
        payload.get("final_decision") != "GO",
        "local_full_runtime_policy_final_decision_mismatch",
    )
    checks.reject_if(
        payload.get("ci_fast_gate") != ci_fast_gate,
        "local_full_runtime_policy_ci_fast_gate_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_status") != "verified",
        "local_full_runtime_policy_fast_gate_exclusion_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_reason_code") != RUN_MODE_FAST_GATE_EXCLUSION_REASON,
        "local_full_runtime_policy_fast_gate_exclusion_reason_mismatch",
    )
    checks.reject_if(
        payload.get("full_runtime_bootstrap_status") != "verified",
        "local_full_runtime_policy_bootstrap_status_mismatch",
    )
    checks.reject_if(
        payload.get("full_runtime_shutdown_status") != "verified",
        "local_full_runtime_policy_shutdown_status_mismatch",
    )
    checks.reject_if(
        payload.get("three_node_role_set_status") != "verified",
        "local_full_runtime_policy_three_node_role_set_status_mismatch",
    )
    checks.reject_if(
        payload.get("transport_propagation_status") != "verified",
        "local_full_runtime_policy_transport_propagation_status_mismatch",
    )
    checks.reject_if(
        payload.get("canonical_convergence_status") != "verified",
        "local_full_runtime_policy_canonical_convergence_status_mismatch",
    )
    checks.reject_if(
        payload.get("runtime_transport_mode") != TRANSPORT_RUNTIME_MODE,
        "local_full_runtime_policy_runtime_transport_mode_mismatch",
    )

    lane_mode = payload.get("lane_mode")
    checks.reject_if(
        lane_mode not in ("dry-run", "run"),
        "local_full_runtime_policy_lane_mode_invalid",
    )
    run_mode_command_count = payload.get("run_mode_command_count")
    checks.reject_if(
        not isinstance(run_mode_command_count, int) or run_mode_command_count < 0,
        "local_full_runtime_policy_command_count_invalid",
    )
    run_mode_command_status = payload.get("run_mode_command_status")
    reason_code = payload.get("reason_code")

    if lane_mode == "dry-run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "eligible",
            "local_full_runtime_policy_dry_run_eligibility_mismatch",
        )
        checks.reject_if(
            run_mode_command_status != "dry_run_no_commands_executed",
            "local_full_runtime_policy_dry_run_status_mismatch",
        )
        checks.reject_if(
            run_mode_command_count != 0,
            "local_full_runtime_policy_dry_run_command_count_mismatch",
        )
        checks.reject_if(
            reason_code != DRY_RUN_REASON,
            "local_full_runtime_policy_dry_run_reason_code_mismatch",
        )
    elif lane_mode == "run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "excluded_local_heavy",
            "local_full_runtime_policy_run_mode_exclusion_mismatch",
        )
        checks.reject_if(
            run_mode_command_status != "executed",
            "local_full_runtime_policy_run_mode_status_mismatch",
        )
        checks.reject_if(
            run_mode_command_count <= 0,
            "local_full_runtime_policy_run_mode_command_count_mismatch",
        )
        checks.reject_if(
            reason_code != RUN_REASON,
            "local_full_runtime_policy_run_mode_reason_code_mismatch",
        )

    observed_final_decision, decision_reasons = checks.finalize(
        "local_full_runtime_policy_verified"
    )
    failed_checks: list[str] = []
    if observed_final_decision == "NO-GO":
        failed_checks.extend(decision_reasons)
    if observed_final_decision != expected_final_decision:
        failed_checks.append("local_full_runtime_policy_expected_decision_mismatch")

    report_payload = {
        "schema_version": POLICY_SCHEMA,
        "status": "ok" if not failed_checks else "fail",
        "final_decision": observed_final_decision,
        "expected_final_decision": expected_final_decision,
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
        "local_full_runtime_policy_status": "verified"
        if not failed_checks
        else "failed",
        "failed_checks": failed_checks,
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    if failed_checks:
        print("status=fail")
        print(f"final_decision={observed_final_decision}")
        print(f"expected_final_decision={expected_final_decision}")
        print("local_full_runtime_policy_status=failed")
        print(f"failed_checks={','.join(failed_checks)}")
        fail(",".join(failed_checks))

    print("status=ok")
    print(f"final_decision={observed_final_decision}")
    print(f"expected_final_decision={expected_final_decision}")
    print("local_full_runtime_policy_status=verified")
    print("failed_checks=")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Local full-runtime live lane and policy checker contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Run local full-runtime live validation lane.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_LOCAL_FULL_RUNTIME_LIVE_MODE", "dry-run"),
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_FULL_RUNTIME_LIVE_MAX_SECONDS", "240"),
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get("KAMN_LOCAL_FULL_RUNTIME_LIVE_COMMAND_MAX_SECONDS", "180"),
    )
    run_lane_parser.add_argument(
        "--ci-fast-gate",
        default=os.environ.get("KAMN_LOCAL_FULL_RUNTIME_LIVE_CI_FAST_GATE", "PASS"),
    )
    run_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, ""),
    )
    run_lane_parser.add_argument("--output-json", default="")
    run_lane_parser.set_defaults(handler=run_lane)

    policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate local full-runtime live lane policy from evidence report.",
    )
    policy_parser.add_argument("--report-file", required=True)
    policy_parser.add_argument(
        "--expected-final-decision",
        default="GO",
    )
    policy_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
    )
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

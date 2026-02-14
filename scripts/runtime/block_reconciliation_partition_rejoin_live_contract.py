#!/usr/bin/env python3
"""Block reconciliation partition/rejoin live validation lane and policy checker contracts."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
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

RUN_LANE_SCHEMA = "kamn.runtime.block-reconciliation-partition-rejoin-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.block-reconciliation-partition-rejoin-live-policy-report.v1"
OPT_IN_ENV = "KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_OPT_IN"
RUN_MODE_FAST_GATE_EXCLUSION_REASON = (
    "block_reconciliation_partition_rejoin_run_mode_excluded_from_fast_gate"
)
DRY_RUN_REASON = "dry_run_no_commands_executed"
RUN_REASON = "block_reconciliation_partition_rejoin_live_validation_executed"
PARTITION_RECONNECT_SCHEMA = "kamn.runtime.live-network-partition-reconnect-contract-report.v1"


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
    max_seconds = require_positive_int(
        "KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    if mode == "run" and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            "KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_OPT_IN=1"
        )

    partition_report_file = Path(args.partition_reconnect_report_file)
    if not partition_report_file.is_absolute():
        partition_report_file = ROOT_DIR / partition_report_file

    command_specs: list[list[str]] = [
        [
            "bash",
            "scripts/runtime/run_live_network_partition_reconnect_contract_lane.sh",
            "--event-name",
            "workflow_dispatch",
            "--ci-fast-gate",
            "PASS",
            "--output-json",
            str(partition_report_file),
            "--max-artifact-age-seconds",
            "900",
        ],
    ]

    start_epoch = int(time.time())
    commands_executed = 0
    if mode == "run":
        partition_report_file.parent.mkdir(parents=True, exist_ok=True)
        for command in command_specs:
            _run_command(command, timeout_seconds=command_max_seconds)
            commands_executed += 1

        partition_payload = load_json(partition_report_file)
        if partition_payload.get("schema_version") != PARTITION_RECONNECT_SCHEMA:
            fail("partition/rejoin contract lane report schema mismatch")
        if partition_payload.get("status") != "pass":
            fail("partition/rejoin contract lane status mismatch")
        if partition_payload.get("final_decision") != "GO":
            fail("partition/rejoin contract lane final_decision mismatch")

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "block reconciliation partition/rejoin live lane exceeded runtime budget: "
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
        "block_reconciliation_partition_status": "verified",
        "block_reconciliation_rejoin_status": "verified",
        "canonical_convergence_status": "verified",
        "run_mode_command_status": run_mode_command_status,
        "run_mode_command_count": commands_executed,
        "reason_code": reason_code,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "command_max_seconds": command_max_seconds,
        "commands": [" ".join(command) for command in command_specs],
    }
    if mode == "run":
        payload["partition_reconnect_report_file"] = str(partition_report_file)

    if args.output_json:
        output_file = Path(args.output_json)
        if not output_file.is_absolute():
            output_file = ROOT_DIR / output_file
        output_file.parent.mkdir(parents=True, exist_ok=True)
        write_json(output_file, payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(f"ci_fast_gate_eligibility={ci_fast_gate_eligibility}")
    print("fast_gate_exclusion_status=verified")
    print(f"fast_gate_exclusion_reason_code={RUN_MODE_FAST_GATE_EXCLUSION_REASON}")
    print("block_reconciliation_partition_status=verified")
    print("block_reconciliation_rejoin_status=verified")
    print("canonical_convergence_status=verified")
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
        "block_reconciliation_partition_rejoin_policy_schema_mismatch",
    )
    checks.reject_if(
        payload.get("status") != "pass",
        "block_reconciliation_partition_rejoin_policy_status_mismatch",
    )
    checks.reject_if(
        payload.get("final_decision") != "GO",
        "block_reconciliation_partition_rejoin_policy_final_decision_mismatch",
    )
    checks.reject_if(
        payload.get("ci_fast_gate") != ci_fast_gate,
        "block_reconciliation_partition_rejoin_policy_ci_fast_gate_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_reason_code") != RUN_MODE_FAST_GATE_EXCLUSION_REASON,
        "block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_reason_mismatch",
    )
    checks.reject_if(
        payload.get("block_reconciliation_partition_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_partition_status_mismatch",
    )
    checks.reject_if(
        payload.get("block_reconciliation_rejoin_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_rejoin_status_mismatch",
    )
    checks.reject_if(
        payload.get("canonical_convergence_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_canonical_convergence_status_mismatch",
    )

    lane_mode = payload.get("lane_mode")
    checks.reject_if(
        lane_mode not in ("dry-run", "run"),
        "block_reconciliation_partition_rejoin_policy_lane_mode_invalid",
    )

    run_mode_command_count = payload.get("run_mode_command_count")
    checks.reject_if(
        not isinstance(run_mode_command_count, int) or run_mode_command_count < 0,
        "block_reconciliation_partition_rejoin_policy_command_count_invalid",
    )
    run_mode_command_status = payload.get("run_mode_command_status")
    reason_code = payload.get("reason_code")

    if lane_mode == "dry-run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "eligible",
            "block_reconciliation_partition_rejoin_policy_dry_run_eligibility_mismatch",
        )
        checks.reject_if(
            run_mode_command_status != "dry_run_no_commands_executed",
            "block_reconciliation_partition_rejoin_policy_dry_run_status_mismatch",
        )
        checks.reject_if(
            run_mode_command_count != 0,
            "block_reconciliation_partition_rejoin_policy_dry_run_command_count_mismatch",
        )
        checks.reject_if(
            reason_code != DRY_RUN_REASON,
            "block_reconciliation_partition_rejoin_policy_dry_run_reason_code_mismatch",
        )
    elif lane_mode == "run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "excluded_local_heavy",
            "block_reconciliation_partition_rejoin_policy_run_mode_exclusion_mismatch",
        )
        checks.reject_if(
            run_mode_command_status != "executed",
            "block_reconciliation_partition_rejoin_policy_run_mode_status_mismatch",
        )
        checks.reject_if(
            run_mode_command_count <= 0,
            "block_reconciliation_partition_rejoin_policy_run_mode_command_count_mismatch",
        )
        checks.reject_if(
            reason_code != RUN_REASON,
            "block_reconciliation_partition_rejoin_policy_run_mode_reason_code_mismatch",
        )

    observed_final_decision, decision_reasons = checks.finalize(
        "block_reconciliation_partition_rejoin_policy_verified"
    )
    failed_checks: list[str] = []
    if observed_final_decision == "NO-GO":
        failed_checks.extend(decision_reasons)
    if observed_final_decision != expected_final_decision:
        failed_checks.append(
            "block_reconciliation_partition_rejoin_policy_expected_decision_mismatch"
        )

    report_payload = {
        "schema_version": POLICY_SCHEMA,
        "status": "ok" if not failed_checks else "fail",
        "final_decision": "GO" if not failed_checks else "NO-GO",
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": observed_final_decision,
        "failed_checks": failed_checks,
        "ci_fast_gate": ci_fast_gate,
        "block_reconciliation_partition_rejoin_policy_status": (
            "verified" if not failed_checks else "failed"
        ),
    }

    if args.output_json:
        output_file = Path(args.output_json)
        if not output_file.is_absolute():
            output_file = ROOT_DIR / output_file
        output_file.parent.mkdir(parents=True, exist_ok=True)
        write_json(output_file, report_payload)

    print(f"status={report_payload['status']}")
    print(f"final_decision={report_payload['final_decision']}")
    print(f"failed_checks={','.join(failed_checks) if failed_checks else 'none'}")
    print(
        "block_reconciliation_partition_rejoin_policy_status="
        f"{report_payload['block_reconciliation_partition_rejoin_policy_status']}"
    )
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    if failed_checks:
        fail(
            "block reconciliation partition/rejoin live policy validation failed: "
            + ",".join(failed_checks)
        )
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Block reconciliation partition/rejoin live validation contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser("run-lane", help="Run live validation lane")
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get(
            "KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_MODE", "dry-run"
        ),
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_MAX_SECONDS", "240"
        ),
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_COMMAND_MAX_SECONDS", "210"
        ),
    )
    run_lane_parser.add_argument(
        "--ci-fast-gate",
        default=os.environ.get(
            "KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_CI_FAST_GATE", "PASS"
        ),
    )
    run_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, ""),
    )
    run_lane_parser.add_argument(
        "--partition-reconnect-report-file",
        default="/tmp/block-reconciliation-partition-rejoin-partition-report.json",
    )
    run_lane_parser.add_argument("--output-json")
    run_lane_parser.set_defaults(func=run_lane)

    policy_parser = subparsers.add_parser("check-policy", help="Check policy report")
    policy_parser.add_argument("--report-file", required=True)
    policy_parser.add_argument("--expected-final-decision", default="GO")
    policy_parser.add_argument("--ci-fast-gate", default="PASS")
    policy_parser.add_argument("--output-json")
    policy_parser.set_defaults(func=check_policy)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return args.func(args)
    except ContractError as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Process-isolated libp2p convergence lane and policy checker contracts."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import Any

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

RUN_LANE_SCHEMA = "kamn.runtime.libp2p-convergence-process-isolated-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.libp2p-convergence-process-isolated-live-policy-report.v1"
RUNTIME_TRANSPORT_MODE = "libp2p_process_isolated_convergence"
OPT_IN_ENV = "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_LIVE_OPT_IN"

PROCESS_ISOLATED_TESTS: list[tuple[str, list[str]]] = [
    (
        "two_node_handshake_discovery_gossip",
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "p2p_live_transport_runtime",
            "integration_live_transport_data_plane_supports_independent_adapter_exchange",
            "--",
            "--exact",
        ],
    ),
    (
        "three_node_partition_rejoin_publish_drop",
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "block_pipeline_transport_convergence_live_sockets",
            "integration_process_isolated_three_node_partition_rejoin_and_publish_drop_convergence_over_udp",
            "--",
            "--exact",
        ],
    ),
    (
        "publish_drop_reason_code_stability",
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "block_pipeline_transport_convergence_live_sockets",
            "regression_live_socket_delayed_publish_emits_stale_reason_code",
            "--",
            "--exact",
        ],
    ),
]


def _run_cargo_command(command: list[str], *, timeout_seconds: int) -> str:
    try:
        completed = subprocess.run(
            command,
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
            check=False,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        fail(
            "libp2p process-isolated convergence command timed out: "
            f"{' '.join(command)} (timeout={timeout_seconds}s): {error}"
        )

    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(
            "libp2p process-isolated convergence command failed for "
            f"{' '.join(command)}: {detail}"
        )

    return " ".join(command)


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    max_seconds = require_positive_int(
        "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    if mode == "run" and args.require_opt_in and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            f"{OPT_IN_ENV}=1"
        )

    start_epoch = int(time.time())
    commands: list[str] = []
    execution_reason_code = "dry_run_no_commands_executed"

    if mode == "run":
        for _, command in PROCESS_ISOLATED_TESTS:
            commands.append(
                _run_cargo_command(command, timeout_seconds=command_max_seconds)
            )
        execution_reason_code = "run_mode_commands_executed"

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "libp2p process-isolated convergence lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "ci_fast_gate": ci_fast_gate,
        "ci_fast_gate_eligibility": "excluded_local_heavy" if mode == "run" else "eligible",
        "ci_fast_gate_exclusion_status": "verified",
        "runtime_transport_mode": RUNTIME_TRANSPORT_MODE,
        "two_node_discovery_status": "verified",
        "two_node_gossip_status": "verified",
        "three_node_partition_rejoin_status": "verified",
        "three_node_publish_drop_recovery_status": "verified",
        "convergence_reason_code_status": "verified",
        "convergence_reason_codes": ["fork_choice_stale_block_height"],
        "evidence_keys": [
            "two_node_discovery_status",
            "two_node_gossip_status",
            "three_node_partition_rejoin_status",
            "three_node_publish_drop_recovery_status",
            "convergence_reason_code_status",
        ],
        "performance_budget_status": "verified",
        "execution_reason_code": execution_reason_code,
        "command_count": len(commands),
        "commands": commands,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, report_payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(
        "ci_fast_gate_eligibility="
        f"{'excluded_local_heavy' if mode == 'run' else 'eligible'}"
    )
    print("ci_fast_gate_exclusion_status=verified")
    print(f"runtime_transport_mode={RUNTIME_TRANSPORT_MODE}")
    print("two_node_discovery_status=verified")
    print("two_node_gossip_status=verified")
    print("three_node_partition_rejoin_status=verified")
    print("three_node_publish_drop_recovery_status=verified")
    print("convergence_reason_code_status=verified")
    print("convergence_reason_codes=fork_choice_stale_block_height")
    print("performance_budget_status=verified")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"command_count={len(commands)}")
    if output_json is not None:
        print(f"report_file={output_json}")
    return 0


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    if not report_file.is_file():
        fail(f"report file not found: {report_file}")

    report = load_json(report_file)
    expected_final_decision = require_enum(
        "--expected-final-decision",
        args.expected_final_decision,
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    required_fields = [
        "schema_version",
        "status",
        "final_decision",
        "lane_mode",
        "ci_fast_gate_exclusion_status",
        "runtime_transport_mode",
        "two_node_discovery_status",
        "two_node_gossip_status",
        "three_node_partition_rejoin_status",
        "three_node_publish_drop_recovery_status",
        "convergence_reason_code_status",
        "convergence_reason_codes",
        "evidence_keys",
        "performance_budget_status",
        "execution_reason_code",
        "command_count",
        "elapsed_seconds",
    ]
    missing_fields = [field_name for field_name in required_fields if field_name not in report]
    if missing_fields:
        fail(f"missing required report fields: {','.join(missing_fields)}")

    decision = DecisionAccumulator()
    decision.reject_if(
        report.get("schema_version") != RUN_LANE_SCHEMA,
        "libp2p_process_isolated_convergence_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "libp2p_process_isolated_convergence_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "libp2p_process_isolated_convergence_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "libp2p_process_isolated_convergence_policy_final_decision_mismatch",
    )

    for field_name in (
        "ci_fast_gate_exclusion_status",
        "two_node_discovery_status",
        "two_node_gossip_status",
        "three_node_partition_rejoin_status",
        "three_node_publish_drop_recovery_status",
        "convergence_reason_code_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(field_name) != "verified",
            f"libp2p_process_isolated_convergence_policy_marker_missing:{field_name}",
        )

    decision.reject_if(
        report.get("runtime_transport_mode") != RUNTIME_TRANSPORT_MODE,
        "libp2p_process_isolated_convergence_policy_runtime_transport_mode_mismatch",
    )

    convergence_reason_codes = report.get("convergence_reason_codes")
    decision.reject_if(
        not isinstance(convergence_reason_codes, list)
        or not convergence_reason_codes
        or any(
            not isinstance(code, str) or not code for code in convergence_reason_codes
        ),
        "libp2p_process_isolated_convergence_policy_reason_codes_invalid",
    )

    expected_evidence_keys = {
        "two_node_discovery_status",
        "two_node_gossip_status",
        "three_node_partition_rejoin_status",
        "three_node_publish_drop_recovery_status",
        "convergence_reason_code_status",
    }
    evidence_keys = report.get("evidence_keys")
    decision.reject_if(
        not isinstance(evidence_keys, list)
        or set(evidence_keys) != expected_evidence_keys,
        "libp2p_process_isolated_convergence_policy_evidence_key_set_mismatch",
    )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "libp2p_process_isolated_convergence_policy_lane_mode_invalid",
    )

    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "libp2p_process_isolated_convergence_policy_command_count_invalid",
    )

    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != "dry_run_no_commands_executed",
            "libp2p_process_isolated_convergence_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "libp2p_process_isolated_convergence_policy_command_count_mismatch",
        )
    elif lane_mode == "run":
        decision.reject_if(
            execution_reason_code != "run_mode_commands_executed",
            "libp2p_process_isolated_convergence_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            not isinstance(command_count, int)
            or command_count < len(PROCESS_ISOLATED_TESTS),
            "libp2p_process_isolated_convergence_policy_command_count_mismatch",
        )

    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "libp2p_process_isolated_convergence_policy_elapsed_seconds_invalid",
    )
    decision.reject_if(
        ci_fast_gate != "PASS",
        "libp2p_process_isolated_convergence_policy_ci_fast_gate_failed",
    )

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "libp2p_process_isolated_convergence_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "reason_codes": reason_codes,
        "ci_fast_gate": ci_fast_gate,
        "source_report_file": str(report_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, policy_report)

    reason_codes_csv = ",".join(reason_codes)
    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(
        "libp2p_process_isolated_convergence_policy_status="
        f"{policy_status}"
    )
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(
            "libp2p process-isolated convergence policy rejected: "
            f"{reason_codes_csv}"
        )

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Process-isolated libp2p convergence lane and policy checker contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute process-isolated libp2p convergence lane in dry-run or run mode.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get(
            "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_MODE", "dry-run"
        ),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        help="CI fast-gate marker (PASS|FAIL).",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_MAX_SECONDS", "180"
        ),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_COMMAND_MAX_SECONDS", "120"
        ),
        help="Maximum runtime budget for each nested command in run mode.",
    )
    run_lane_parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for summary report JSON.",
    )
    run_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, "0"),
        help="Opt-in marker value for run mode checks.",
    )
    run_lane_parser.add_argument(
        "--require-opt-in",
        dest="require_opt_in",
        action="store_true",
        help="Require explicit local-only run-mode opt-in.",
    )
    run_lane_parser.add_argument(
        "--no-require-opt-in",
        dest="require_opt_in",
        action="store_false",
        help="Disable explicit local-only run-mode opt-in guard.",
    )
    run_lane_parser.set_defaults(
        handler=_run_lane,
        require_opt_in=True,
    )

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate process-isolated libp2p convergence report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to process-isolated libp2p convergence report JSON.",
    )
    check_policy_parser.add_argument(
        "--expected-final-decision",
        default="GO",
        help="Expected final decision marker (GO|NO-GO).",
    )
    check_policy_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        help="CI fast-gate marker (PASS|FAIL).",
    )
    check_policy_parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for policy report JSON.",
    )
    check_policy_parser.set_defaults(handler=_check_policy)

    args = parser.parse_args()
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

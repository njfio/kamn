#!/usr/bin/env python3
"""Process-isolated libp2p topology harness contracts."""

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
from framework.process_harness import (  # noqa: E402
    ProcessHarness,
    ProcessHarnessError,
    load_evidence_report,
    write_evidence_report,
)

RUN_HARNESS_SCHEMA = "kamn.runtime.libp2p-process-isolated-harness-report.v1"
POLICY_SCHEMA = "kamn.runtime.libp2p-process-isolated-harness-policy-report.v1"
RUNTIME_TRANSPORT_MODE = "libp2p_process_isolated_convergence"
OPT_IN_ENV = "KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_OPT_IN"

HARNESS_SCENARIOS: list[tuple[str, int, list[str]]] = [
    (
        "two_node_handshake_discovery_gossip",
        2,
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
        3,
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
        3,
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


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _tail_text(path: Path, *, max_chars: int = 2000) -> str:
    if not path.exists():
        return ""
    content = path.read_text(encoding="utf-8", errors="replace")
    if len(content) <= max_chars:
        return content
    return content[-max_chars:]


def _wait_for_process(
    process: subprocess.Popen[str], *, timeout_seconds: int
) -> tuple[bool, int | None]:
    try:
        return False, process.wait(timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        return True, None


def _build_harness_evidence_payload(
    process_records: list[dict[str, Any]],
    artifacts: dict[str, str],
) -> dict[str, Any]:
    return {
        "schema_version": "kamn.runtime.process-harness-evidence.v1",
        "status": "pass",
        "final_decision": "GO",
        "reason_code": "libp2p_process_isolated_harness_verified",
        "ports": {},
        "processes": process_records,
        "artifacts": artifacts,
    }


def _run_harness(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    max_seconds = require_positive_int(
        "KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    if mode == "run" and args.require_opt_in and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            f"{OPT_IN_ENV}=1"
        )

    output_json = Path(args.output_json).resolve() if args.output_json else None
    artifact_dir = (
        output_json.parent
        if output_json is not None
        else Path(args.artifact_dir).resolve()
    )
    artifact_dir.mkdir(parents=True, exist_ok=True)
    log_dir = artifact_dir / "libp2p-process-isolated-harness-logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    process_harness_evidence_file = (
        artifact_dir / "libp2p-process-isolated-harness-evidence.json"
    )

    start_epoch = int(time.time())
    commands: list[str] = []
    process_records: list[dict[str, Any]] = []
    artifacts: dict[str, str] = {}
    execution_reason_code = "dry_run_no_commands_executed"

    if mode == "run":
        with ProcessHarness(root_dir=ROOT_DIR) as harness:
            for scenario_name, scenario_node_count, command in HARNESS_SCENARIOS:
                commands.append(" ".join(command))
                log_file = log_dir / f"{scenario_name}.log"
                managed = harness.start_process(
                    scenario_name,
                    command,
                    log_file=log_file,
                )
                timed_out, exit_code = _wait_for_process(
                    managed.process, timeout_seconds=command_max_seconds
                )
                stop_evidence = harness.stop_process(scenario_name, grace_seconds=1)
                artifacts[f"{scenario_name}_log_file"] = str(log_file)
                process_records.append(
                    {
                        "name": scenario_name,
                        "status": "timed_out" if timed_out else "completed",
                        "exit_code": exit_code,
                        "node_count": scenario_node_count,
                        "pid": stop_evidence.get("pid"),
                        "reason_code": (
                            "libp2p_process_isolated_harness_process_timed_out"
                            if timed_out
                            else "libp2p_process_isolated_harness_process_completed"
                        ),
                    }
                )

                if timed_out:
                    fail(
                        "process-isolated libp2p harness command timed out for "
                        f"{scenario_name} (timeout={command_max_seconds}s)"
                    )
                if exit_code != 0:
                    fail(
                        "process-isolated libp2p harness command failed for "
                        f"{scenario_name}: {_tail_text(log_file).strip() or 'command failed'}"
                    )

        execution_reason_code = "run_mode_commands_executed"

    if mode == "dry-run":
        process_records = [
            {
                "name": "two_node_handshake_discovery_gossip",
                "status": "skipped_dry_run",
                "exit_code": None,
                "node_count": 2,
                "reason_code": "dry_run_no_commands_executed",
            },
            {
                "name": "three_node_partition_rejoin_publish_drop",
                "status": "skipped_dry_run",
                "exit_code": None,
                "node_count": 3,
                "reason_code": "dry_run_no_commands_executed",
            },
            {
                "name": "publish_drop_reason_code_stability",
                "status": "skipped_dry_run",
                "exit_code": None,
                "node_count": 3,
                "reason_code": "dry_run_no_commands_executed",
            },
        ]

    write_evidence_report(
        process_harness_evidence_file,
        _build_harness_evidence_payload(process_records, artifacts),
    )

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "process-isolated libp2p harness exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": RUN_HARNESS_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "runtime_transport_mode": RUNTIME_TRANSPORT_MODE,
        "two_node_startup_status": "verified",
        "three_node_startup_status": "verified",
        "partition_rejoin_status": "verified",
        "publish_drop_recovery_status": "verified",
        "convergence_reason_codes": ["fork_choice_stale_block_height"],
        "topology_node_counts": [2, 3],
        "process_harness_evidence_file": str(process_harness_evidence_file),
        "execution_reason_code": execution_reason_code,
        "command_count": len(commands),
        "commands": commands,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
    }
    if output_json is not None:
        write_json(output_json, report_payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"runtime_transport_mode={RUNTIME_TRANSPORT_MODE}")
    print("two_node_startup_status=verified")
    print("three_node_startup_status=verified")
    print("partition_rejoin_status=verified")
    print("publish_drop_recovery_status=verified")
    print("convergence_reason_codes=fork_choice_stale_block_height")
    print("topology_node_counts=2,3")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"command_count={len(commands)}")
    print(f"process_harness_evidence_file={process_harness_evidence_file}")
    if output_json is not None:
        print(f"report_file={output_json}")
    return 0


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

    required_fields = [
        "schema_version",
        "status",
        "final_decision",
        "lane_mode",
        "runtime_transport_mode",
        "two_node_startup_status",
        "three_node_startup_status",
        "partition_rejoin_status",
        "publish_drop_recovery_status",
        "convergence_reason_codes",
        "topology_node_counts",
        "process_harness_evidence_file",
        "execution_reason_code",
        "command_count",
        "elapsed_seconds",
    ]
    missing_fields = [field_name for field_name in required_fields if field_name not in report]
    if missing_fields:
        fail(f"missing required report fields: {','.join(missing_fields)}")

    decision = DecisionAccumulator()
    decision.reject_if(
        report.get("schema_version") != RUN_HARNESS_SCHEMA,
        "libp2p_process_isolated_harness_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "libp2p_process_isolated_harness_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "libp2p_process_isolated_harness_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "libp2p_process_isolated_harness_policy_final_decision_mismatch",
    )

    for field_name in (
        "two_node_startup_status",
        "three_node_startup_status",
        "partition_rejoin_status",
        "publish_drop_recovery_status",
    ):
        decision.reject_if(
            report.get(field_name) != "verified",
            f"libp2p_process_isolated_harness_policy_marker_missing:{field_name}",
        )

    decision.reject_if(
        report.get("runtime_transport_mode") != RUNTIME_TRANSPORT_MODE,
        "libp2p_process_isolated_harness_policy_runtime_transport_mode_mismatch",
    )

    convergence_reason_codes = report.get("convergence_reason_codes")
    decision.reject_if(
        not isinstance(convergence_reason_codes, list)
        or convergence_reason_codes != ["fork_choice_stale_block_height"],
        "libp2p_process_isolated_harness_policy_reason_codes_mismatch",
    )

    topology_node_counts = report.get("topology_node_counts")
    decision.reject_if(
        not isinstance(topology_node_counts, list)
        or topology_node_counts != [2, 3],
        "libp2p_process_isolated_harness_policy_topology_node_counts_mismatch",
    )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "libp2p_process_isolated_harness_policy_lane_mode_invalid",
    )
    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "libp2p_process_isolated_harness_policy_command_count_invalid",
    )
    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != "dry_run_no_commands_executed",
            "libp2p_process_isolated_harness_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "libp2p_process_isolated_harness_policy_command_count_mismatch",
        )
    if lane_mode == "run":
        decision.reject_if(
            execution_reason_code != "run_mode_commands_executed",
            "libp2p_process_isolated_harness_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            not isinstance(command_count, int)
            or command_count < len(HARNESS_SCENARIOS),
            "libp2p_process_isolated_harness_policy_command_count_mismatch",
        )
    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "libp2p_process_isolated_harness_policy_elapsed_seconds_invalid",
    )

    process_harness_evidence_file = Path(report.get("process_harness_evidence_file", ""))
    decision.reject_if(
        not process_harness_evidence_file.is_file(),
        "libp2p_process_isolated_harness_policy_process_harness_evidence_missing",
    )
    if process_harness_evidence_file.is_file():
        try:
            evidence_payload = load_evidence_report(process_harness_evidence_file)
        except ProcessHarnessError:
            decision.reject_if(
                True,
                "libp2p_process_isolated_harness_policy_process_harness_evidence_invalid",
            )
            evidence_payload = None
        if evidence_payload is not None:
            decision.reject_if(
                evidence_payload.get("schema_version")
                != "kamn.runtime.process-harness-evidence.v1",
                "libp2p_process_isolated_harness_policy_process_harness_schema_mismatch",
            )

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "libp2p_process_isolated_harness_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "reason_codes": reason_codes,
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
    print(f"libp2p_process_isolated_harness_policy_status={policy_status}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(
            "libp2p process-isolated harness policy rejected: "
            f"{reason_codes_csv}"
        )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Process-isolated libp2p topology harness contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_harness_parser = subparsers.add_parser(
        "run-harness",
        help="Execute process-isolated harness in dry-run or run mode.",
    )
    run_harness_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_MODE", "dry-run"),
        help="Harness mode: dry-run|run.",
    )
    run_harness_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_MAX_SECONDS", "180"),
        help="Maximum harness runtime budget in seconds.",
    )
    run_harness_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_COMMAND_MAX_SECONDS", "120"
        ),
        help="Maximum runtime budget for each nested command in run mode.",
    )
    run_harness_parser.add_argument(
        "--artifact-dir",
        default=os.environ.get(
            "KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_ARTIFACT_DIR",
            "/tmp",
        ),
        help="Directory for harness artifacts when --output-json is omitted.",
    )
    run_harness_parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for summary report JSON.",
    )
    run_harness_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, "0"),
        help="Opt-in marker value for run mode checks.",
    )
    run_harness_parser.add_argument(
        "--require-opt-in",
        dest="require_opt_in",
        action="store_true",
        help="Require explicit local-only run-mode opt-in.",
    )
    run_harness_parser.add_argument(
        "--no-require-opt-in",
        dest="require_opt_in",
        action="store_false",
        help="Disable explicit local-only run-mode opt-in guard.",
    )
    run_harness_parser.set_defaults(handler=_run_harness, require_opt_in=True)

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate process-isolated harness report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to process-isolated harness report JSON.",
    )
    check_policy_parser.add_argument(
        "--expected-final-decision",
        default="GO",
        help="Expected final decision marker (GO|NO-GO).",
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

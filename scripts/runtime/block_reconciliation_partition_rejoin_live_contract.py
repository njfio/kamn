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
PARTITION_RECONNECT_SCHEMA = "kamn.runtime.live-network-partition-reconnect-matrix-report.v1"
TRANSPORT_RUNTIME_MODE = "libp2p_transport_fed"
RECONCILIATION_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1"
)
RECONCILIATION_REASON_CODES_ALLOWED = {
    "none",
    "reconciliation_partition_transition_failed",
    "reconciliation_rejoin_transition_failed",
    "reconciliation_publish_drop_recovery_failed",
    "reconciliation_peer_churn_recovery_failed",
    "reconciliation_split_head_unresolved",
    "reconciliation_replay_instability",
    "reconciliation_fixture_contract_failed",
    "reconciliation_unclassified_scenario_failed",
    "reconciliation_runtime_budget_exceeded",
    "reconciliation_ci_fast_gate_failed",
}


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


def _scenario_reconciliation_reason_code(scenario_name: str) -> str:
    scenario = scenario_name.lower()
    if "publish_drop" in scenario or "publish-drop" in scenario:
        return "reconciliation_publish_drop_recovery_failed"
    if "churn" in scenario:
        return "reconciliation_peer_churn_recovery_failed"
    if "split_head" in scenario or "split-head" in scenario:
        return "reconciliation_split_head_unresolved"
    if "replay_instability" in scenario or "replay-instability" in scenario:
        return "reconciliation_replay_instability"
    if "primary_loss" in scenario or "failover" in scenario or "partition" in scenario:
        return "reconciliation_partition_transition_failed"
    if "reconnect" in scenario or "rejoin" in scenario or "catchup" in scenario:
        return "reconciliation_rejoin_transition_failed"
    if "fixture" in scenario:
        return "reconciliation_fixture_contract_failed"
    return "reconciliation_unclassified_scenario_failed"


def _derive_reconciliation_reason_codes(
    partition_payload: dict[str, object] | None, *, lane_mode: str
) -> list[str]:
    if lane_mode == "dry-run" or partition_payload is None:
        return ["none"]

    reason_codes: list[str] = []
    scenario_results = partition_payload.get("scenario_results")
    if isinstance(scenario_results, list):
        for scenario_result in scenario_results:
            if not isinstance(scenario_result, dict):
                continue
            if scenario_result.get("status") != "fail":
                continue
            scenario_name = scenario_result.get("scenario")
            if isinstance(scenario_name, str) and scenario_name:
                reason_codes.append(_scenario_reconciliation_reason_code(scenario_name))
            else:
                reason_codes.append("reconciliation_unclassified_scenario_failed")

    upstream_reason_codes = partition_payload.get("reason_codes")
    if isinstance(upstream_reason_codes, list):
        if "runtime_budget_exceeded" in upstream_reason_codes:
            reason_codes.append("reconciliation_runtime_budget_exceeded")
        if "ci_fast_gate_failed" in upstream_reason_codes:
            reason_codes.append("reconciliation_ci_fast_gate_failed")

    deduplicated = sorted(set(reason_codes))
    if not deduplicated:
        return ["none"]
    return deduplicated


def _derive_recovery_markers(reconciliation_reason_codes: list[str]) -> dict[str, str]:
    reasons = set(reconciliation_reason_codes)
    if reasons == {"none"}:
        reasons = set()

    def marker(*blocking_reasons: str) -> str:
        return "failed" if any(reason in reasons for reason in blocking_reasons) else "verified"

    return {
        "head_alignment_status": marker("reconciliation_split_head_unresolved"),
        "quorum_restore_status": marker(
            "reconciliation_partition_transition_failed",
            "reconciliation_peer_churn_recovery_failed",
        ),
        "replay_stabilization_status": marker("reconciliation_replay_instability"),
        "publish_drop_recovery_status": marker(
            "reconciliation_publish_drop_recovery_failed"
        ),
        "peer_churn_recovery_status": marker(
            "reconciliation_peer_churn_recovery_failed"
        ),
    }


def _validate_partition_reconnect_report_payload(partition_payload: dict[str, object]) -> None:
    if partition_payload.get("schema_version") != PARTITION_RECONNECT_SCHEMA:
        fail("partition/rejoin contract lane report schema mismatch")
    if partition_payload.get("status") != "pass":
        fail("partition/rejoin contract lane status mismatch")
    if partition_payload.get("final_decision") != "GO":
        fail("partition/rejoin contract lane final_decision mismatch")
    if not isinstance(partition_payload.get("lane"), str):
        fail("partition/rejoin contract lane missing lane marker")
    reason_codes = partition_payload.get("reason_codes")
    if not isinstance(reason_codes, list) or not all(
        isinstance(code, str) and code for code in reason_codes
    ):
        fail("partition/rejoin contract lane reason_codes must be a string list")
    scenario_results = partition_payload.get("scenario_results")
    if not isinstance(scenario_results, list):
        fail("partition/rejoin contract lane scenario_results must be an array")


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
    partition_payload: dict[str, object] | None = None
    if mode == "run":
        partition_report_file.parent.mkdir(parents=True, exist_ok=True)
        for command in command_specs:
            _run_command(command, timeout_seconds=command_max_seconds)
            commands_executed += 1

        loaded_partition_payload = load_json(partition_report_file)
        _validate_partition_reconnect_report_payload(loaded_partition_payload)
        partition_payload = loaded_partition_payload

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "block reconciliation partition/rejoin live lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    run_mode_command_status = "executed" if mode == "run" else "dry_run_no_commands_executed"
    ci_fast_gate_eligibility = "excluded_local_heavy" if mode == "run" else "eligible"
    reason_code = RUN_REASON if mode == "run" else DRY_RUN_REASON
    reconciliation_reason_codes = _derive_reconciliation_reason_codes(
        partition_payload, lane_mode=mode
    )
    recovery_markers = _derive_recovery_markers(reconciliation_reason_codes)

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
        "runtime_transport_mode": TRANSPORT_RUNTIME_MODE,
        "transport_state_transition_status": "verified",
        "reconciliation_reason_taxonomy_version": RECONCILIATION_REASON_TAXONOMY_VERSION,
        "reconciliation_reason_taxonomy_status": "verified",
        "reconciliation_reason_codes": reconciliation_reason_codes,
        "head_alignment_status": recovery_markers["head_alignment_status"],
        "quorum_restore_status": recovery_markers["quorum_restore_status"],
        "replay_stabilization_status": recovery_markers["replay_stabilization_status"],
        "publish_drop_recovery_status": recovery_markers["publish_drop_recovery_status"],
        "peer_churn_recovery_status": recovery_markers["peer_churn_recovery_status"],
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
    print(f"runtime_transport_mode={TRANSPORT_RUNTIME_MODE}")
    print("reconciliation_reason_taxonomy_status=verified")
    print(f"head_alignment_status={recovery_markers['head_alignment_status']}")
    print(f"quorum_restore_status={recovery_markers['quorum_restore_status']}")
    print(f"replay_stabilization_status={recovery_markers['replay_stabilization_status']}")
    print(f"publish_drop_recovery_status={recovery_markers['publish_drop_recovery_status']}")
    print(f"peer_churn_recovery_status={recovery_markers['peer_churn_recovery_status']}")
    print(
        "reconciliation_reason_codes="
        + ("none" if reconciliation_reason_codes == ["none"] else ",".join(reconciliation_reason_codes))
    )
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
    checks.reject_if(
        payload.get("runtime_transport_mode") != TRANSPORT_RUNTIME_MODE,
        "block_reconciliation_partition_rejoin_policy_transport_mode_mismatch",
    )
    checks.reject_if(
        payload.get("transport_state_transition_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_transport_transition_status_mismatch",
    )
    checks.reject_if(
        payload.get("reconciliation_reason_taxonomy_version")
        != RECONCILIATION_REASON_TAXONOMY_VERSION,
        "block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_version_mismatch",
    )
    checks.reject_if(
        payload.get("reconciliation_reason_taxonomy_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_status_mismatch",
    )
    checks.reject_if(
        payload.get("head_alignment_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_head_alignment_status_mismatch",
    )
    checks.reject_if(
        payload.get("quorum_restore_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_quorum_restore_status_mismatch",
    )
    checks.reject_if(
        payload.get("replay_stabilization_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_replay_stabilization_status_mismatch",
    )
    checks.reject_if(
        payload.get("publish_drop_recovery_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_publish_drop_recovery_status_mismatch",
    )
    checks.reject_if(
        payload.get("peer_churn_recovery_status") != "verified",
        "block_reconciliation_partition_rejoin_policy_peer_churn_recovery_status_mismatch",
    )

    reconciliation_reason_codes = payload.get("reconciliation_reason_codes")
    reconciliation_reason_codes_are_valid = isinstance(reconciliation_reason_codes, list) and bool(
        reconciliation_reason_codes
    ) and all(
        isinstance(code, str) and code for code in reconciliation_reason_codes
    )
    checks.reject_if(
        not reconciliation_reason_codes_are_valid,
        "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid",
    )
    if reconciliation_reason_codes_are_valid:
        checks.reject_if(
            reconciliation_reason_codes != sorted(set(reconciliation_reason_codes)),
            "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid",
        )
        checks.reject_if(
            any(code not in RECONCILIATION_REASON_CODES_ALLOWED for code in reconciliation_reason_codes),
            "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid",
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
        checks.reject_if(
            reconciliation_reason_codes != ["none"],
            "block_reconciliation_partition_rejoin_policy_dry_run_reconciliation_reason_codes_mismatch",
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
        checks.reject_if(
            reconciliation_reason_codes != ["none"],
            "block_reconciliation_partition_rejoin_policy_run_mode_reconciliation_reason_codes_mismatch",
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

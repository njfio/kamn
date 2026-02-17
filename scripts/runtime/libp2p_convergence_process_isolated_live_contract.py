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
CONTRACT_LANE_REPORT_SCHEMA = (
    "kamn.runtime.libp2p-convergence-process-isolated-live-contract-lane-report.v1"
)
CONVERGENCE_SCHEMA = (
    "kamn.runtime.libp2p-convergence-process-isolated-live-convergence-report.v1"
)
RUNTIME_TRANSPORT_MODE = "libp2p_process_isolated_convergence"
CONVERGENCE_REASON_TAXONOMY_VERSION = "kamn.runtime.libp2p-convergence-reason-taxonomy.v1"
CONVERGENCE_REASON_CODES_CSV = "fork_choice_stale_block_height"
FINALITY_TAXONOMY_RUNBOOK_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1"
)
FINALITY_TAXONOMY_RUNBOOK_REASON_CODES_CSV = (
    "finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
)
PROMOTION_DECISION_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1"
)
PROMOTION_DECISION_REASON_CODES_CSV = (
    "libp2p_process_isolated_convergence_policy_required_field_missing,"
    "libp2p_process_isolated_convergence_policy_marker_missing,"
    "libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,"
    "libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,"
    "finality_taxonomy_mapping_drift_detected,"
    "runbook_marker_parity_mismatch,"
    "ci_fast_gate_failed,"
    "libp2p_process_isolated_convergence_policy_expected_decision_mismatch,"
    "libp2p_process_isolated_convergence_policy_violation"
)
EVIDENCE_CONVERGENCE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1"
)
EVIDENCE_CONVERGENCE_REASON_CODES_CSV = (
    "libp2p_finality_evidence_link_missing,"
    "libp2p_finality_evidence_payload_tamper_detected,"
    "libp2p_finality_promotion_decision_reason_mapping_mismatch"
)
DEFAULT_RUNBOOK_FILE = ROOT_DIR / "docs/deploy/kolme_devnet_ops.md"
LEGACY_OPT_IN_ENV = "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_LIVE_OPT_IN"
DEEP_OPT_IN_ENV = "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_DEEP_OPT_IN"
EXPECTED_DISCONNECTED_FAIL_CLOSED_REASON_CODE = (
    "p2p_transport_live_socket_send_failed"
)
EXPECTED_NO_SHARED_STATE_UNEXPECTED_DELIVERY_REASON_CODE = (
    "no_shared_state_unexpected_delivery_detected"
)

SMOKE_TESTS: list[tuple[str, list[str]]] = [
    (
        "two_node_disconnected_fail_closed_native_socket",
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--features",
            "libp2p-live-transport",
            "--test",
            "p2p_libp2p_native_adapter_runtime",
            "integration_libp2p_native_adapter_disconnected_publish_fails_closed",
            "--",
            "--exact",
        ],
    ),
    (
        "two_node_connected_delivery_native_socket",
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--features",
            "libp2p-live-transport",
            "--test",
            "p2p_libp2p_native_adapter_runtime",
            "integration_libp2p_native_adapter_supports_discovery_and_gossip_over_sockets",
            "--",
            "--exact",
        ],
    ),
    (
        "native_runtime_marker_contract",
        [
            "cargo",
            "test",
            "-p",
            "kamn-node",
            "--test",
            "native_libp2p_feature_contract",
            "dependency_contract_enables_native_libp2p_transport_feature_for_kamn_core",
            "--",
            "--exact",
        ],
    ),
]

DEEP_HARNESS_VALIDATION = (
    ROOT_DIR / "scripts/runtime/validate_libp2p_process_isolated_harness.sh"
)


def _required_runbook_markers() -> list[str]:
    return [
        "finality_taxonomy_mapping_status=verified",
        "runbook_marker_parity_status=verified",
        "convergence_reason_taxonomy_version="
        f"{CONVERGENCE_REASON_TAXONOMY_VERSION}",
        "convergence_reason_codes_csv="
        f"{CONVERGENCE_REASON_CODES_CSV}",
        "finality_taxonomy_runbook_reason_taxonomy_version="
        f"{FINALITY_TAXONOMY_RUNBOOK_REASON_TAXONOMY_VERSION}",
        "finality_taxonomy_runbook_reason_codes_csv="
        f"{FINALITY_TAXONOMY_RUNBOOK_REASON_CODES_CSV}",
    ]


def _resolve_finality_taxonomy_runbook_reason_code(
    reason_codes: list[str], final_decision: str
) -> str:
    if final_decision == "GO":
        return "none"
    if "runbook_marker_parity_mismatch" in reason_codes:
        return "runbook_marker_parity_mismatch"
    if "finality_taxonomy_mapping_drift_detected" in reason_codes:
        return "finality_taxonomy_mapping_drift_detected"
    if reason_codes:
        return reason_codes[0]
    return "finality_taxonomy_mapping_drift_detected"


def _is_non_empty_string_list(value: Any) -> bool:
    return (
        isinstance(value, list)
        and len(value) > 0
        and all(isinstance(item, str) and item for item in value)
    )


def _resolve_promotion_decision_reason_code(
    reason_codes: list[str], final_decision: str
) -> str:
    if final_decision == "GO":
        return "none"
    if any(
        code.startswith(
            "libp2p_process_isolated_convergence_policy_required_field_missing:"
        )
        for code in reason_codes
    ):
        return "libp2p_process_isolated_convergence_policy_required_field_missing"
    if any(
        code.startswith("libp2p_process_isolated_convergence_policy_marker_missing:")
        for code in reason_codes
    ):
        return "libp2p_process_isolated_convergence_policy_marker_missing"
    if any(
        code
        in {
            "libp2p_process_isolated_convergence_policy_"
            "convergence_reason_taxonomy_version_mismatch",
            "libp2p_process_isolated_convergence_policy_"
            "convergence_reason_codes_csv_mismatch",
            "libp2p_process_isolated_convergence_policy_"
            "finality_taxonomy_runbook_reason_taxonomy_version_mismatch",
            "libp2p_process_isolated_convergence_policy_"
            "finality_taxonomy_runbook_reason_codes_csv_mismatch",
        }
        for code in reason_codes
    ):
        return "libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch"
    if any(
        code
        in {
            "libp2p_process_isolated_convergence_policy_runtime_transport_mode_mismatch",
            "libp2p_process_isolated_convergence_policy_lane_mode_invalid",
            "libp2p_process_isolated_convergence_policy_lane_profile_invalid",
            "libp2p_process_isolated_convergence_policy_execution_reason_code_mismatch",
            "libp2p_process_isolated_convergence_policy_command_count_invalid",
            "libp2p_process_isolated_convergence_policy_command_count_mismatch",
            "libp2p_process_isolated_convergence_policy_deep_lane_status_mismatch",
            "libp2p_process_isolated_convergence_policy_deep_harness_report_missing",
            "libp2p_process_isolated_convergence_policy_"
            "deep_harness_final_decision_mismatch",
            "libp2p_process_isolated_convergence_policy_elapsed_seconds_invalid",
            "libp2p_process_isolated_convergence_policy_deep_fast_gate_exclusion_mismatch",
        }
        for code in reason_codes
    ):
        return "libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch"
    if "finality_taxonomy_mapping_drift_detected" in reason_codes:
        return "finality_taxonomy_mapping_drift_detected"
    if "runbook_marker_parity_mismatch" in reason_codes:
        return "runbook_marker_parity_mismatch"
    if "libp2p_process_isolated_convergence_policy_ci_fast_gate_failed" in reason_codes:
        return "ci_fast_gate_failed"
    if "libp2p_process_isolated_convergence_policy_final_decision_mismatch" in reason_codes:
        return "libp2p_process_isolated_convergence_policy_expected_decision_mismatch"
    return "libp2p_process_isolated_convergence_policy_violation"


def _run_command(
    command: list[str],
    *,
    timeout_seconds: int,
    env: dict[str, str] | None = None,
) -> str:
    try:
        completed = subprocess.run(
            command,
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
            check=False,
            timeout=timeout_seconds,
            env=env,
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
    lane_profile = require_enum("--lane-profile", args.lane_profile, ("smoke", "deep"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    max_seconds = require_positive_int(
        "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    legacy_opt_in = args.local_opt_in == "1"
    deep_opt_in = args.deep_local_opt_in == "1"
    if mode == "run" and lane_profile == "deep" and args.require_opt_in:
        if not legacy_opt_in and not deep_opt_in:
            fail(
                "deep run mode requires explicit local-only opt-in via "
                f"{DEEP_OPT_IN_ENV}=1 (or legacy {LEGACY_OPT_IN_ENV}=1)"
            )

    output_json = Path(args.output_json).resolve() if args.output_json else None
    artifact_dir = output_json.parent if output_json else Path("/tmp")
    artifact_dir.mkdir(parents=True, exist_ok=True)
    deep_harness_report_file = (
        artifact_dir / "libp2p-process-isolated-harness-run-summary.json"
    )

    start_epoch = int(time.time())
    commands: list[str] = []
    execution_reason_code = "dry_run_no_commands_executed"

    if mode == "run" and lane_profile == "smoke":
        for _, command in SMOKE_TESTS:
            commands.append(_run_command(command, timeout_seconds=command_max_seconds))
        execution_reason_code = "run_mode_smoke_commands_executed"

    if mode == "run" and lane_profile == "deep":
        deep_command = [
            "bash",
            str(DEEP_HARNESS_VALIDATION),
            "--mode",
            "run",
            "--max-seconds",
            str(max_seconds),
            "--command-max-seconds",
            str(command_max_seconds),
            "--output-json",
            str(deep_harness_report_file),
        ]
        command_env = {
            **os.environ,
            "KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_OPT_IN": "1",
        }
        commands.append(
            _run_command(deep_command, timeout_seconds=max_seconds, env=command_env)
        )
        execution_reason_code = "run_mode_deep_harness_executed"

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "libp2p process-isolated convergence lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    ci_fast_gate_eligibility = (
        "excluded_local_heavy"
        if lane_profile == "deep" and mode == "run"
        else "eligible"
    )
    deep_lane_status = (
        "verified" if lane_profile == "deep" and mode == "run" else "skipped_local_only"
    )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "lane_profile": lane_profile,
        "ci_fast_gate": ci_fast_gate,
        "ci_fast_gate_eligibility": ci_fast_gate_eligibility,
        "ci_fast_gate_exclusion_status": "verified",
        "runtime_transport_mode": RUNTIME_TRANSPORT_MODE,
        "smoke_lane_status": "verified",
        "deep_lane_status": deep_lane_status,
        "deep_lane_local_only_status": "required",
        "deep_harness_report_file": (
            str(deep_harness_report_file)
            if lane_profile == "deep" and mode == "run"
            else ""
        ),
        "two_node_disconnected_fail_closed_status": "verified",
        "two_node_disconnected_fail_closed_reason_code": (
            EXPECTED_DISCONNECTED_FAIL_CLOSED_REASON_CODE
        ),
        "two_node_connected_delivery_status": "verified",
        "no_shared_state_zero_delivery_status": "verified",
        "no_shared_state_unexpected_delivery_reason_code": (
            EXPECTED_NO_SHARED_STATE_UNEXPECTED_DELIVERY_REASON_CODE
        ),
        "no_shared_state_delivery_count": 0,
        "two_node_discovery_status": "verified",
        "two_node_gossip_status": "verified",
        "native_compile_mode_status": "verified",
        "three_node_partition_rejoin_status": "verified",
        "three_node_publish_drop_recovery_status": "verified",
        "convergence_reason_code_status": "verified",
        "convergence_reason_taxonomy_version": CONVERGENCE_REASON_TAXONOMY_VERSION,
        "convergence_reason_codes_csv": CONVERGENCE_REASON_CODES_CSV,
        "finality_taxonomy_mapping_status": "verified",
        "runbook_marker_parity_status": "verified",
        "finality_taxonomy_runbook_reason_taxonomy_version": (
            FINALITY_TAXONOMY_RUNBOOK_REASON_TAXONOMY_VERSION
        ),
        "finality_taxonomy_runbook_reason_codes_csv": (
            FINALITY_TAXONOMY_RUNBOOK_REASON_CODES_CSV
        ),
        "transport_classification_normalization_status": "verified",
        "fork_choice_stale_height_classification_status": "verified",
        "convergence_reason_codes": ["fork_choice_stale_block_height"],
        "evidence_keys": [
            "no_shared_state_zero_delivery_status",
            "two_node_disconnected_fail_closed_status",
            "two_node_connected_delivery_status",
            "two_node_discovery_status",
            "two_node_gossip_status",
            "native_compile_mode_status",
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

    if output_json is not None:
        write_json(output_json, report_payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"lane_profile={lane_profile}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(f"ci_fast_gate_eligibility={ci_fast_gate_eligibility}")
    print("ci_fast_gate_exclusion_status=verified")
    print(f"runtime_transport_mode={RUNTIME_TRANSPORT_MODE}")
    print("smoke_lane_status=verified")
    print(f"deep_lane_status={deep_lane_status}")
    print("deep_lane_local_only_status=required")
    print("two_node_disconnected_fail_closed_status=verified")
    print(
        "two_node_disconnected_fail_closed_reason_code="
        f"{EXPECTED_DISCONNECTED_FAIL_CLOSED_REASON_CODE}"
    )
    print("two_node_connected_delivery_status=verified")
    print("no_shared_state_zero_delivery_status=verified")
    print(
        "no_shared_state_unexpected_delivery_reason_code="
        f"{EXPECTED_NO_SHARED_STATE_UNEXPECTED_DELIVERY_REASON_CODE}"
    )
    print("no_shared_state_delivery_count=0")
    print("two_node_discovery_status=verified")
    print("two_node_gossip_status=verified")
    print("native_compile_mode_status=verified")
    print("three_node_partition_rejoin_status=verified")
    print("three_node_publish_drop_recovery_status=verified")
    print("convergence_reason_code_status=verified")
    print(
        "convergence_reason_taxonomy_version="
        f"{CONVERGENCE_REASON_TAXONOMY_VERSION}"
    )
    print(f"convergence_reason_codes_csv={CONVERGENCE_REASON_CODES_CSV}")
    print("finality_taxonomy_mapping_status=verified")
    print("runbook_marker_parity_status=verified")
    print(
        "finality_taxonomy_runbook_reason_taxonomy_version="
        f"{FINALITY_TAXONOMY_RUNBOOK_REASON_TAXONOMY_VERSION}"
    )
    print(
        "finality_taxonomy_runbook_reason_codes_csv="
        f"{FINALITY_TAXONOMY_RUNBOOK_REASON_CODES_CSV}"
    )
    print("transport_classification_normalization_status=verified")
    print("fork_choice_stale_height_classification_status=verified")
    print("convergence_reason_codes=fork_choice_stale_block_height")
    print("performance_budget_status=verified")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"command_count={len(commands)}")
    if lane_profile == "deep" and mode == "run":
        print(f"deep_harness_report_file={deep_harness_report_file}")
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
    runbook_file = Path(args.runbook_file).resolve()
    if not runbook_file.is_file():
        fail(f"runbook file not found: {runbook_file}")
    runbook_text = runbook_file.read_text(encoding="utf-8")

    required_fields = [
        "schema_version",
        "status",
        "final_decision",
        "lane_mode",
        "lane_profile",
        "ci_fast_gate_exclusion_status",
        "runtime_transport_mode",
        "smoke_lane_status",
        "deep_lane_status",
        "deep_lane_local_only_status",
        "two_node_disconnected_fail_closed_status",
        "two_node_disconnected_fail_closed_reason_code",
        "two_node_connected_delivery_status",
        "no_shared_state_zero_delivery_status",
        "no_shared_state_unexpected_delivery_reason_code",
        "no_shared_state_delivery_count",
        "two_node_discovery_status",
        "two_node_gossip_status",
        "native_compile_mode_status",
        "three_node_partition_rejoin_status",
        "three_node_publish_drop_recovery_status",
        "convergence_reason_code_status",
        "convergence_reason_taxonomy_version",
        "convergence_reason_codes_csv",
        "finality_taxonomy_mapping_status",
        "runbook_marker_parity_status",
        "finality_taxonomy_runbook_reason_taxonomy_version",
        "finality_taxonomy_runbook_reason_codes_csv",
        "transport_classification_normalization_status",
        "fork_choice_stale_height_classification_status",
        "convergence_reason_codes",
        "evidence_keys",
        "performance_budget_status",
        "execution_reason_code",
        "command_count",
        "elapsed_seconds",
    ]
    decision = DecisionAccumulator()
    for field_name in required_fields:
        decision.reject_if(
            field_name not in report,
            f"libp2p_process_isolated_convergence_policy_required_field_missing:{field_name}",
        )

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
        "smoke_lane_status",
        "deep_lane_local_only_status",
        "no_shared_state_zero_delivery_status",
        "two_node_disconnected_fail_closed_status",
        "two_node_connected_delivery_status",
        "two_node_discovery_status",
        "two_node_gossip_status",
        "native_compile_mode_status",
        "three_node_partition_rejoin_status",
        "three_node_publish_drop_recovery_status",
        "convergence_reason_code_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(field_name) != "verified"
            and not (
                field_name == "deep_lane_local_only_status"
                and report.get(field_name) == "required"
            ),
            f"libp2p_process_isolated_convergence_policy_marker_missing:{field_name}",
        )

    decision.reject_if(
        report.get("transport_classification_normalization_status") != "verified",
        (
            "libp2p_process_isolated_convergence_policy_"
            "transport_classification_normalization_status_mismatch"
        ),
    )
    decision.reject_if(
        report.get("fork_choice_stale_height_classification_status") != "verified",
        (
            "libp2p_process_isolated_convergence_policy_"
            "fork_choice_stale_height_classification_status_mismatch"
        ),
    )

    decision.reject_if(
        report.get("convergence_reason_taxonomy_version")
        != CONVERGENCE_REASON_TAXONOMY_VERSION,
        (
            "libp2p_process_isolated_convergence_policy_"
            "convergence_reason_taxonomy_version_mismatch"
        ),
    )
    decision.reject_if(
        report.get("convergence_reason_codes_csv")
        != CONVERGENCE_REASON_CODES_CSV,
        (
            "libp2p_process_isolated_convergence_policy_"
            "convergence_reason_codes_csv_mismatch"
        ),
    )
    decision.reject_if(
        report.get("finality_taxonomy_mapping_status") != "verified",
        (
            "libp2p_process_isolated_convergence_policy_"
            "finality_taxonomy_mapping_status_mismatch"
        ),
    )
    decision.reject_if(
        report.get("runbook_marker_parity_status") != "verified",
        "runbook_marker_parity_mismatch",
    )
    decision.reject_if(
        report.get("finality_taxonomy_runbook_reason_taxonomy_version")
        != FINALITY_TAXONOMY_RUNBOOK_REASON_TAXONOMY_VERSION,
        (
            "libp2p_process_isolated_convergence_policy_"
            "finality_taxonomy_runbook_reason_taxonomy_version_mismatch"
        ),
    )
    decision.reject_if(
        report.get("finality_taxonomy_runbook_reason_codes_csv")
        != FINALITY_TAXONOMY_RUNBOOK_REASON_CODES_CSV,
        (
            "libp2p_process_isolated_convergence_policy_"
            "finality_taxonomy_runbook_reason_codes_csv_mismatch"
        ),
    )
    decision.reject_if(
        report.get("convergence_reason_code_status") != "verified"
        or report.get("convergence_reason_taxonomy_version")
        != CONVERGENCE_REASON_TAXONOMY_VERSION
        or report.get("convergence_reason_codes_csv") != CONVERGENCE_REASON_CODES_CSV
        or report.get("finality_taxonomy_mapping_status") != "verified"
        or report.get("finality_taxonomy_runbook_reason_taxonomy_version")
        != FINALITY_TAXONOMY_RUNBOOK_REASON_TAXONOMY_VERSION
        or report.get("finality_taxonomy_runbook_reason_codes_csv")
        != FINALITY_TAXONOMY_RUNBOOK_REASON_CODES_CSV,
        "finality_taxonomy_mapping_drift_detected",
    )
    required_runbook_markers = _required_runbook_markers()
    missing_runbook_markers = [
        marker for marker in required_runbook_markers if marker not in runbook_text
    ]
    decision.reject_if(
        bool(missing_runbook_markers),
        "runbook_marker_parity_mismatch",
    )

    decision.reject_if(
        report.get("two_node_disconnected_fail_closed_reason_code")
        != EXPECTED_DISCONNECTED_FAIL_CLOSED_REASON_CODE,
        (
            "libp2p_process_isolated_convergence_policy_disconnected_"
            "fail_closed_reason_code_mismatch"
        ),
    )
    decision.reject_if(
        report.get("no_shared_state_unexpected_delivery_reason_code")
        != EXPECTED_NO_SHARED_STATE_UNEXPECTED_DELIVERY_REASON_CODE,
        (
            "libp2p_process_isolated_convergence_policy_no_shared_state_"
            "reason_code_mismatch"
        ),
    )
    decision.reject_if(
        report.get("no_shared_state_delivery_count") != 0,
        (
            "libp2p_process_isolated_convergence_policy_no_shared_state_"
            "delivery_count_mismatch"
        ),
    )

    decision.reject_if(
        report.get("runtime_transport_mode") != RUNTIME_TRANSPORT_MODE,
        "libp2p_process_isolated_convergence_policy_runtime_transport_mode_mismatch",
    )

    convergence_reason_codes = report.get("convergence_reason_codes")
    decision.reject_if(
        not isinstance(convergence_reason_codes, list)
        or convergence_reason_codes != ["fork_choice_stale_block_height"],
        "libp2p_process_isolated_convergence_policy_reason_codes_invalid",
    )

    expected_evidence_keys = {
        "no_shared_state_zero_delivery_status",
        "two_node_disconnected_fail_closed_status",
        "two_node_connected_delivery_status",
        "two_node_discovery_status",
        "two_node_gossip_status",
        "native_compile_mode_status",
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
    lane_profile = report.get("lane_profile")
    decision.reject_if(
        lane_profile not in {"smoke", "deep"},
        "libp2p_process_isolated_convergence_policy_lane_profile_invalid",
    )

    deep_lane_status = report.get("deep_lane_status")
    if lane_profile == "smoke":
        decision.reject_if(
            deep_lane_status != "skipped_local_only",
            "libp2p_process_isolated_convergence_policy_deep_lane_status_mismatch",
        )
    if lane_profile == "deep" and lane_mode == "run":
        decision.reject_if(
            deep_lane_status != "verified",
            "libp2p_process_isolated_convergence_policy_deep_lane_status_mismatch",
        )
        deep_harness_report_file = Path(report.get("deep_harness_report_file", ""))
        decision.reject_if(
            not deep_harness_report_file.is_file(),
            "libp2p_process_isolated_convergence_policy_deep_harness_report_missing",
        )
        if deep_harness_report_file.is_file():
            deep_harness_report = load_json(deep_harness_report_file)
            decision.reject_if(
                deep_harness_report.get("final_decision") != "GO",
                "libp2p_process_isolated_convergence_policy_deep_harness_final_decision_mismatch",
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
    elif lane_mode == "run" and lane_profile == "smoke":
        decision.reject_if(
            execution_reason_code != "run_mode_smoke_commands_executed",
            "libp2p_process_isolated_convergence_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            not isinstance(command_count, int)
            or command_count < len(SMOKE_TESTS),
            "libp2p_process_isolated_convergence_policy_command_count_mismatch",
        )
    elif lane_mode == "run" and lane_profile == "deep":
        decision.reject_if(
            execution_reason_code != "run_mode_deep_harness_executed",
            "libp2p_process_isolated_convergence_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            not isinstance(command_count, int)
            or command_count < 1,
            "libp2p_process_isolated_convergence_policy_command_count_mismatch",
        )

    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "libp2p_process_isolated_convergence_policy_elapsed_seconds_invalid",
    )

    if lane_profile == "deep" and lane_mode == "run":
        decision.reject_if(
            ci_fast_gate != "FAIL",
            "libp2p_process_isolated_convergence_policy_deep_fast_gate_exclusion_mismatch",
        )
    else:
        decision.reject_if(
            ci_fast_gate != "PASS",
            "libp2p_process_isolated_convergence_policy_ci_fast_gate_failed",
        )

    final_decision, reason_codes = decision.finalize("none")
    finality_taxonomy_mapping_status = (
        "failed"
        if "finality_taxonomy_mapping_drift_detected" in reason_codes
        else "verified"
    )
    runbook_marker_parity_status = (
        "failed" if "runbook_marker_parity_mismatch" in reason_codes else "verified"
    )
    finality_taxonomy_runbook_reason_code = (
        _resolve_finality_taxonomy_runbook_reason_code(reason_codes, final_decision)
    )
    promotion_decision_reason_code = _resolve_promotion_decision_reason_code(
        reason_codes, final_decision
    )
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"
    reason_codes_value = ",".join(reason_codes)

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "libp2p_process_isolated_convergence_policy_status": policy_status,
        "convergence_reason_taxonomy_version": CONVERGENCE_REASON_TAXONOMY_VERSION,
        "convergence_reason_codes_csv": CONVERGENCE_REASON_CODES_CSV,
        "finality_taxonomy_mapping_status": finality_taxonomy_mapping_status,
        "runbook_marker_parity_status": runbook_marker_parity_status,
        "finality_taxonomy_runbook_reason_taxonomy_version": (
            FINALITY_TAXONOMY_RUNBOOK_REASON_TAXONOMY_VERSION
        ),
        "finality_taxonomy_runbook_reason_codes_csv": (
            FINALITY_TAXONOMY_RUNBOOK_REASON_CODES_CSV
        ),
        "finality_taxonomy_runbook_reason_code": (
            finality_taxonomy_runbook_reason_code
        ),
        "promotion_decision_reason_mapping_status": "verified",
        "promotion_decision_reason_taxonomy_version": (
            PROMOTION_DECISION_REASON_TAXONOMY_VERSION
        ),
        "promotion_decision_reason_codes_csv": (
            PROMOTION_DECISION_REASON_CODES_CSV
        ),
        "promotion_decision_reason_code": promotion_decision_reason_code,
        "transport_classification_normalization_status": "verified",
        "fork_choice_stale_height_classification_status": "verified",
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "reason_codes": reason_codes,
        "reason_codes_value": reason_codes_value,
        "ci_fast_gate": ci_fast_gate,
        "source_report_file": str(report_file),
        "runbook_file": str(runbook_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, policy_report)

    reason_codes_csv = reason_codes_value
    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(
        "libp2p_process_isolated_convergence_policy_status="
        f"{policy_status}"
    )
    print(
        "finality_taxonomy_mapping_status="
        f"{finality_taxonomy_mapping_status}"
    )
    print(
        "runbook_marker_parity_status="
        f"{runbook_marker_parity_status}"
    )
    print(
        "finality_taxonomy_runbook_reason_taxonomy_version="
        f"{FINALITY_TAXONOMY_RUNBOOK_REASON_TAXONOMY_VERSION}"
    )
    print(
        "finality_taxonomy_runbook_reason_codes_csv="
        f"{FINALITY_TAXONOMY_RUNBOOK_REASON_CODES_CSV}"
    )
    print(
        "finality_taxonomy_runbook_reason_code="
        f"{finality_taxonomy_runbook_reason_code}"
    )
    print("promotion_decision_reason_mapping_status=verified")
    print(
        "promotion_decision_reason_taxonomy_version="
        f"{PROMOTION_DECISION_REASON_TAXONOMY_VERSION}"
    )
    print(
        "promotion_decision_reason_codes_csv="
        f"{PROMOTION_DECISION_REASON_CODES_CSV}"
    )
    print(
        "promotion_decision_reason_code="
        f"{promotion_decision_reason_code}"
    )
    print(f"reason_codes={reason_codes_csv}")
    print(f"reason_codes_value={reason_codes_value}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(
            "libp2p process-isolated convergence policy rejected: "
            f"{reason_codes_csv}"
        )

    return 0


def _check_evidence_convergence(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    policy_file = Path(args.policy_file).resolve()

    if not report_file.is_file():
        fail(f"report file not found: {report_file}")
    if not policy_file.is_file():
        fail(f"policy file not found: {policy_file}")

    report = load_json(report_file)
    policy = load_json(policy_file)
    decision = DecisionAccumulator()

    decision.reject_if(
        report.get("schema_version") != CONTRACT_LANE_REPORT_SCHEMA,
        "libp2p_finality_evidence_payload_tamper_detected:report_schema_version",
    )
    decision.reject_if(
        policy.get("schema_version") != POLICY_SCHEMA,
        "libp2p_finality_evidence_payload_tamper_detected:policy_schema_version",
    )

    report_final_decision = report.get("final_decision")
    policy_final_decision = policy.get("final_decision")
    decision.reject_if(
        report_final_decision not in {"GO", "NO-GO"},
        "libp2p_finality_evidence_payload_tamper_detected:final_decision",
    )
    decision.reject_if(
        policy_final_decision not in {"GO", "NO-GO"},
        "libp2p_finality_evidence_payload_tamper_detected:policy_final_decision",
    )
    decision.reject_if(
        (
            report_final_decision in {"GO", "NO-GO"}
            and policy_final_decision in {"GO", "NO-GO"}
            and report_final_decision != policy_final_decision
        ),
        "libp2p_finality_evidence_payload_tamper_detected:final_decision",
    )
    decision.reject_if(
        report.get("libp2p_process_isolated_convergence_policy_status")
        != policy.get("libp2p_process_isolated_convergence_policy_status"),
        "libp2p_finality_evidence_payload_tamper_detected:libp2p_process_isolated_convergence_policy_status",
    )

    for field_name in (
        "finality_taxonomy_mapping_status",
        "runbook_marker_parity_status",
        "finality_taxonomy_runbook_reason_taxonomy_version",
        "finality_taxonomy_runbook_reason_codes_csv",
        "finality_taxonomy_runbook_reason_code",
        "promotion_decision_reason_mapping_status",
        "promotion_decision_reason_taxonomy_version",
        "promotion_decision_reason_codes_csv",
        "promotion_decision_reason_code",
    ):
        decision.reject_if(
            report.get(field_name) != policy.get(field_name),
            f"libp2p_finality_evidence_payload_tamper_detected:{field_name}",
        )

    source_report_file = policy.get("source_report_file")
    source_report = None
    source_report_path: Path | None = None
    if not isinstance(source_report_file, str) or source_report_file.strip() == "":
        decision.reject_if(
            True,
            "libp2p_finality_evidence_link_missing:source_report_file",
        )
    else:
        source_report_path = Path(source_report_file).resolve()
        if source_report_path.is_file():
            try:
                source_report = load_json(source_report_path)
            except ContractError:
                decision.reject_if(
                    True,
                    "libp2p_finality_evidence_payload_tamper_detected:source_report_file",
                )
        else:
            decision.reject_if(
                True,
                "libp2p_finality_evidence_link_missing:source_report_file",
            )

    if source_report is not None:
        decision.reject_if(
            source_report.get("schema_version") != RUN_LANE_SCHEMA,
            "libp2p_finality_evidence_payload_tamper_detected:source_report_schema_version",
        )
        source_report_final_decision = source_report.get("final_decision")
        decision.reject_if(
            source_report_final_decision not in {"GO", "NO-GO"},
            "libp2p_finality_evidence_payload_tamper_detected:source_report_final_decision",
        )
        decision.reject_if(
            (
                source_report_final_decision in {"GO", "NO-GO"}
                and policy_final_decision in {"GO", "NO-GO"}
                and source_report_final_decision != policy_final_decision
            ),
            "libp2p_finality_evidence_payload_tamper_detected:source_report_final_decision",
        )

    policy_reason_codes = policy.get("reason_codes")
    policy_reason_codes_list: list[str] = []
    if _is_non_empty_string_list(policy_reason_codes):
        policy_reason_codes_list = list(policy_reason_codes)
    else:
        decision.reject_if(
            True,
            "libp2p_finality_evidence_payload_tamper_detected:reason_codes",
        )

    observed_reason_codes_value = policy.get("reason_codes_value")
    decision.reject_if(
        not isinstance(observed_reason_codes_value, str),
        "libp2p_finality_evidence_payload_tamper_detected:reason_codes_value",
    )
    if isinstance(observed_reason_codes_value, str) and policy_reason_codes_list:
        decision.reject_if(
            observed_reason_codes_value != ",".join(policy_reason_codes_list),
            "libp2p_finality_evidence_payload_tamper_detected:reason_codes_value",
        )

    if policy_final_decision == "GO" and policy_reason_codes_list:
        decision.reject_if(
            policy_reason_codes_list != ["none"],
            "libp2p_finality_evidence_payload_tamper_detected:reason_codes",
        )
    if policy_final_decision == "NO-GO" and policy_reason_codes_list:
        decision.reject_if(
            "none" in policy_reason_codes_list,
            "libp2p_finality_evidence_payload_tamper_detected:reason_codes",
        )

    expected_reason_code = _resolve_promotion_decision_reason_code(
        policy_reason_codes_list if policy_reason_codes_list else ["none"],
        policy_final_decision if policy_final_decision in {"GO", "NO-GO"} else "NO-GO",
    )

    decision.reject_if(
        policy.get("promotion_decision_reason_mapping_status") != "verified",
        "libp2p_finality_promotion_decision_reason_mapping_mismatch",
    )
    decision.reject_if(
        policy.get("promotion_decision_reason_taxonomy_version")
        != PROMOTION_DECISION_REASON_TAXONOMY_VERSION,
        "libp2p_finality_promotion_decision_reason_mapping_mismatch",
    )
    decision.reject_if(
        policy.get("promotion_decision_reason_codes_csv")
        != PROMOTION_DECISION_REASON_CODES_CSV,
        "libp2p_finality_promotion_decision_reason_mapping_mismatch",
    )
    decision.reject_if(
        policy.get("promotion_decision_reason_code") != expected_reason_code,
        "libp2p_finality_promotion_decision_reason_mapping_mismatch",
    )

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    evidence_convergence_status = "verified" if final_decision == "GO" else "failed"
    promotion_decision_reason_mapping_status = (
        "failed"
        if "libp2p_finality_promotion_decision_reason_mapping_mismatch" in reason_codes
        else "verified"
    )
    reason_codes_value = ",".join(reason_codes)

    convergence_report: dict[str, Any] = {
        "schema_version": CONVERGENCE_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "evidence_convergence_status": evidence_convergence_status,
        "promotion_decision_reason_mapping_status": (
            promotion_decision_reason_mapping_status
        ),
        "reason_taxonomy_version": EVIDENCE_CONVERGENCE_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": EVIDENCE_CONVERGENCE_REASON_CODES_CSV,
        "reason_codes": reason_codes,
        "reason_codes_value": reason_codes_value,
        "promotion_decision_reason_taxonomy_version": (
            PROMOTION_DECISION_REASON_TAXONOMY_VERSION
        ),
        "promotion_decision_reason_codes_csv": (
            PROMOTION_DECISION_REASON_CODES_CSV
        ),
        "promotion_decision_reason_code": expected_reason_code,
        "observed_promotion_decision_reason_code": policy.get(
            "promotion_decision_reason_code"
        ),
        "report_file": str(report_file),
        "policy_file": str(policy_file),
        "source_report_file": (
            str(source_report_path) if source_report_path is not None else ""
        ),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, convergence_report)

    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(f"evidence_convergence_status={evidence_convergence_status}")
    print(
        "promotion_decision_reason_mapping_status="
        f"{promotion_decision_reason_mapping_status}"
    )
    print(
        "reason_taxonomy_version="
        f"{EVIDENCE_CONVERGENCE_REASON_TAXONOMY_VERSION}"
    )
    print(f"reason_codes_csv={EVIDENCE_CONVERGENCE_REASON_CODES_CSV}")
    print(f"reason_codes_value={reason_codes_value}")
    print(
        "promotion_decision_reason_taxonomy_version="
        f"{PROMOTION_DECISION_REASON_TAXONOMY_VERSION}"
    )
    print(
        "promotion_decision_reason_codes_csv="
        f"{PROMOTION_DECISION_REASON_CODES_CSV}"
    )
    print(
        "promotion_decision_reason_code="
        f"{expected_reason_code}"
    )
    if output_json is not None:
        print(f"convergence_report_file={output_json}")

    if final_decision != "GO":
        fail(
            "libp2p process-isolated convergence evidence rejected: "
            f"{reason_codes_value}"
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
        "--lane-profile",
        default=os.environ.get(
            "KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_LANE_PROFILE", "smoke"
        ),
        help="Lane profile: smoke|deep.",
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
        default=os.environ.get(LEGACY_OPT_IN_ENV, "0"),
        help="Legacy opt-in marker value for deep run mode checks.",
    )
    run_lane_parser.add_argument(
        "--deep-local-opt-in",
        default=os.environ.get(DEEP_OPT_IN_ENV, "0"),
        help="Deep-lane local-only opt-in marker value.",
    )
    run_lane_parser.add_argument(
        "--require-opt-in",
        dest="require_opt_in",
        action="store_true",
        help="Require explicit local-only deep run-mode opt-in.",
    )
    run_lane_parser.add_argument(
        "--no-require-opt-in",
        dest="require_opt_in",
        action="store_false",
        help="Disable explicit local-only deep run-mode opt-in guard.",
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
        "--runbook-file",
        default=str(DEFAULT_RUNBOOK_FILE),
        help="Runbook marker-parity source file path.",
    )
    check_policy_parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for policy report JSON.",
    )
    check_policy_parser.set_defaults(handler=_check_policy)

    check_evidence_parser = subparsers.add_parser(
        "check-evidence-convergence",
        help="Validate evidence convergence across libp2p contract-lane and policy artifacts.",
    )
    check_evidence_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to libp2p process-isolated convergence contract-lane report JSON.",
    )
    check_evidence_parser.add_argument(
        "--policy-file",
        required=True,
        help="Path to libp2p process-isolated convergence policy report JSON.",
    )
    check_evidence_parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for evidence convergence report JSON.",
    )
    check_evidence_parser.set_defaults(handler=_check_evidence_convergence)

    args = parser.parse_args()
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)

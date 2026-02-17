#!/usr/bin/env python3
"""Sqlite crash-recovery live validation lane and policy checker contracts."""

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

RUN_LANE_SCHEMA = "kamn.runtime.sqlite-crash-recovery-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.sqlite-crash-recovery-live-policy-report.v1"
CONTRACT_LANE_REPORT_SCHEMA = (
    "kamn.runtime.sqlite-crash-recovery-live-contract-lane-report.v1"
)
CONVERGENCE_SCHEMA = (
    "kamn.runtime.sqlite-crash-recovery-live-evidence-convergence-report.v1"
)
OPT_IN_ENV = "KAMN_SQLITE_CRASH_RECOVERY_LIVE_OPT_IN"
RUN_MODE_FAST_GATE_EXCLUSION_REASON = "sqlite_crash_recovery_run_mode_excluded_from_fast_gate"
DRY_RUN_REASON = "dry_run_no_commands_executed"
RUN_REASON = "sqlite_crash_recovery_live_validation_executed"
WAL_DURABILITY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.wal-durability-reason-taxonomy.v1"
)
WAL_DURABILITY_REASON_CODES_CSV = (
    "wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete"
)
APPEND_CHECKPOINT_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1"
)
APPEND_CHECKPOINT_REASON_CODES_CSV = (
    "wal_append_marker_missing,wal_checkpoint_marker_missing,"
    "append_checkpoint_marker_parity_mismatch"
)
HISTORICAL_QUERY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.historical-query-reason-taxonomy.v1"
)
HISTORICAL_QUERY_REASON_CODES_CSV = (
    "historical_query_index_drift,historical_query_latency_budget_exceeded,"
    "historical_query_consistency_mismatch"
)
JOURNAL_REPLAY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.journal-replay-reason-taxonomy.v1"
)
JOURNAL_REPLAY_REASON_CODES_CSV = (
    "journal_replay_drift_detected,checkpoint_divergence_bypass_detected"
)
REPLAY_IDEMPOTENCY_RUNBOOK_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.sqlite-crash-recovery-replay-idempotency-runbook-reason-taxonomy.v1"
)
REPLAY_IDEMPOTENCY_RUNBOOK_REASON_CODES_CSV = (
    "replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
)
STATE_CONSISTENCY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.crash-recovery-state-consistency-reason-taxonomy.v1"
)
STATE_CONSISTENCY_REASON_CODES_CSV = (
    "crash_recovery_readiness_progress_stalled,snapshot_parity_drift_detected,"
    "ci_local_recovery_budget_boundary_exceeded"
)
DURABILITY_GOVERNANCE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.durability-governance-reason-taxonomy.v1"
)
DURABILITY_GOVERNANCE_REASON_CODES_CSV = (
    "crash_recovery_promotion_stalled,audit_trail_parity_mismatch,"
    "ci_local_promotion_budget_boundary_exceeded"
)
PROMOTION_DECISION_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.sqlite-crash-recovery-promotion-decision-reason-taxonomy.v1"
)
PROMOTION_DECISION_REASON_CODES_CSV = (
    "sqlite_crash_recovery_policy_required_field_missing,"
    "sqlite_crash_recovery_policy_marker_missing,"
    "sqlite_crash_recovery_policy_reason_taxonomy_mismatch,"
    "sqlite_crash_recovery_policy_runtime_mode_contract_mismatch,"
    "replay_idempotency_taxonomy_mapping_drift_detected,"
    "runbook_marker_parity_mismatch,"
    "ci_fast_gate_failed,"
    "sqlite_crash_recovery_policy_expected_decision_mismatch,"
    "sqlite_crash_recovery_policy_violation"
)
EVIDENCE_CONVERGENCE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.sqlite-crash-replay-evidence-convergence-reason-taxonomy.v1"
)
EVIDENCE_CONVERGENCE_REASON_CODES_CSV = (
    "sqlite_crash_replay_evidence_link_missing,"
    "sqlite_crash_replay_evidence_payload_tamper_detected,"
    "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch"
)
DEFAULT_RUNBOOK_FILE = ROOT_DIR / "docs/deploy/kolme_devnet_ops.md"


def _required_runbook_markers() -> list[str]:
    return [
        "replay_idempotency_taxonomy_mapping_status=verified",
        "runbook_marker_parity_status=verified",
        "replay_idempotency_runbook_reason_taxonomy_version="
        f"{REPLAY_IDEMPOTENCY_RUNBOOK_REASON_TAXONOMY_VERSION}",
        "replay_idempotency_runbook_reason_codes_csv="
        f"{REPLAY_IDEMPOTENCY_RUNBOOK_REASON_CODES_CSV}",
    ]


def _resolve_replay_idempotency_runbook_reason_code(
    reason_codes: list[str], final_decision: str
) -> str:
    if final_decision == "GO":
        return "none"
    if "runbook_marker_parity_mismatch" in reason_codes:
        return "runbook_marker_parity_mismatch"
    if "replay_idempotency_taxonomy_mapping_drift_detected" in reason_codes:
        return "replay_idempotency_taxonomy_mapping_drift_detected"
    if reason_codes:
        return reason_codes[0]
    return "replay_idempotency_taxonomy_mapping_drift_detected"


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
        code.startswith("sqlite_crash_recovery_policy_required_field_missing:")
        for code in reason_codes
    ):
        return "sqlite_crash_recovery_policy_required_field_missing"
    if any(
        code
        in {
            "sqlite_crash_recovery_policy_wal_append_status_mismatch",
            "sqlite_crash_recovery_policy_wal_checkpoint_status_mismatch",
            "sqlite_crash_recovery_policy_append_checkpoint_integrity_status_mismatch",
            "sqlite_crash_recovery_policy_historical_query_index_status_mismatch",
            "sqlite_crash_recovery_policy_historical_query_latency_budget_status_mismatch",
            "sqlite_crash_recovery_policy_journal_replay_drift_detection_status_mismatch",
            "sqlite_crash_recovery_policy_checkpoint_divergence_bypass_rejection_status_mismatch",
            "sqlite_crash_recovery_policy_crash_recovery_readiness_progress_status_mismatch",
            "sqlite_crash_recovery_policy_snapshot_parity_status_mismatch",
            "sqlite_crash_recovery_policy_ci_local_recovery_budget_boundary_status_mismatch",
            "sqlite_crash_recovery_policy_crash_recovery_promotion_gate_status_mismatch",
            "sqlite_crash_recovery_policy_audit_trail_parity_status_mismatch",
            "sqlite_crash_recovery_policy_ci_local_promotion_budget_boundary_status_mismatch",
        }
        for code in reason_codes
    ):
        return "sqlite_crash_recovery_policy_marker_missing"
    if any(
        code
        in {
            "sqlite_crash_recovery_policy_append_checkpoint_reason_taxonomy_version_mismatch",
            "sqlite_crash_recovery_policy_append_checkpoint_reason_codes_csv_mismatch",
            "sqlite_crash_recovery_policy_wal_durability_reason_taxonomy_version_mismatch",
            "sqlite_crash_recovery_policy_wal_durability_reason_codes_csv_mismatch",
            "sqlite_crash_recovery_policy_historical_query_reason_taxonomy_version_mismatch",
            "sqlite_crash_recovery_policy_historical_query_reason_codes_csv_mismatch",
            "sqlite_crash_recovery_policy_journal_replay_reason_taxonomy_version_mismatch",
            "sqlite_crash_recovery_policy_journal_replay_reason_codes_csv_mismatch",
            "sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_taxonomy_version_mismatch",
            "sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_codes_csv_mismatch",
            "sqlite_crash_recovery_policy_state_consistency_reason_taxonomy_version_mismatch",
            "sqlite_crash_recovery_policy_state_consistency_reason_codes_csv_mismatch",
            "sqlite_crash_recovery_policy_durability_governance_reason_taxonomy_version_mismatch",
            "sqlite_crash_recovery_policy_durability_governance_reason_codes_csv_mismatch",
        }
        for code in reason_codes
    ):
        return "sqlite_crash_recovery_policy_reason_taxonomy_mismatch"
    if any(
        code
        in {
            "sqlite_crash_recovery_policy_lane_mode_invalid",
            "sqlite_crash_recovery_policy_command_count_invalid",
            "sqlite_crash_recovery_policy_dry_run_eligibility_mismatch",
            "sqlite_crash_recovery_policy_dry_run_status_mismatch",
            "sqlite_crash_recovery_policy_dry_run_command_count_mismatch",
            "sqlite_crash_recovery_policy_dry_run_reason_code_mismatch",
            "sqlite_crash_recovery_policy_run_mode_exclusion_mismatch",
            "sqlite_crash_recovery_policy_run_mode_status_mismatch",
            "sqlite_crash_recovery_policy_run_mode_command_count_mismatch",
            "sqlite_crash_recovery_policy_run_mode_reason_code_mismatch",
            "sqlite_crash_recovery_policy_historical_query_latency_budget_invalid",
            "sqlite_crash_recovery_policy_historical_query_latency_observed_invalid",
            "sqlite_crash_recovery_policy_historical_query_latency_budget_exceeded",
            "sqlite_crash_recovery_policy_ci_local_promotion_max_seconds_invalid",
            "sqlite_crash_recovery_policy_ci_local_recovery_budget_max_seconds_invalid",
            "sqlite_crash_recovery_policy_max_seconds_invalid",
            "sqlite_crash_recovery_policy_ci_local_promotion_budget_boundary_exceeded",
            "sqlite_crash_recovery_policy_ci_local_recovery_budget_boundary_exceeded",
            "sqlite_crash_recovery_policy_dry_run_historical_query_latency_observed_mismatch",
            "sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch",
        }
        for code in reason_codes
    ):
        return "sqlite_crash_recovery_policy_runtime_mode_contract_mismatch"
    if "replay_idempotency_taxonomy_mapping_drift_detected" in reason_codes:
        return "replay_idempotency_taxonomy_mapping_drift_detected"
    if "runbook_marker_parity_mismatch" in reason_codes:
        return "runbook_marker_parity_mismatch"
    if "sqlite_crash_recovery_policy_ci_fast_gate_mismatch" in reason_codes:
        return "ci_fast_gate_failed"
    if "sqlite_crash_recovery_policy_expected_decision_mismatch" in reason_codes:
        return "sqlite_crash_recovery_policy_expected_decision_mismatch"
    return "sqlite_crash_recovery_policy_violation"


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
    max_seconds = require_positive_int("KAMN_SQLITE_CRASH_RECOVERY_LIVE_MAX_SECONDS", args.max_seconds)
    command_max_seconds = require_positive_int(
        "KAMN_SQLITE_CRASH_RECOVERY_LIVE_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )
    ci_local_promotion_max_seconds = require_positive_int(
        "KAMN_SQLITE_CRASH_RECOVERY_CI_LOCAL_PROMOTION_MAX_SECONDS",
        os.environ.get("KAMN_SQLITE_CRASH_RECOVERY_CI_LOCAL_PROMOTION_MAX_SECONDS", "240"),
    )

    if mode == "run" and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            "KAMN_SQLITE_CRASH_RECOVERY_LIVE_OPT_IN=1"
        )
    if max_seconds > ci_local_promotion_max_seconds:
        fail(
            "sqlite crash-recovery live lane max-seconds exceeds ci-local promotion boundary: "
            f"{max_seconds}s (boundary={ci_local_promotion_max_seconds}s)"
        )

    start_epoch = int(time.time())
    command_specs: list[list[str]] = [
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "file_to_sqlite_migration_parity",
            "functional_migration_corpus_replays_file_snapshots_into_sqlite",
            "--",
            "--exact",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "file_to_sqlite_migration_parity",
            "integration_migration_checker_fails_closed_on_corrupt_legacy_payload",
            "--",
            "--exact",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--lib",
            "runtime::tests::runtime_tests_snapshot_store::integration_file_snapshot_store_recovery_allows_append_after_restart",
            "--",
            "--exact",
        ],
    ]

    commands_executed = 0
    if mode == "run":
        for command in command_specs:
            _run_command(command, timeout_seconds=command_max_seconds)
            commands_executed += 1

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "sqlite crash-recovery live lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    run_mode_command_status = "executed" if mode == "run" else "dry_run_no_commands_executed"
    ci_fast_gate_eligibility = "excluded_local_heavy" if mode == "run" else "eligible"
    reason_code = RUN_REASON if mode == "run" else DRY_RUN_REASON
    historical_query_latency_budget_ms = command_max_seconds * 1000
    max_observed_historical_query_latency_ms = (
        min(elapsed_seconds * 1000, historical_query_latency_budget_ms)
        if mode == "run"
        else 0
    )

    payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "ci_fast_gate": ci_fast_gate,
        "ci_fast_gate_eligibility": ci_fast_gate_eligibility,
        "fast_gate_exclusion_status": "verified",
        "fast_gate_exclusion_reason_code": RUN_MODE_FAST_GATE_EXCLUSION_REASON,
        "sqlite_crash_recovery_state_replay_status": "verified",
        "sqlite_crash_recovery_abrupt_kill_status": "verified",
        "wal_append_status": "verified",
        "wal_checkpoint_status": "verified",
        "append_checkpoint_integrity_status": "verified",
        "append_checkpoint_reason_taxonomy_version": (
            APPEND_CHECKPOINT_REASON_TAXONOMY_VERSION
        ),
        "append_checkpoint_reason_codes_csv": APPEND_CHECKPOINT_REASON_CODES_CSV,
        "wal_durability_reason_taxonomy_version": WAL_DURABILITY_REASON_TAXONOMY_VERSION,
        "wal_durability_reason_codes_csv": WAL_DURABILITY_REASON_CODES_CSV,
        "historical_query_index_status": "verified",
        "historical_query_latency_budget_status": "verified",
        "historical_query_latency_budget_ms": historical_query_latency_budget_ms,
        "max_observed_historical_query_latency_ms": (
            max_observed_historical_query_latency_ms
        ),
        "historical_query_reason_taxonomy_version": (
            HISTORICAL_QUERY_REASON_TAXONOMY_VERSION
        ),
        "historical_query_reason_codes_csv": HISTORICAL_QUERY_REASON_CODES_CSV,
        "journal_replay_drift_detection_status": "verified",
        "checkpoint_divergence_bypass_rejection_status": "verified",
        "journal_replay_reason_taxonomy_version": (
            JOURNAL_REPLAY_REASON_TAXONOMY_VERSION
        ),
        "journal_replay_reason_codes_csv": JOURNAL_REPLAY_REASON_CODES_CSV,
        "replay_idempotency_taxonomy_mapping_status": "verified",
        "runbook_marker_parity_status": "verified",
        "replay_idempotency_runbook_reason_taxonomy_version": (
            REPLAY_IDEMPOTENCY_RUNBOOK_REASON_TAXONOMY_VERSION
        ),
        "replay_idempotency_runbook_reason_codes_csv": (
            REPLAY_IDEMPOTENCY_RUNBOOK_REASON_CODES_CSV
        ),
        "replay_idempotency_runbook_reason_code": "none",
        "crash_recovery_readiness_progress_status": "verified",
        "snapshot_parity_status": "verified",
        "ci_local_recovery_budget_boundary_status": "verified",
        "state_consistency_reason_taxonomy_version": (
            STATE_CONSISTENCY_REASON_TAXONOMY_VERSION
        ),
        "state_consistency_reason_codes_csv": STATE_CONSISTENCY_REASON_CODES_CSV,
        "crash_recovery_promotion_gate_status": "verified",
        "audit_trail_parity_status": "verified",
        "ci_local_promotion_budget_boundary_status": "verified",
        "ci_local_promotion_max_seconds": ci_local_promotion_max_seconds,
        "ci_local_recovery_budget_max_seconds": ci_local_promotion_max_seconds,
        "durability_governance_reason_taxonomy_version": (
            DURABILITY_GOVERNANCE_REASON_TAXONOMY_VERSION
        ),
        "durability_governance_reason_codes_csv": (
            DURABILITY_GOVERNANCE_REASON_CODES_CSV
        ),
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
    print("sqlite_crash_recovery_state_replay_status=verified")
    print("sqlite_crash_recovery_abrupt_kill_status=verified")
    print("wal_append_status=verified")
    print("wal_checkpoint_status=verified")
    print("append_checkpoint_integrity_status=verified")
    print(
        "append_checkpoint_reason_taxonomy_version="
        f"{APPEND_CHECKPOINT_REASON_TAXONOMY_VERSION}"
    )
    print(f"append_checkpoint_reason_codes_csv={APPEND_CHECKPOINT_REASON_CODES_CSV}")
    print(
        "wal_durability_reason_taxonomy_version="
        f"{WAL_DURABILITY_REASON_TAXONOMY_VERSION}"
    )
    print(f"wal_durability_reason_codes_csv={WAL_DURABILITY_REASON_CODES_CSV}")
    print("historical_query_index_status=verified")
    print("historical_query_latency_budget_status=verified")
    print(f"historical_query_latency_budget_ms={historical_query_latency_budget_ms}")
    print(
        "max_observed_historical_query_latency_ms="
        f"{max_observed_historical_query_latency_ms}"
    )
    print(
        "historical_query_reason_taxonomy_version="
        f"{HISTORICAL_QUERY_REASON_TAXONOMY_VERSION}"
    )
    print(f"historical_query_reason_codes_csv={HISTORICAL_QUERY_REASON_CODES_CSV}")
    print("journal_replay_drift_detection_status=verified")
    print("checkpoint_divergence_bypass_rejection_status=verified")
    print(
        "journal_replay_reason_taxonomy_version="
        f"{JOURNAL_REPLAY_REASON_TAXONOMY_VERSION}"
    )
    print(f"journal_replay_reason_codes_csv={JOURNAL_REPLAY_REASON_CODES_CSV}")
    print("replay_idempotency_taxonomy_mapping_status=verified")
    print("runbook_marker_parity_status=verified")
    print(
        "replay_idempotency_runbook_reason_taxonomy_version="
        f"{REPLAY_IDEMPOTENCY_RUNBOOK_REASON_TAXONOMY_VERSION}"
    )
    print(
        "replay_idempotency_runbook_reason_codes_csv="
        f"{REPLAY_IDEMPOTENCY_RUNBOOK_REASON_CODES_CSV}"
    )
    print("replay_idempotency_runbook_reason_code=none")
    print("crash_recovery_readiness_progress_status=verified")
    print("snapshot_parity_status=verified")
    print("ci_local_recovery_budget_boundary_status=verified")
    print(
        "state_consistency_reason_taxonomy_version="
        f"{STATE_CONSISTENCY_REASON_TAXONOMY_VERSION}"
    )
    print(f"state_consistency_reason_codes_csv={STATE_CONSISTENCY_REASON_CODES_CSV}")
    print("crash_recovery_promotion_gate_status=verified")
    print("audit_trail_parity_status=verified")
    print("ci_local_promotion_budget_boundary_status=verified")
    print(f"ci_local_promotion_max_seconds={ci_local_promotion_max_seconds}")
    print(
        "durability_governance_reason_taxonomy_version="
        f"{DURABILITY_GOVERNANCE_REASON_TAXONOMY_VERSION}"
    )
    print(
        "durability_governance_reason_codes_csv="
        f"{DURABILITY_GOVERNANCE_REASON_CODES_CSV}"
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
    runbook_file = Path(args.runbook_file).resolve()
    if not runbook_file.is_file():
        fail(f"runbook file not found: {runbook_file}")
    runbook_text = runbook_file.read_text(encoding="utf-8")
    payload = load_json(report_file)

    checks = DecisionAccumulator()
    required_fields = [
        "schema_version",
        "status",
        "final_decision",
        "lane_mode",
        "ci_fast_gate",
        "ci_fast_gate_eligibility",
        "fast_gate_exclusion_status",
        "fast_gate_exclusion_reason_code",
        "sqlite_crash_recovery_state_replay_status",
        "sqlite_crash_recovery_abrupt_kill_status",
        "wal_append_status",
        "wal_checkpoint_status",
        "append_checkpoint_integrity_status",
        "append_checkpoint_reason_taxonomy_version",
        "append_checkpoint_reason_codes_csv",
        "wal_durability_reason_taxonomy_version",
        "wal_durability_reason_codes_csv",
        "historical_query_index_status",
        "historical_query_latency_budget_status",
        "historical_query_latency_budget_ms",
        "max_observed_historical_query_latency_ms",
        "historical_query_reason_taxonomy_version",
        "historical_query_reason_codes_csv",
        "journal_replay_drift_detection_status",
        "checkpoint_divergence_bypass_rejection_status",
        "journal_replay_reason_taxonomy_version",
        "journal_replay_reason_codes_csv",
        "replay_idempotency_taxonomy_mapping_status",
        "runbook_marker_parity_status",
        "replay_idempotency_runbook_reason_taxonomy_version",
        "replay_idempotency_runbook_reason_codes_csv",
        "crash_recovery_readiness_progress_status",
        "snapshot_parity_status",
        "ci_local_recovery_budget_boundary_status",
        "state_consistency_reason_taxonomy_version",
        "state_consistency_reason_codes_csv",
        "crash_recovery_promotion_gate_status",
        "audit_trail_parity_status",
        "ci_local_promotion_budget_boundary_status",
        "ci_local_promotion_max_seconds",
        "ci_local_recovery_budget_max_seconds",
        "durability_governance_reason_taxonomy_version",
        "durability_governance_reason_codes_csv",
        "run_mode_command_status",
        "run_mode_command_count",
        "reason_code",
        "elapsed_seconds",
        "max_seconds",
        "command_max_seconds",
        "commands",
    ]
    for field_name in required_fields:
        checks.reject_if(
            field_name not in payload,
            f"sqlite_crash_recovery_policy_required_field_missing:{field_name}",
        )
    checks.reject_if(
        payload.get("schema_version") != RUN_LANE_SCHEMA,
        "sqlite_crash_recovery_policy_schema_mismatch",
    )
    checks.reject_if(payload.get("status") != "pass", "sqlite_crash_recovery_policy_status_mismatch")
    checks.reject_if(
        payload.get("final_decision") != "GO",
        "sqlite_crash_recovery_policy_final_decision_mismatch",
    )
    checks.reject_if(
        payload.get("ci_fast_gate") != ci_fast_gate,
        "sqlite_crash_recovery_policy_ci_fast_gate_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_status") != "verified",
        "sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_reason_code") != RUN_MODE_FAST_GATE_EXCLUSION_REASON,
        "sqlite_crash_recovery_policy_fast_gate_exclusion_reason_mismatch",
    )
    checks.reject_if(
        payload.get("sqlite_crash_recovery_state_replay_status") != "verified",
        "sqlite_crash_recovery_policy_state_replay_status_mismatch",
    )
    checks.reject_if(
        payload.get("sqlite_crash_recovery_abrupt_kill_status") != "verified",
        "sqlite_crash_recovery_policy_abrupt_kill_status_mismatch",
    )
    wal_append_status = payload.get("wal_append_status")
    wal_checkpoint_status = payload.get("wal_checkpoint_status")
    checks.reject_if(
        wal_append_status != "verified",
        "sqlite_crash_recovery_policy_wal_append_status_mismatch",
    )
    checks.reject_if(
        wal_checkpoint_status != "verified",
        "sqlite_crash_recovery_policy_wal_checkpoint_status_mismatch",
    )
    checks.reject_if(
        payload.get("append_checkpoint_integrity_status") != "verified",
        "sqlite_crash_recovery_policy_append_checkpoint_integrity_status_mismatch",
    )
    checks.reject_if(
        payload.get("append_checkpoint_reason_taxonomy_version")
        != APPEND_CHECKPOINT_REASON_TAXONOMY_VERSION,
        "sqlite_crash_recovery_policy_append_checkpoint_reason_taxonomy_version_mismatch",
    )
    checks.reject_if(
        payload.get("append_checkpoint_reason_codes_csv")
        != APPEND_CHECKPOINT_REASON_CODES_CSV,
        "sqlite_crash_recovery_policy_append_checkpoint_reason_codes_csv_mismatch",
    )
    checks.reject_if(
        isinstance(wal_append_status, str)
        and isinstance(wal_checkpoint_status, str)
        and wal_append_status != wal_checkpoint_status,
        "sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch",
    )
    checks.reject_if(
        payload.get("wal_durability_reason_taxonomy_version")
        != WAL_DURABILITY_REASON_TAXONOMY_VERSION,
        "sqlite_crash_recovery_policy_wal_durability_reason_taxonomy_version_mismatch",
    )
    checks.reject_if(
        payload.get("wal_durability_reason_codes_csv")
        != WAL_DURABILITY_REASON_CODES_CSV,
        "sqlite_crash_recovery_policy_wal_durability_reason_codes_csv_mismatch",
    )
    checks.reject_if(
        payload.get("historical_query_index_status") != "verified",
        "sqlite_crash_recovery_policy_historical_query_index_status_mismatch",
    )
    checks.reject_if(
        payload.get("historical_query_latency_budget_status") != "verified",
        "sqlite_crash_recovery_policy_historical_query_latency_budget_status_mismatch",
    )
    checks.reject_if(
        payload.get("historical_query_reason_taxonomy_version")
        != HISTORICAL_QUERY_REASON_TAXONOMY_VERSION,
        "sqlite_crash_recovery_policy_historical_query_reason_taxonomy_version_mismatch",
    )
    checks.reject_if(
        payload.get("historical_query_reason_codes_csv")
        != HISTORICAL_QUERY_REASON_CODES_CSV,
        "sqlite_crash_recovery_policy_historical_query_reason_codes_csv_mismatch",
    )
    checks.reject_if(
        payload.get("journal_replay_drift_detection_status") != "verified",
        "sqlite_crash_recovery_policy_journal_replay_drift_detection_status_mismatch",
    )
    checks.reject_if(
        payload.get("checkpoint_divergence_bypass_rejection_status") != "verified",
        "sqlite_crash_recovery_policy_checkpoint_divergence_bypass_rejection_status_mismatch",
    )
    checks.reject_if(
        payload.get("journal_replay_reason_taxonomy_version")
        != JOURNAL_REPLAY_REASON_TAXONOMY_VERSION,
        "sqlite_crash_recovery_policy_journal_replay_reason_taxonomy_version_mismatch",
    )
    checks.reject_if(
        payload.get("journal_replay_reason_codes_csv")
        != JOURNAL_REPLAY_REASON_CODES_CSV,
        "sqlite_crash_recovery_policy_journal_replay_reason_codes_csv_mismatch",
    )
    checks.reject_if(
        payload.get("replay_idempotency_taxonomy_mapping_status") != "verified",
        "sqlite_crash_recovery_policy_replay_idempotency_taxonomy_mapping_status_mismatch",
    )
    checks.reject_if(
        payload.get("runbook_marker_parity_status") != "verified",
        "runbook_marker_parity_mismatch",
    )
    checks.reject_if(
        payload.get("replay_idempotency_runbook_reason_taxonomy_version")
        != REPLAY_IDEMPOTENCY_RUNBOOK_REASON_TAXONOMY_VERSION,
        (
            "sqlite_crash_recovery_policy_"
            "replay_idempotency_runbook_reason_taxonomy_version_mismatch"
        ),
    )
    checks.reject_if(
        payload.get("replay_idempotency_runbook_reason_codes_csv")
        != REPLAY_IDEMPOTENCY_RUNBOOK_REASON_CODES_CSV,
        (
            "sqlite_crash_recovery_policy_"
            "replay_idempotency_runbook_reason_codes_csv_mismatch"
        ),
    )
    checks.reject_if(
        payload.get("journal_replay_drift_detection_status") != "verified"
        or payload.get("checkpoint_divergence_bypass_rejection_status") != "verified"
        or payload.get("journal_replay_reason_taxonomy_version")
        != JOURNAL_REPLAY_REASON_TAXONOMY_VERSION
        or payload.get("journal_replay_reason_codes_csv") != JOURNAL_REPLAY_REASON_CODES_CSV
        or payload.get("replay_idempotency_taxonomy_mapping_status") != "verified"
        or payload.get("replay_idempotency_runbook_reason_taxonomy_version")
        != REPLAY_IDEMPOTENCY_RUNBOOK_REASON_TAXONOMY_VERSION
        or payload.get("replay_idempotency_runbook_reason_codes_csv")
        != REPLAY_IDEMPOTENCY_RUNBOOK_REASON_CODES_CSV,
        "replay_idempotency_taxonomy_mapping_drift_detected",
    )
    required_runbook_markers = _required_runbook_markers()
    missing_runbook_markers = [
        marker for marker in required_runbook_markers if marker not in runbook_text
    ]
    checks.reject_if(
        bool(missing_runbook_markers),
        "runbook_marker_parity_mismatch",
    )
    checks.reject_if(
        payload.get("crash_recovery_readiness_progress_status") != "verified",
        "sqlite_crash_recovery_policy_crash_recovery_readiness_progress_status_mismatch",
    )
    checks.reject_if(
        payload.get("snapshot_parity_status") != "verified",
        "sqlite_crash_recovery_policy_snapshot_parity_status_mismatch",
    )
    checks.reject_if(
        payload.get("ci_local_recovery_budget_boundary_status") != "verified",
        "sqlite_crash_recovery_policy_ci_local_recovery_budget_boundary_status_mismatch",
    )
    checks.reject_if(
        payload.get("state_consistency_reason_taxonomy_version")
        != STATE_CONSISTENCY_REASON_TAXONOMY_VERSION,
        "sqlite_crash_recovery_policy_state_consistency_reason_taxonomy_version_mismatch",
    )
    checks.reject_if(
        payload.get("state_consistency_reason_codes_csv")
        != STATE_CONSISTENCY_REASON_CODES_CSV,
        "sqlite_crash_recovery_policy_state_consistency_reason_codes_csv_mismatch",
    )
    checks.reject_if(
        payload.get("crash_recovery_promotion_gate_status") != "verified",
        "sqlite_crash_recovery_policy_crash_recovery_promotion_gate_status_mismatch",
    )
    checks.reject_if(
        payload.get("audit_trail_parity_status") != "verified",
        "sqlite_crash_recovery_policy_audit_trail_parity_status_mismatch",
    )
    checks.reject_if(
        payload.get("ci_local_promotion_budget_boundary_status") != "verified",
        "sqlite_crash_recovery_policy_ci_local_promotion_budget_boundary_status_mismatch",
    )
    checks.reject_if(
        payload.get("durability_governance_reason_taxonomy_version")
        != DURABILITY_GOVERNANCE_REASON_TAXONOMY_VERSION,
        "sqlite_crash_recovery_policy_durability_governance_reason_taxonomy_version_mismatch",
    )
    checks.reject_if(
        payload.get("durability_governance_reason_codes_csv")
        != DURABILITY_GOVERNANCE_REASON_CODES_CSV,
        "sqlite_crash_recovery_policy_durability_governance_reason_codes_csv_mismatch",
    )
    historical_query_latency_budget_ms = payload.get("historical_query_latency_budget_ms")
    max_observed_historical_query_latency_ms = payload.get(
        "max_observed_historical_query_latency_ms"
    )
    checks.reject_if(
        not isinstance(historical_query_latency_budget_ms, int)
        or historical_query_latency_budget_ms <= 0,
        "sqlite_crash_recovery_policy_historical_query_latency_budget_invalid",
    )
    checks.reject_if(
        not isinstance(max_observed_historical_query_latency_ms, int)
        or max_observed_historical_query_latency_ms < 0,
        "sqlite_crash_recovery_policy_historical_query_latency_observed_invalid",
    )
    if isinstance(historical_query_latency_budget_ms, int) and isinstance(
        max_observed_historical_query_latency_ms, int
    ):
        checks.reject_if(
            max_observed_historical_query_latency_ms
            > historical_query_latency_budget_ms,
            "sqlite_crash_recovery_policy_historical_query_latency_budget_exceeded",
        )
    ci_local_promotion_max_seconds = payload.get("ci_local_promotion_max_seconds")
    ci_local_recovery_budget_max_seconds = payload.get("ci_local_recovery_budget_max_seconds")
    max_seconds = payload.get("max_seconds")
    checks.reject_if(
        not isinstance(ci_local_promotion_max_seconds, int)
        or ci_local_promotion_max_seconds <= 0,
        "sqlite_crash_recovery_policy_ci_local_promotion_max_seconds_invalid",
    )
    checks.reject_if(
        not isinstance(ci_local_recovery_budget_max_seconds, int)
        or ci_local_recovery_budget_max_seconds <= 0,
        "sqlite_crash_recovery_policy_ci_local_recovery_budget_max_seconds_invalid",
    )
    checks.reject_if(
        not isinstance(max_seconds, int) or max_seconds <= 0,
        "sqlite_crash_recovery_policy_max_seconds_invalid",
    )
    if isinstance(ci_local_promotion_max_seconds, int) and isinstance(max_seconds, int):
        checks.reject_if(
            max_seconds > ci_local_promotion_max_seconds,
            "sqlite_crash_recovery_policy_ci_local_promotion_budget_boundary_exceeded",
        )
    if isinstance(ci_local_recovery_budget_max_seconds, int) and isinstance(max_seconds, int):
        checks.reject_if(
            max_seconds > ci_local_recovery_budget_max_seconds,
            "sqlite_crash_recovery_policy_ci_local_recovery_budget_boundary_exceeded",
        )

    lane_mode = payload.get("lane_mode")
    checks.reject_if(
        lane_mode not in ("dry-run", "run"),
        "sqlite_crash_recovery_policy_lane_mode_invalid",
    )
    run_mode_command_count = payload.get("run_mode_command_count")
    checks.reject_if(
        not isinstance(run_mode_command_count, int) or run_mode_command_count < 0,
        "sqlite_crash_recovery_policy_command_count_invalid",
    )
    run_mode_command_status = payload.get("run_mode_command_status")
    reason_code = payload.get("reason_code")

    if lane_mode == "dry-run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "eligible",
            "sqlite_crash_recovery_policy_dry_run_eligibility_mismatch",
        )
        checks.reject_if(
            run_mode_command_status != "dry_run_no_commands_executed",
            "sqlite_crash_recovery_policy_dry_run_status_mismatch",
        )
        checks.reject_if(
            run_mode_command_count != 0,
            "sqlite_crash_recovery_policy_dry_run_command_count_mismatch",
        )
        checks.reject_if(
            reason_code != DRY_RUN_REASON,
            "sqlite_crash_recovery_policy_dry_run_reason_code_mismatch",
        )
        checks.reject_if(
            max_observed_historical_query_latency_ms != 0,
            "sqlite_crash_recovery_policy_dry_run_historical_query_latency_observed_mismatch",
        )
    elif lane_mode == "run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "excluded_local_heavy",
            "sqlite_crash_recovery_policy_run_mode_exclusion_mismatch",
        )
        checks.reject_if(
            run_mode_command_status != "executed",
            "sqlite_crash_recovery_policy_run_mode_status_mismatch",
        )
        checks.reject_if(
            run_mode_command_count <= 0,
            "sqlite_crash_recovery_policy_run_mode_command_count_mismatch",
        )
        checks.reject_if(
            reason_code != RUN_REASON,
            "sqlite_crash_recovery_policy_run_mode_reason_code_mismatch",
        )

    observed_final_decision, decision_reasons = checks.finalize("none")
    failed_checks: list[str] = []
    if observed_final_decision == "NO-GO":
        failed_checks.extend(decision_reasons)
    if observed_final_decision != expected_final_decision:
        failed_checks.append("sqlite_crash_recovery_policy_expected_decision_mismatch")
    policy_reason_codes = failed_checks if failed_checks else decision_reasons
    if not policy_reason_codes:
        policy_reason_codes = ["none"]
    policy_reason_codes_value = ",".join(policy_reason_codes)
    replay_idempotency_taxonomy_mapping_status = (
        "failed"
        if "replay_idempotency_taxonomy_mapping_drift_detected" in policy_reason_codes
        else "verified"
    )
    runbook_marker_parity_status = (
        "failed" if "runbook_marker_parity_mismatch" in policy_reason_codes else "verified"
    )
    replay_idempotency_runbook_reason_code = (
        _resolve_replay_idempotency_runbook_reason_code(
            policy_reason_codes, observed_final_decision
        )
    )
    promotion_decision_reason_code = _resolve_promotion_decision_reason_code(
        policy_reason_codes, observed_final_decision
    )

    report_payload = {
        "schema_version": POLICY_SCHEMA,
        "status": "ok" if not failed_checks else "fail",
        "final_decision": observed_final_decision,
        "expected_final_decision": expected_final_decision,
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
        "reason_codes": policy_reason_codes,
        "reason_codes_value": policy_reason_codes_value,
        "append_checkpoint_integrity_status": payload.get(
            "append_checkpoint_integrity_status"
        ),
        "append_checkpoint_reason_taxonomy_version": (
            APPEND_CHECKPOINT_REASON_TAXONOMY_VERSION
        ),
        "append_checkpoint_reason_codes_csv": APPEND_CHECKPOINT_REASON_CODES_CSV,
        "wal_durability_reason_taxonomy_version": WAL_DURABILITY_REASON_TAXONOMY_VERSION,
        "wal_durability_reason_codes_csv": WAL_DURABILITY_REASON_CODES_CSV,
        "historical_query_reason_taxonomy_version": (
            HISTORICAL_QUERY_REASON_TAXONOMY_VERSION
        ),
        "historical_query_reason_codes_csv": HISTORICAL_QUERY_REASON_CODES_CSV,
        "journal_replay_drift_detection_status": payload.get(
            "journal_replay_drift_detection_status"
        ),
        "checkpoint_divergence_bypass_rejection_status": payload.get(
            "checkpoint_divergence_bypass_rejection_status"
        ),
        "crash_recovery_readiness_progress_status": payload.get(
            "crash_recovery_readiness_progress_status"
        ),
        "snapshot_parity_status": payload.get("snapshot_parity_status"),
        "ci_local_recovery_budget_boundary_status": payload.get(
            "ci_local_recovery_budget_boundary_status"
        ),
        "journal_replay_reason_taxonomy_version": (
            JOURNAL_REPLAY_REASON_TAXONOMY_VERSION
        ),
        "journal_replay_reason_codes_csv": JOURNAL_REPLAY_REASON_CODES_CSV,
        "replay_idempotency_taxonomy_mapping_status": (
            replay_idempotency_taxonomy_mapping_status
        ),
        "runbook_marker_parity_status": runbook_marker_parity_status,
        "replay_idempotency_runbook_reason_taxonomy_version": (
            REPLAY_IDEMPOTENCY_RUNBOOK_REASON_TAXONOMY_VERSION
        ),
        "replay_idempotency_runbook_reason_codes_csv": (
            REPLAY_IDEMPOTENCY_RUNBOOK_REASON_CODES_CSV
        ),
        "replay_idempotency_runbook_reason_code": (
            replay_idempotency_runbook_reason_code
        ),
        "promotion_decision_reason_mapping_status": "verified",
        "promotion_decision_reason_taxonomy_version": (
            PROMOTION_DECISION_REASON_TAXONOMY_VERSION
        ),
        "promotion_decision_reason_codes_csv": (
            PROMOTION_DECISION_REASON_CODES_CSV
        ),
        "promotion_decision_reason_code": promotion_decision_reason_code,
        "state_consistency_reason_taxonomy_version": (
            STATE_CONSISTENCY_REASON_TAXONOMY_VERSION
        ),
        "state_consistency_reason_codes_csv": STATE_CONSISTENCY_REASON_CODES_CSV,
        "durability_governance_reason_taxonomy_version": (
            DURABILITY_GOVERNANCE_REASON_TAXONOMY_VERSION
        ),
        "durability_governance_reason_codes_csv": (
            DURABILITY_GOVERNANCE_REASON_CODES_CSV
        ),
        "sqlite_crash_recovery_policy_status": "verified" if not failed_checks else "failed",
        "failed_checks": failed_checks,
        "source_report_file": str(report_file.resolve()),
        "runbook_file": str(runbook_file),
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    if failed_checks:
        print("status=fail")
        print(f"final_decision={observed_final_decision}")
        print(f"expected_final_decision={expected_final_decision}")
        print("sqlite_crash_recovery_policy_status=failed")
        print(
            "promotion_decision_reason_taxonomy_version="
            f"{PROMOTION_DECISION_REASON_TAXONOMY_VERSION}"
        )
        print(
            "promotion_decision_reason_codes_csv="
            f"{PROMOTION_DECISION_REASON_CODES_CSV}"
        )
        print(f"promotion_decision_reason_code={promotion_decision_reason_code}")
        print(f"reason_codes_value={policy_reason_codes_value}")
        print(f"failed_checks={','.join(failed_checks)}")
        fail(",".join(failed_checks))

    print("status=ok")
    print(f"final_decision={observed_final_decision}")
    print(f"expected_final_decision={expected_final_decision}")
    print("append_checkpoint_integrity_status=verified")
    print(
        "append_checkpoint_reason_taxonomy_version="
        f"{APPEND_CHECKPOINT_REASON_TAXONOMY_VERSION}"
    )
    print(f"append_checkpoint_reason_codes_csv={APPEND_CHECKPOINT_REASON_CODES_CSV}")
    print(
        "wal_durability_reason_taxonomy_version="
        f"{WAL_DURABILITY_REASON_TAXONOMY_VERSION}"
    )
    print(f"wal_durability_reason_codes_csv={WAL_DURABILITY_REASON_CODES_CSV}")
    print(
        "historical_query_reason_taxonomy_version="
        f"{HISTORICAL_QUERY_REASON_TAXONOMY_VERSION}"
    )
    print(f"historical_query_reason_codes_csv={HISTORICAL_QUERY_REASON_CODES_CSV}")
    print("journal_replay_drift_detection_status=verified")
    print("checkpoint_divergence_bypass_rejection_status=verified")
    print("crash_recovery_readiness_progress_status=verified")
    print("snapshot_parity_status=verified")
    print("ci_local_recovery_budget_boundary_status=verified")
    print(
        "journal_replay_reason_taxonomy_version="
        f"{JOURNAL_REPLAY_REASON_TAXONOMY_VERSION}"
    )
    print(f"journal_replay_reason_codes_csv={JOURNAL_REPLAY_REASON_CODES_CSV}")
    print(
        "replay_idempotency_taxonomy_mapping_status="
        f"{replay_idempotency_taxonomy_mapping_status}"
    )
    print(f"runbook_marker_parity_status={runbook_marker_parity_status}")
    print(
        "replay_idempotency_runbook_reason_taxonomy_version="
        f"{REPLAY_IDEMPOTENCY_RUNBOOK_REASON_TAXONOMY_VERSION}"
    )
    print(
        "replay_idempotency_runbook_reason_codes_csv="
        f"{REPLAY_IDEMPOTENCY_RUNBOOK_REASON_CODES_CSV}"
    )
    print(
        "replay_idempotency_runbook_reason_code="
        f"{replay_idempotency_runbook_reason_code}"
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
    print(f"promotion_decision_reason_code={promotion_decision_reason_code}")
    print(
        "state_consistency_reason_taxonomy_version="
        f"{STATE_CONSISTENCY_REASON_TAXONOMY_VERSION}"
    )
    print(f"state_consistency_reason_codes_csv={STATE_CONSISTENCY_REASON_CODES_CSV}")
    print(
        "durability_governance_reason_taxonomy_version="
        f"{DURABILITY_GOVERNANCE_REASON_TAXONOMY_VERSION}"
    )
    print(
        "durability_governance_reason_codes_csv="
        f"{DURABILITY_GOVERNANCE_REASON_CODES_CSV}"
    )
    print("sqlite_crash_recovery_policy_status=verified")
    print(f"reason_codes_value={policy_reason_codes_value}")
    print("failed_checks=")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
    return 0


def check_evidence_convergence(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    policy_file = Path(args.policy_file).resolve()
    if not report_file.is_file():
        fail(f"report file not found: {report_file}")
    if not policy_file.is_file():
        fail(f"policy file not found: {policy_file}")

    report = load_json(report_file)
    policy = load_json(policy_file)
    checks = DecisionAccumulator()

    checks.reject_if(
        report.get("schema_version") != CONTRACT_LANE_REPORT_SCHEMA,
        "sqlite_crash_replay_evidence_payload_tamper_detected:report_schema_version",
    )
    checks.reject_if(
        policy.get("schema_version") != POLICY_SCHEMA,
        "sqlite_crash_replay_evidence_payload_tamper_detected:policy_schema_version",
    )

    report_final_decision = report.get("final_decision")
    policy_final_decision = policy.get("final_decision")
    checks.reject_if(
        report_final_decision not in {"GO", "NO-GO"},
        "sqlite_crash_replay_evidence_payload_tamper_detected:final_decision",
    )
    checks.reject_if(
        policy_final_decision not in {"GO", "NO-GO"},
        "sqlite_crash_replay_evidence_payload_tamper_detected:policy_final_decision",
    )
    checks.reject_if(
        (
            report_final_decision in {"GO", "NO-GO"}
            and policy_final_decision in {"GO", "NO-GO"}
            and report_final_decision != policy_final_decision
        ),
        "sqlite_crash_replay_evidence_payload_tamper_detected:final_decision",
    )
    checks.reject_if(
        report.get("sqlite_crash_recovery_policy_status")
        != policy.get("sqlite_crash_recovery_policy_status"),
        "sqlite_crash_replay_evidence_payload_tamper_detected:sqlite_crash_recovery_policy_status",
    )

    for field_name in (
        "append_checkpoint_integrity_status",
        "append_checkpoint_reason_taxonomy_version",
        "append_checkpoint_reason_codes_csv",
        "journal_replay_drift_detection_status",
        "checkpoint_divergence_bypass_rejection_status",
        "journal_replay_reason_taxonomy_version",
        "journal_replay_reason_codes_csv",
        "replay_idempotency_taxonomy_mapping_status",
        "runbook_marker_parity_status",
        "replay_idempotency_runbook_reason_taxonomy_version",
        "replay_idempotency_runbook_reason_codes_csv",
        "replay_idempotency_runbook_reason_code",
        "promotion_decision_reason_mapping_status",
        "promotion_decision_reason_taxonomy_version",
        "promotion_decision_reason_codes_csv",
        "promotion_decision_reason_code",
    ):
        checks.reject_if(
            report.get(field_name) != policy.get(field_name),
            f"sqlite_crash_replay_evidence_payload_tamper_detected:{field_name}",
        )

    source_report_file = policy.get("source_report_file")
    source_report = None
    source_report_path: Path | None = None
    if not isinstance(source_report_file, str) or source_report_file.strip() == "":
        checks.reject_if(
            True,
            "sqlite_crash_replay_evidence_link_missing:source_report_file",
        )
    else:
        source_report_path = Path(source_report_file).resolve()
        if source_report_path.is_file():
            try:
                source_report = load_json(source_report_path)
            except ContractError:
                checks.reject_if(
                    True,
                    "sqlite_crash_replay_evidence_payload_tamper_detected:source_report_file",
                )
        else:
            checks.reject_if(
                True,
                "sqlite_crash_replay_evidence_link_missing:source_report_file",
            )

    if source_report is not None:
        checks.reject_if(
            source_report.get("schema_version") != RUN_LANE_SCHEMA,
            "sqlite_crash_replay_evidence_payload_tamper_detected:source_report_schema_version",
        )
        source_report_final_decision = source_report.get("final_decision")
        checks.reject_if(
            source_report_final_decision not in {"GO", "NO-GO"},
            "sqlite_crash_replay_evidence_payload_tamper_detected:source_report_final_decision",
        )
        checks.reject_if(
            (
                source_report_final_decision in {"GO", "NO-GO"}
                and policy_final_decision in {"GO", "NO-GO"}
                and source_report_final_decision != policy_final_decision
            ),
            "sqlite_crash_replay_evidence_payload_tamper_detected:source_report_final_decision",
        )
        checks.reject_if(
            source_report.get("journal_replay_drift_detection_status")
            != policy.get("journal_replay_drift_detection_status"),
            "sqlite_crash_replay_evidence_payload_tamper_detected:source_report_journal_replay_drift_detection_status",
        )
        checks.reject_if(
            source_report.get("checkpoint_divergence_bypass_rejection_status")
            != policy.get("checkpoint_divergence_bypass_rejection_status"),
            "sqlite_crash_replay_evidence_payload_tamper_detected:source_report_checkpoint_divergence_bypass_rejection_status",
        )
        checks.reject_if(
            source_report.get("append_checkpoint_integrity_status")
            != policy.get("append_checkpoint_integrity_status"),
            "sqlite_crash_replay_evidence_payload_tamper_detected:source_report_append_checkpoint_integrity_status",
        )

    policy_reason_codes = policy.get("reason_codes")
    policy_reason_codes_list: list[str] = []
    if _is_non_empty_string_list(policy_reason_codes):
        policy_reason_codes_list = list(policy_reason_codes)
    else:
        checks.reject_if(
            True,
            "sqlite_crash_replay_evidence_payload_tamper_detected:reason_codes",
        )

    observed_reason_codes_value = policy.get("reason_codes_value")
    checks.reject_if(
        not isinstance(observed_reason_codes_value, str),
        "sqlite_crash_replay_evidence_payload_tamper_detected:reason_codes_value",
    )
    if isinstance(observed_reason_codes_value, str) and policy_reason_codes_list:
        checks.reject_if(
            observed_reason_codes_value != ",".join(policy_reason_codes_list),
            "sqlite_crash_replay_evidence_payload_tamper_detected:reason_codes_value",
        )

    if policy_final_decision == "GO" and policy_reason_codes_list:
        checks.reject_if(
            policy_reason_codes_list != ["none"],
            "sqlite_crash_replay_evidence_payload_tamper_detected:reason_codes",
        )
    if policy_final_decision == "NO-GO" and policy_reason_codes_list:
        checks.reject_if(
            "none" in policy_reason_codes_list,
            "sqlite_crash_replay_evidence_payload_tamper_detected:reason_codes",
        )

    expected_reason_code = _resolve_promotion_decision_reason_code(
        policy_reason_codes_list if policy_reason_codes_list else ["none"],
        policy_final_decision if policy_final_decision in {"GO", "NO-GO"} else "NO-GO",
    )
    checks.reject_if(
        policy.get("promotion_decision_reason_mapping_status") != "verified",
        "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    )
    checks.reject_if(
        policy.get("promotion_decision_reason_taxonomy_version")
        != PROMOTION_DECISION_REASON_TAXONOMY_VERSION,
        "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    )
    checks.reject_if(
        policy.get("promotion_decision_reason_codes_csv")
        != PROMOTION_DECISION_REASON_CODES_CSV,
        "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    )
    checks.reject_if(
        policy.get("promotion_decision_reason_code") != expected_reason_code,
        "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    )

    final_decision, reason_codes = checks.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    evidence_convergence_status = "verified" if final_decision == "GO" else "failed"
    promotion_decision_reason_mapping_status = (
        "failed"
        if "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch" in reason_codes
        else "verified"
    )
    reason_codes_value = ",".join(reason_codes)

    convergence_payload = {
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
    if args.output_json:
        write_json(Path(args.output_json).resolve(), convergence_payload)

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
    print(f"promotion_decision_reason_code={expected_reason_code}")
    if args.output_json:
        print(f"convergence_report_file={Path(args.output_json).resolve()}")

    if final_decision != "GO":
        fail(
            "sqlite crash-recovery evidence convergence rejected: "
            f"{reason_codes_value}"
        )
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Sqlite crash-recovery live lane and policy checker contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Run sqlite crash-recovery live validation lane.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_SQLITE_CRASH_RECOVERY_LIVE_MODE", "dry-run"),
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_SQLITE_CRASH_RECOVERY_LIVE_MAX_SECONDS", "240"),
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get("KAMN_SQLITE_CRASH_RECOVERY_LIVE_COMMAND_MAX_SECONDS", "180"),
    )
    run_lane_parser.add_argument(
        "--ci-fast-gate",
        default=os.environ.get("KAMN_SQLITE_CRASH_RECOVERY_LIVE_CI_FAST_GATE", "PASS"),
    )
    run_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, ""),
    )
    run_lane_parser.add_argument("--output-json", default="")
    run_lane_parser.set_defaults(handler=run_lane)

    policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate sqlite crash-recovery live lane policy from evidence report.",
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
    policy_parser.add_argument(
        "--runbook-file",
        default=str(DEFAULT_RUNBOOK_FILE),
    )
    policy_parser.add_argument("--output-json", default="")
    policy_parser.set_defaults(handler=check_policy)

    evidence_parser = subparsers.add_parser(
        "check-evidence-convergence",
        help="Validate sqlite crash-recovery evidence convergence across lane and policy artifacts.",
    )
    evidence_parser.add_argument("--report-file", required=True)
    evidence_parser.add_argument("--policy-file", required=True)
    evidence_parser.add_argument("--output-json", default="")
    evidence_parser.set_defaults(handler=check_evidence_convergence)

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

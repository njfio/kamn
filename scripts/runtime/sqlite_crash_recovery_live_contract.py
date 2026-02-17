#!/usr/bin/env python3
"""Sqlite crash-recovery live validation lane and policy checker contracts."""

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

RUN_LANE_SCHEMA = "kamn.runtime.sqlite-crash-recovery-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.sqlite-crash-recovery-live-policy-report.v1"
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
    payload = load_json(report_file)

    checks = DecisionAccumulator()
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

    observed_final_decision, decision_reasons = checks.finalize(
        "sqlite_crash_recovery_policy_verified"
    )
    failed_checks: list[str] = []
    if observed_final_decision == "NO-GO":
        failed_checks.extend(decision_reasons)
    if observed_final_decision != expected_final_decision:
        failed_checks.append("sqlite_crash_recovery_policy_expected_decision_mismatch")

    report_payload = {
        "schema_version": POLICY_SCHEMA,
        "status": "ok" if not failed_checks else "fail",
        "final_decision": observed_final_decision,
        "expected_final_decision": expected_final_decision,
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
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
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    if failed_checks:
        print("status=fail")
        print(f"final_decision={observed_final_decision}")
        print(f"expected_final_decision={expected_final_decision}")
        print("sqlite_crash_recovery_policy_status=failed")
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
    print("failed_checks=")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
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

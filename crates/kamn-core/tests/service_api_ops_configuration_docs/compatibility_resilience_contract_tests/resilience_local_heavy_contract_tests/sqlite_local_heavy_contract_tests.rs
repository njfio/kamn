use super::*;

const SQLITE_CRASH_RECOVERY_CI_DRY_RUN_MARKERS: &[&str] = &[
    "## SQLite Crash-Recovery CI Dry-Run Durability Governance Contract (Issue #4014)",
    "sqlite_crash_recovery_ci_dry_run_reason_taxonomy_version=kamn.ci.sqlite-crash-recovery-ci-dry-run-governance-reason-taxonomy.v1",
    "sqlite_crash_recovery_ci_dry_run_reason_codes_csv=sqlite_crash_recovery_ci_dry_run_argument_invalid,sqlite_crash_recovery_ci_dry_run_threshold_contract_violation,sqlite_crash_recovery_ci_dry_run_report_contract_violation,sqlite_crash_recovery_ci_dry_run_runtime_budget_exceeded,sqlite_crash_recovery_ci_dry_run_fast_mode_selector_drift,sqlite_crash_recovery_ci_dry_run_workflow_exclusion_drift,sqlite_crash_recovery_ci_dry_run_docs_marker_parity_drift,sqlite_crash_recovery_ci_dry_run_docs_remediation_marker_missing",
    "sqlite_crash_recovery_ci_dry_run_threshold_fixture_path=fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env",
    "sqlite_crash_recovery_ci_dry_run_max_seconds=120",
    "sqlite_crash_recovery_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract -- --nocapture",
    "sqlite_crash_recovery_ci_dry_run_fast_mode_forbidden_entry=bash \"$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh\" --mode run",
    "sqlite_crash_recovery_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/validate_sqlite_crash_recovery_live.sh --mode run",
    "python3 scripts/ci/check_sqlite_crash_recovery_ci_dry_run_governance.py --sqlite-crash-recovery-summary-report-file /tmp/sqlite-crash-recovery-live-summary.json --sqlite-crash-recovery-policy-report-file /tmp/sqlite-crash-recovery-live-policy.json --sqlite-crash-recovery-contract-lane-report-file /tmp/sqlite-crash-recovery-live-contract-lane-report.json --threshold-file fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env --strategy-doc docs/ci/strategy.md --ops-doc docs/ops/configuration.md --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --output-json /tmp/sqlite-crash-recovery-ci-dry-run-governance-report.json",
    "cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract -- --nocapture",
    "sqlite_crash_recovery_ci_dry_run_remediation_map_version=v1",
    "Regression: #4014",
];

const SQLITE_CRASH_RECOVERY_CI_DRY_RUN_REMEDIATION_CODES: &[&str] = &[
    "sqlite_crash_recovery_ci_dry_run_argument_invalid",
    "sqlite_crash_recovery_ci_dry_run_threshold_contract_violation",
    "sqlite_crash_recovery_ci_dry_run_report_contract_violation",
    "sqlite_crash_recovery_ci_dry_run_runtime_budget_exceeded",
    "sqlite_crash_recovery_ci_dry_run_fast_mode_selector_drift",
    "sqlite_crash_recovery_ci_dry_run_workflow_exclusion_drift",
    "sqlite_crash_recovery_ci_dry_run_docs_marker_parity_drift",
    "sqlite_crash_recovery_ci_dry_run_docs_remediation_marker_missing",
];

#[test]
fn service_api_ops_configuration_contains_sqlite_crash_recovery_ci_dry_run_governance_markers() {
    assert_doc_contains_all(SQLITE_CRASH_RECOVERY_CI_DRY_RUN_MARKERS);
    assert_doc_contains_prefixed_entries("sqlite_crash_recovery_ci_dry_run_remediation", SQLITE_CRASH_RECOVERY_CI_DRY_RUN_REMEDIATION_CODES);
}
#[test]
fn service_api_ops_configuration_contains_journal_wal_partial_write_fault_injection_markers() {
    assert_doc_contains_all(&["## Journal/WAL Partial-Write Fault Injection Contracts (Issue #4016)", "journal_wal_partial_write_fixture_path=fixtures/runtime/journal_wal_partial_write_fault_matrix.txt", "journal_wal_partial_write_fixture_schema_version=kamn.runtime.journal-wal-partial-write-fault-matrix.v1", "journal_wal_partial_write_reason_taxonomy_version=kamn.runtime.journal-wal-partial-write-reason-taxonomy.v1", "journal_wal_partial_write_reason_codes_csv=partial_snapshot_file_write_recovered_from_journal,partial_journal_tail_write_fail_closed,partial_snapshot_without_journal_repaired", "journal_wal_partial_write_required_fault_modes_csv=partial_snapshot_file_write,partial_journal_tail_write,partial_snapshot_without_journal", "partial_snapshot_file_write -> recovery_clean", "partial_journal_tail_write -> fail_closed_corrupt_tail", "partial_snapshot_without_journal -> recovery_repaired_corrupt_payload", "cargo test -p kamn-core --test journal_wal_partial_write_fault_contract -- --nocapture", "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_journal_wal_partial_write_fault_injection_markers -- --exact", "Regression: #4016"]);
}
#[test]
fn service_api_ops_configuration_contains_sqlite_crash_restart_local_heavy_lane_markers() {
    assert_doc_contains_all(&["## SQLite Crash-Restart Local-Heavy Lane Artifact Contract (Issue #4017)", "sqlite_crash_restart_local_heavy_lane_schema_version=kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1", "sqlite_crash_restart_local_heavy_artifact_schema_version=kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1", "sqlite_crash_restart_local_heavy_reason_taxonomy_version=kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1", "sqlite_crash_restart_local_heavy_reason_codes_csv=crash_restart_profile_restart_status_mismatch,crash_restart_profile_corruption_status_mismatch,crash_restart_profile_combined_status_mismatch", "sqlite_crash_restart_local_heavy_required_profiles_csv=restart,corruption,combined", "profile=restart -> restart_drill_status=verified", "profile=corruption -> corruption_drill_status=verified", "profile=combined -> restart_drill_status=verified + corruption_drill_status=verified", "bash scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh --profile combined --mode dry-run --ci-fast-gate PASS --max-seconds 240 --output-json /tmp/sqlite-crash-restart-local-heavy-lane-report.json", "cargo test -p kamn-core --test sqlite_crash_restart_local_heavy_lane_contract -- --nocapture", "Regression: #4017"]);
}

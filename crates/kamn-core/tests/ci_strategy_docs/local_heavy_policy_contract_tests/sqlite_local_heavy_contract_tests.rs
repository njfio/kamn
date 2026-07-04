use super::super::fairness_deletion_support::assert_contains_all;
use super::super::DOC;

#[test]
fn doc_contains_sqlite_crash_recovery_ci_dry_run_durability_governance_contract() {
    assert_sqlite_dry_run_doc_headers();
    assert_sqlite_dry_run_doc_policy();
    assert_sqlite_dry_run_remediation_markers();
    assert!(DOC.contains("Regression: #4014"));
}

#[test]
fn doc_contains_sqlite_crash_restart_local_heavy_policy_checker_contract() {
    assert_contains_all(
        DOC,
        &[
            "## SQLite Crash-Restart Local-Heavy Policy Checker Contract",
            "bash scripts/runtime/check_sqlite_crash_restart_local_heavy_policy.sh --report-file /tmp/sqlite-crash-restart-local-heavy-lane-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --strategy-doc docs/ci/strategy.md --output-json /tmp/sqlite-crash-restart-local-heavy-policy-report.json",
            "bash scripts/runtime/test_check_sqlite_crash_restart_local_heavy_policy.sh",
            "sqlite_crash_restart_local_heavy_policy_reason_taxonomy_version=kamn.runtime.sqlite-crash-restart-local-heavy-policy-reason-taxonomy.v1",
            "sqlite_crash_restart_local_heavy_policy_reason_codes_csv=sqlite_crash_restart_policy_required_field_missing,sqlite_crash_restart_policy_marker_mismatch,sqlite_crash_restart_policy_reason_taxonomy_mismatch,sqlite_crash_restart_policy_profile_contract_mismatch,sqlite_crash_restart_policy_runbook_marker_parity_mismatch,sqlite_crash_restart_policy_strategy_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_restart_policy_expected_decision_mismatch,sqlite_crash_restart_policy_violation",
            "sqlite_crash_restart_local_heavy_policy_runbook_path=docs/deploy/kolme_devnet_ops.md",
            "sqlite_crash_restart_local_heavy_policy_strategy_doc_path=docs/ci/strategy.md",
            "sqlite_crash_restart_policy_runbook_marker_parity_mismatch",
            "sqlite_crash_restart_policy_strategy_marker_parity_mismatch",
            "Regression: #4018",
        ],
        "sqlite local heavy policy",
    );
}

fn assert_sqlite_dry_run_doc_headers() {
    assert_contains_all(
        DOC,
        &[
            "## SQLite Crash-Recovery CI Dry-Run Durability Governance Contract",
            "python3 scripts/ci/check_sqlite_crash_recovery_ci_dry_run_governance.py --sqlite-crash-recovery-summary-report-file /tmp/sqlite-crash-recovery-live-summary.json --sqlite-crash-recovery-policy-report-file /tmp/sqlite-crash-recovery-live-policy.json --sqlite-crash-recovery-contract-lane-report-file /tmp/sqlite-crash-recovery-live-contract-lane-report.json --threshold-file fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env --strategy-doc docs/ci/strategy.md --ops-doc docs/ops/configuration.md --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --output-json /tmp/sqlite-crash-recovery-ci-dry-run-governance-report.json",
            "cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract -- --nocapture",
            "sqlite_crash_recovery_ci_dry_run_reason_taxonomy_version=kamn.ci.sqlite-crash-recovery-ci-dry-run-governance-reason-taxonomy.v1",
            "sqlite_crash_recovery_ci_dry_run_reason_codes_csv=sqlite_crash_recovery_ci_dry_run_argument_invalid,sqlite_crash_recovery_ci_dry_run_threshold_contract_violation,sqlite_crash_recovery_ci_dry_run_report_contract_violation,sqlite_crash_recovery_ci_dry_run_runtime_budget_exceeded,sqlite_crash_recovery_ci_dry_run_fast_mode_selector_drift,sqlite_crash_recovery_ci_dry_run_workflow_exclusion_drift,sqlite_crash_recovery_ci_dry_run_docs_marker_parity_drift,sqlite_crash_recovery_ci_dry_run_docs_remediation_marker_missing",
            "sqlite_crash_recovery_ci_dry_run_threshold_fixture_path=fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env",
            "sqlite_crash_recovery_ci_dry_run_max_seconds=120",
        ],
        "sqlite dry run governance",
    );
}

fn assert_sqlite_dry_run_doc_policy() {
    assert_contains_all(
        DOC,
        &[
            "sqlite_crash_recovery_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract -- --nocapture",
            "sqlite_crash_recovery_ci_dry_run_fast_mode_forbidden_entry=bash \"$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh\" --mode run",
            "sqlite_crash_recovery_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/validate_sqlite_crash_recovery_live.sh --mode run",
            "sqlite_crash_recovery_ci_dry_run_remediation_map_version=v1",
        ],
        "sqlite dry run policy",
    );
}

fn assert_sqlite_dry_run_remediation_markers() {
    for reason_code in [
        "sqlite_crash_recovery_ci_dry_run_argument_invalid",
        "sqlite_crash_recovery_ci_dry_run_threshold_contract_violation",
        "sqlite_crash_recovery_ci_dry_run_report_contract_violation",
        "sqlite_crash_recovery_ci_dry_run_runtime_budget_exceeded",
        "sqlite_crash_recovery_ci_dry_run_fast_mode_selector_drift",
        "sqlite_crash_recovery_ci_dry_run_workflow_exclusion_drift",
        "sqlite_crash_recovery_ci_dry_run_docs_marker_parity_drift",
        "sqlite_crash_recovery_ci_dry_run_docs_remediation_marker_missing",
    ] {
        assert!(DOC.contains(&format!(
            "sqlite_crash_recovery_ci_dry_run_remediation.{reason_code}="
        )));
    }
}

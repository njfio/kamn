use super::super::{DOC, OVERLOAD_RUNNER_SOURCE};
use super::super::fairness_deletion_support::assert_contains_all;

#[test]
fn doc_contains_overload_ci_dry_run_policy_checker_markers() {
    assert_contains_all(
        DOC,
        &[
            "### Overload CI Dry-Run Policy Checker Contract",
            "python3 scripts/ci/check_daemon_os_signal_stress_policy.py --report-file /tmp/daemon-os-signal-stress-matrix-report.json --threshold-file fixtures/ci/daemon_os_signal_stress_policy_thresholds.env --ci-tools-script scripts/ci/test_ci_tools.sh --expected-final-decision GO --output-json /tmp/daemon-os-signal-stress-policy-report.json",
            "fixtures/ci/daemon_os_signal_stress_policy_thresholds.env",
            "overload_policy_reason_taxonomy_version=kamn.ci.daemon-os-signal-stress-policy-reason-taxonomy.v1",
            "overload_policy_reason_codes_csv=overload_policy_argument_invalid,overload_policy_ci_tools_fast_mode_heavy_run_leaked,overload_policy_ci_tools_fast_mode_missing_overload_test,overload_policy_ci_tools_script_missing,overload_policy_expected_decision_mismatch,overload_policy_output_json_required,overload_policy_reason_code_unknown,overload_policy_report_file_missing,overload_policy_report_json_invalid,overload_policy_report_reason_codes_csv_mismatch,overload_policy_report_reason_taxonomy_mismatch,overload_policy_report_schema_mismatch,overload_policy_runtime_budget_exceeded,overload_policy_threshold_file_missing,overload_policy_threshold_key_missing,overload_policy_threshold_value_invalid",
            "REPORT_REASON_TAXONOMY_VERSION",
            "REPORT_REASON_CODES_CSV",
            "reason_taxonomy_version=kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1",
            "reason_codes_csv=runtime_budget_exceeded,matrix_failure_threshold_exceeded,quarantine_registry_missing,quarantine_reference_present_without_followup,matrix_failures_within_threshold,stable_success_with_quarantine_followup,stable_success",
            "cargo test -p kamn-core --test ci_strategy_docs doc_contains_overload_ci_dry_run_policy_checker_markers -- --exact",
            "bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh",
            "Regression: #4096, #4095",
        ],
        "overload dry run",
    );
}

#[test]
fn doc_enforces_overload_runner_projects_taxonomy_contract_markers() {
    assert_overload_runner_contract_markers();
    assert_overload_threshold_key_markers();
    assert_overload_policy_reason_markers();
}

fn assert_overload_runner_contract_markers() {
    assert_contains_all(
        OVERLOAD_RUNNER_SOURCE,
        &[
            "reason_taxonomy_version=\"kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1\"",
            "reason_codes_csv=\"runtime_budget_exceeded,matrix_failure_threshold_exceeded,quarantine_registry_missing,quarantine_reference_present_without_followup,matrix_failures_within_threshold,stable_success_with_quarantine_followup,stable_success\"",
            "\"reason_taxonomy_version\": reason_taxonomy_version",
            "\"reason_codes_csv\": reason_codes_csv",
            "echo \"daemon_os_signal_stress_matrix_reason_taxonomy_version=$reason_taxonomy_version\"",
            "echo \"daemon_os_signal_stress_matrix_reason_codes_csv=$reason_codes_csv\"",
        ],
        "overload runner taxonomy",
    );
}

fn assert_overload_threshold_key_markers() {
    for threshold_key in [
        "REPORT_SCHEMA_VERSION",
        "MAX_RUNTIME_SECONDS",
        "ALLOWED_REASON_CODES_CSV",
        "REPORT_REASON_TAXONOMY_VERSION",
        "REPORT_REASON_CODES_CSV",
        "CI_TOOLS_REQUIRED_ENTRY",
        "CI_TOOLS_FORBIDDEN_ENTRY",
    ] {
        assert!(DOC.contains(threshold_key), "missing overload dry-run threshold key marker {threshold_key}");
    }
}

fn assert_overload_policy_reason_markers() {
    for reason in [
        "overload_policy_report_reason_taxonomy_mismatch",
        "overload_policy_report_reason_codes_csv_mismatch",
    ] {
        assert!(DOC.contains(reason), "missing overload dry-run reason marker {reason}");
    }
}

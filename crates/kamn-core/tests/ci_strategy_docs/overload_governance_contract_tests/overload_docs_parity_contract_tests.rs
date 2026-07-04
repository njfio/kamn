use super::super::fairness_deletion_support::{
    assert_contains_all, assert_docs_and_ops_remediation_markers,
};
use super::super::{
    overload_reason_codes, DOC, OPS_DOC, OVERLOAD_REASON_CODES_CSV,
    OVERLOAD_REASON_TAXONOMY_VERSION, OVERLOAD_RUNNER_SOURCE,
};

#[test]
fn doc_contains_overload_docs_parity_and_go_no_go_markers() {
    assert_contains_all(
        DOC,
        &[
            "### Overload Docs/Runbook and Go-No-Go Marker Parity Contract",
            "overload_docs_parity_reason_taxonomy_version=kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1",
            "overload_docs_parity_reason_codes_csv=runtime_budget_exceeded,matrix_failure_threshold_exceeded,quarantine_registry_missing,quarantine_reference_present_without_followup,matrix_failures_within_threshold,stable_success_with_quarantine_followup,stable_success",
            "overload_docs_parity_runner_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1",
            "overload_docs_parity_runner_script_path=scripts/ci/run_daemon_os_signal_stress_matrix.sh",
            "overload_docs_parity_ops_doc_path=docs/ops/configuration.md",
            "overload_docs_parity_strategy_doc_path=docs/ci/strategy.md",
            "overload_docs_parity_go_no_go_status=verified",
            "overload_docs_parity_go_no_go_decision_contract=GO|NO-GO",
            "overload_docs_parity_remediation_map_version=v1",
            "cargo test -p kamn-core --test ci_strategy_docs doc_contains_overload_docs_parity_and_go_no_go_markers -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_overload_docs_parity_matches_ops_docs_and_runner_markers -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_overload_docs_parity_requires_remediation_marker_for_each_reason_code -- --exact",
            "Regression: #4097",
        ],
        "overload docs parity",
    );
}

#[test]
fn doc_enforces_overload_docs_parity_matches_ops_docs_and_runner_markers() {
    assert_overload_strategy_markers();
    assert_overload_ops_markers();
    assert_overload_runner_reason_markers();
}

#[test]
fn doc_enforces_overload_docs_parity_requires_remediation_marker_for_each_reason_code() {
    assert_docs_and_ops_remediation_markers(
        "overload_docs_parity_remediation",
        overload_reason_codes(),
        "overload",
    );
}

fn assert_overload_strategy_markers() {
    assert!(DOC.contains(&format!(
        "overload_docs_parity_reason_taxonomy_version={OVERLOAD_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "overload_docs_parity_reason_codes_csv={OVERLOAD_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains("overload_docs_parity_runner_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1"));
}

fn assert_overload_ops_markers() {
    assert_contains_all(
        OPS_DOC,
        &[
            "daemon_os_signal_stress_matrix_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1",
            "daemon_os_signal_stress_profile_injected_overload_reason_code=matrix_failure_threshold_exceeded",
            "daemon_os_signal_stress_profile_runtime_budget_reason_code=runtime_budget_exceeded",
            "daemon_os_signal_stress_profile_quarantine_reason_code=quarantine_reference_present_without_followup",
            "overload_docs_parity_reason_taxonomy_version=kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1",
        ],
        "overload ops parity",
    );
    assert!(OPS_DOC.contains(&format!(
        "overload_docs_parity_reason_codes_csv={OVERLOAD_REASON_CODES_CSV}"
    )));
}

fn assert_overload_runner_reason_markers() {
    assert!(OVERLOAD_RUNNER_SOURCE.contains("kamn.ci.daemon-os-signal-stress-matrix-report.v1"));
    for reason_code in overload_reason_codes() {
        assert!(
            OVERLOAD_RUNNER_SOURCE.contains(&format!("reason_code=\"{reason_code}\"")),
            "runner source missing overload reason marker {reason_code}"
        );
    }
}

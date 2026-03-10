use super::*;

#[test]
fn service_api_ops_configuration_contains_convergence_promotion_marker_contracts() {
    assert!(DOC.contains("## Convergence Promotion Marker Contracts (Issue #5301)"));
    assert!(DOC.contains("convergence_promotion_contract_status=verified"));
    assert!(DOC.contains(
        "convergence_reason_taxonomy_version=kamn.runtime.daemon.convergence.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "convergence_reason_codes_csv=convergence_promotion_gate_go,convergence_schema_drift_detected,convergence_error_path_drift_detected,convergence_concurrency_drift_detected,convergence_performance_budget_exceeded,convergence_cost_budget_exceeded"
    ));
    assert!(DOC.contains(
        "convergence_promotion_contract=schema+error_path+concurrency+performance+cost->decision;any_failed_gate=no_go"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_daemon_convergence_projection_fail_closed_reason_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5301"));
}

#[test]
fn service_api_ops_configuration_contains_retention_policy_fixture_matrix_controls() {
    assert!(DOC
        .contains("## Retention Policy Fixture Matrix and Parser Helper Contracts (Issue #4075)"));
    assert!(DOC.contains(
        "retention_policy_fixture_matrix_path=fixtures/runtime/retention_policy_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "retention_policy_fixture_matrix_schema_version=kamn.runtime.retention-policy-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "retention_policy_reason_taxonomy_version=kamn.runtime.retention-policy-fixture-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "retention_policy_reason_codes_csv=retention_domain_unknown,retention_window_non_positive"
    ));
    assert!(DOC.contains(
        "retention_policy_fixture_columns=case_id|domain|max_age_seconds|expected_status|expected_reason_code"
    ));
    assert!(DOC.contains("retention_domain_unknown"));
    assert!(DOC.contains("retention_window_non_positive"));
    assert!(DOC.contains("cargo test -p kamn-core --test retention_policy_fixture_parser_contract"));
    assert!(DOC.contains("Regression: #4075"));
}

#[test]
fn service_api_ops_configuration_contains_deletion_proof_artifact_fixture_controls() {
    assert_doc_contains_all(&["## Deletion-Proof Artifact Fixture Set and Checker Behavior Contracts (Issue #4077)", "deletion_proof_fixture_matrix_path=fixtures/runtime/deletion_proof_artifact_fixture_matrix.txt", "deletion_proof_fixture_matrix_schema_version=kamn.runtime.deletion-proof-fixture-matrix.v1", "deletion_proof_reason_taxonomy_version=kamn.runtime.deletion-proof-checker-reason-taxonomy.v1", "deletion_proof_reason_codes_csv=deletion_proof_subject_missing,deletion_proof_tombstone_missing,deletion_proof_status_invalid,deletion_proof_hash_mismatch", "deletion_proof_fixture_columns=case_id|subject_id|tombstone_hash|expected_hash|proof_status|expected_status|expected_reason_code", "deletion_proof_subject_missing", "deletion_proof_tombstone_missing", "deletion_proof_status_invalid", "deletion_proof_hash_mismatch", "cargo test -p kamn-core --test deletion_proof_artifact_checker_contract", "Regression: #4077"]);
}
#[test]
fn service_api_ops_configuration_contains_quota_policy_fixture_matrix_controls() {
    assert!(
        DOC.contains("## Quota Policy Fixture Matrix and Parser Helper Contracts (Issue #4090)")
    );
    assert!(DOC.contains(
        "quota_policy_fixture_matrix_path=fixtures/runtime/quota_policy_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "quota_policy_fixture_matrix_schema_version=kamn.runtime.quota-policy-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "quota_policy_reason_taxonomy_version=kamn.runtime.quota-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "quota_policy_reason_codes_csv=quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive"
    ));
    assert!(DOC.contains(
        "quota_policy_fixture_columns=case_id|scope|window_seconds|limit|expected_status|expected_reason_code"
    ));
    assert!(DOC.contains("quota_scope_unknown"));
    assert!(DOC.contains("quota_window_non_positive"));
    assert!(DOC.contains("quota_limit_non_positive"));
    assert!(DOC.contains("cargo test -p kamn-core --test quota_policy_fixture_parser_contract"));
    assert!(DOC.contains("Regression: #4090"));
}

#[test]
fn service_api_ops_configuration_contains_fairness_starvation_fixture_controls() {
    assert!(DOC.contains("## Fairness Starvation Fixture and Checker Contracts (Issue #4092)"));
    assert!(DOC.contains(
        "fairness_fixture_matrix_path=fixtures/runtime/starvation_fairness_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "fairness_fixture_matrix_schema_version=kamn.runtime.fairness-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "fairness_reason_taxonomy_version=kamn.runtime.fairness-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "fairness_reason_codes_csv=fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap"
    ));
    assert!(DOC.contains(
        "fairness_fixture_columns=case_id|scope|window_seconds|active_weighted_share|max_weighted_share_gap|expected_status|expected_reason_code"
    ));
    assert!(DOC.contains("fairness_scope_unknown"));
    assert!(DOC.contains("fairness_window_non_positive"));
    assert!(DOC.contains("fairness_max_gap_non_positive"));
    assert!(DOC.contains("fairness_weighted_share_exceeds_gap"));
    assert!(DOC.contains("cargo test -p kamn-core --test fairness_policy_checker_contract"));
    assert!(DOC.contains("Regression: #4092"));
}

#[test]
fn service_api_ops_configuration_contains_overload_docs_parity_remediation_controls() {
    assert_doc_contains_all(&["## Daemon OS-Signal Stress Matrix Overload Profiles (Issue #4094)", "daemon_os_signal_stress_matrix_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1", "overload_docs_parity_reason_taxonomy_version=kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1", "overload_docs_parity_reason_codes_csv=runtime_budget_exceeded,matrix_failure_threshold_exceeded,quarantine_registry_missing,quarantine_reference_present_without_followup,matrix_failures_within_threshold,stable_success_with_quarantine_followup,stable_success", "overload_docs_parity_remediation_map_version=v1", "overload_docs_parity_remediation.runtime_budget_exceeded=reduce iterations or increase max-seconds budget after validating reproducer runtime", "overload_docs_parity_remediation.matrix_failure_threshold_exceeded=triage failing iteration artifacts and rerun reproducer before promotion", "overload_docs_parity_remediation.quarantine_registry_missing=restore .ci/flaky-tests.txt or pass an explicit --registry-file", "overload_docs_parity_remediation.quarantine_reference_present_without_followup=add --quarantine-followup-issue #<id> or retire stale quarantine entries", "overload_docs_parity_remediation.matrix_failures_within_threshold=track flaky rows and keep threshold + waiver evidence attached to release review", "overload_docs_parity_remediation.stable_success_with_quarantine_followup=keep follow-up issue open until quarantine references are retired", "overload_docs_parity_remediation.stable_success=no action required; retain report artifact link in release checklist", "Regression: #4097"]);
}

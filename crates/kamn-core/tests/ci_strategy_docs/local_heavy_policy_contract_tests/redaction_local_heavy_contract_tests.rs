use super::super::{
    local_heavy_redaction_policy_reason_codes, DOC,
    LOCAL_HEAVY_REDACTION_POLICY_REASON_CODES_CSV, LOCAL_HEAVY_REDACTION_POLICY_REASON_TAXONOMY_VERSION,
    LOCAL_HEAVY_REDACTION_REASON_CODES_CSV, LOCAL_HEAVY_REDACTION_REASON_TAXONOMY_VERSION,
    LOCAL_HEAVY_REDACTION_RUNNER_SOURCE, OPS_DOC,
};
use super::super::fairness_deletion_support::assert_contains_all;

#[test]
fn doc_contains_local_heavy_redaction_validation_policy_checker_contract() {
    assert_contains_all(
        DOC,
        &[
            "## Local-Heavy Redaction Validation Policy Checker Contract",
            "bash scripts/runtime/check_local_heavy_redaction_validation_policy.sh --report-file /tmp/local-heavy-redaction-validation-baseline.json --expected-final-decision GO --ci-fast-gate PASS --strategy-doc docs/ci/strategy.md --ops-doc docs/ops/configuration.md --output-json /tmp/local-heavy-redaction-validation-policy-report.json",
            "bash scripts/runtime/test_check_local_heavy_redaction_validation_policy.sh",
            "local_heavy_redaction_validation_policy_reason_taxonomy_version=kamn.runtime.local-heavy-redaction-validation-policy-reason-taxonomy.v1",
            "local_heavy_redaction_validation_policy_reason_codes_csv=redaction_policy_required_field_missing,redaction_policy_marker_mismatch,redaction_policy_reason_taxonomy_mismatch,redaction_policy_profile_contract_mismatch,redaction_policy_docs_marker_parity_mismatch,ci_fast_gate_failed,redaction_policy_expected_decision_mismatch,redaction_policy_violation",
            "local_heavy_redaction_validation_policy_strategy_doc_path=docs/ci/strategy.md",
            "local_heavy_redaction_validation_policy_ops_doc_path=docs/ops/configuration.md",
            "local_heavy_redaction_validation_policy_runner_report_schema_version=kamn.runtime.local-heavy-redaction-validation-lane-report.v1",
            "local_heavy_redaction_validation_policy_runner_reason_taxonomy_version=kamn.runtime.local-heavy-redaction-validation-reason-taxonomy.v1",
            "redaction_policy_docs_marker_parity_mismatch",
            "Regression: #4080",
        ],
        "local heavy redaction policy",
    );
}

#[test]
fn doc_enforces_local_heavy_redaction_policy_checker_docs_parity_matches_runner_and_ops_markers() {
    assert_redaction_strategy_markers();
    assert_redaction_ops_markers();
    assert_redaction_runner_markers();
}

#[test]
fn doc_enforces_local_heavy_redaction_policy_checker_reason_codes_have_deterministic_marker_coverage() {
    for reason_code in local_heavy_redaction_policy_reason_codes() {
        assert!(DOC.contains(reason_code), "ci strategy docs missing redaction policy reason marker {reason_code}");
    }
}

fn assert_redaction_strategy_markers() {
    assert!(DOC.contains(&format!("local_heavy_redaction_validation_policy_reason_taxonomy_version={LOCAL_HEAVY_REDACTION_POLICY_REASON_TAXONOMY_VERSION}")));
    assert!(DOC.contains(&format!("local_heavy_redaction_validation_policy_reason_codes_csv={LOCAL_HEAVY_REDACTION_POLICY_REASON_CODES_CSV}")));
    assert!(DOC.contains(&format!("local_heavy_redaction_validation_policy_runner_reason_taxonomy_version={LOCAL_HEAVY_REDACTION_REASON_TAXONOMY_VERSION}")));
    assert!(DOC.contains(&format!("local_heavy_redaction_validation_policy_runner_reason_codes_csv={LOCAL_HEAVY_REDACTION_REASON_CODES_CSV}")));
}

fn assert_redaction_ops_markers() {
    assert!(OPS_DOC.contains(&format!("local_heavy_redaction_validation_reason_taxonomy_version={LOCAL_HEAVY_REDACTION_REASON_TAXONOMY_VERSION}")));
    assert!(OPS_DOC.contains(&format!("local_heavy_redaction_validation_reason_codes_csv={LOCAL_HEAVY_REDACTION_REASON_CODES_CSV}")));
    assert!(OPS_DOC.contains("local_heavy_redaction_validation_required_profiles_csv=baseline,injected-leak"));
}

fn assert_redaction_runner_markers() {
    assert!(LOCAL_HEAVY_REDACTION_RUNNER_SOURCE.contains("RUN_SCHEMA_VERSION = \"kamn.runtime.local-heavy-redaction-validation-lane-report.v1\""));
    assert!(LOCAL_HEAVY_REDACTION_RUNNER_SOURCE.contains(&format!("REASON_TAXONOMY_VERSION = \"{LOCAL_HEAVY_REDACTION_REASON_TAXONOMY_VERSION}\"")));
    assert!(LOCAL_HEAVY_REDACTION_RUNNER_SOURCE.contains("REASON_CODES_CSV = ("));
    for reason_code in LOCAL_HEAVY_REDACTION_REASON_CODES_CSV.split(',') {
        assert!(LOCAL_HEAVY_REDACTION_RUNNER_SOURCE.contains(reason_code), "runner source missing redaction reason marker {reason_code}");
    }
}

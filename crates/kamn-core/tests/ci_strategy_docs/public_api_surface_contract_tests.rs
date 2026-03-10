use super::DOC;
use super::fairness_deletion_support::assert_contains_all;

#[test]
fn doc_contains_public_api_surface_ratchet_contract_markers() {
    assert_public_api_fixture_markers();
    assert_public_api_report_markers();
    assert_public_api_reason_markers();
    assert_public_api_waiver_markers();
}

fn assert_public_api_fixture_markers() {
    assert_contains_all(
        DOC,
        &[
            "Public API surface ratchet (Rust-first, fail-closed):",
            "fixtures/ci/kamn_core_public_api_surface_baseline.env",
            ".ci/kamn-core-public-api-surface-thresholds.env",
            ".ci/kamn-core-public-api-surface-waiver.env",
            ".ci/kamn-core-public-api-surface-waiver.example.env",
            "KAMN_CORE_PUBLIC_API_SURFACE_REPORT_OUTPUT=/tmp/kamn-core-public-api-surface-report.env cargo test -p kamn-core --test public_api_surface_policy public_api_surface_report_schema_is_deterministic -- --exact --nocapture",
            "cargo test -p kamn-core --test public_api_surface_policy public_api_surface_policy_enforces_warn_fail_contract -- --exact --nocapture",
        ],
        "public api surface fixture",
    );
}

fn assert_public_api_report_markers() {
    assert_contains_all(
        DOC,
        &[
            "report_schema_version=kamn.core.public-api-surface-report.v1",
            "policy_schema_version=kamn.core.public-api-surface-thresholds.v1",
            "policy_status=within|warn|exception-applied",
            "module_public_items.<module>=<integer>",
            "module_public_items_delta.<module>=<integer>",
        ],
        "public api surface report",
    );
}

fn assert_public_api_reason_markers() {
    assert_contains_all(
        DOC,
        &[
            "public_api_surface_reason_taxonomy_version=kamn.core.public-api-surface-reason-taxonomy.v1",
            "public_api_surface_reason_codes_csv=baseline_fixture_missing,baseline_fixture_invalid,baseline_schema_mismatch,baseline_threshold_missing,baseline_threshold_invalid,baseline_module_missing,module_source_missing,threshold_fixture_missing,threshold_fixture_invalid,threshold_schema_mismatch,threshold_value_invalid,waiver_fixture_invalid,waiver_schema_mismatch,waiver_missing_mitigation_issue,waiver_invalid_mitigation_issue,waiver_cap_exceeded,public_api_surface_fail_threshold_exceeded_unwaived,report_output_write_failed",
            "reason_codes=public_api_surface_warn_threshold_exceeded",
            "reason_codes=public_api_surface_fail_threshold_exceeded_unwaived",
            "reason_codes=waiver_cap_exceeded",
        ],
        "public api surface reason",
    );
}

fn assert_public_api_waiver_markers() {
    assert_contains_all(
        DOC,
        &["set `mitigation_issue=#<issue-id>` and a bounded `max_total_delta`"],
        "public api surface waiver",
    );
}

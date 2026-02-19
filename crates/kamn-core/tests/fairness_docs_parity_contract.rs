use kamn_core::fairness_policy::{
    fairness_policy_reason_codes_csv, fairness_policy_reason_taxonomy_version,
};

const OPS_DOC: &str = include_str!("../../../docs/ops/configuration.md");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const FIXTURE: &str =
    include_str!("../../../fixtures/runtime/starvation_fairness_fixture_matrix.txt");

fn fairness_reason_codes() -> Vec<&'static str> {
    fairness_policy_reason_codes_csv().split(',').collect()
}

#[test]
fn unit_fairness_docs_parity_checker_taxonomy_markers_remain_deterministic() {
    assert_eq!(
        fairness_policy_reason_taxonomy_version(),
        "kamn.runtime.fairness-policy-reason-taxonomy.v1"
    );
    assert_eq!(
        fairness_policy_reason_codes_csv(),
        "fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap"
    );
}

#[test]
fn functional_fairness_docs_parity_strategy_markers_are_present() {
    assert!(CI_STRATEGY_DOC.contains("### Fairness Docs Parity and Remediation Contract"));
    assert!(CI_STRATEGY_DOC.contains(
        "fairness_docs_parity_reason_taxonomy_version=kamn.runtime.fairness-policy-reason-taxonomy.v1"
    ));
    assert!(CI_STRATEGY_DOC.contains(
        "fairness_docs_parity_reason_codes_csv=fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap"
    ));
    assert!(CI_STRATEGY_DOC.contains(
        "fairness_docs_parity_fixture_path=fixtures/runtime/starvation_fairness_fixture_matrix.txt"
    ));
    assert!(CI_STRATEGY_DOC.contains("fairness_docs_parity_ops_doc_path=docs/ops/configuration.md"));
    assert!(CI_STRATEGY_DOC.contains("fairness_docs_parity_strategy_doc_path=docs/ci/strategy.md"));
    assert!(CI_STRATEGY_DOC.contains("fairness_docs_parity_remediation_map_version=v1"));
    assert!(
        CI_STRATEGY_DOC.contains("cargo test -p kamn-core --test fairness_docs_parity_contract")
    );
}

#[test]
fn integration_fairness_docs_parity_matches_ops_docs_and_fixture_metadata() {
    let taxonomy = fairness_policy_reason_taxonomy_version();
    let reason_codes_csv = fairness_policy_reason_codes_csv();

    assert!(OPS_DOC.contains(&format!("fairness_reason_taxonomy_version={taxonomy}")));
    assert!(OPS_DOC.contains(&format!("fairness_reason_codes_csv={reason_codes_csv}")));

    assert!(FIXTURE.contains(
        "fairness_fixture_matrix_schema_version=kamn.runtime.fairness-fixture-matrix.v1"
    ));
    assert!(FIXTURE.contains(&format!("fairness_reason_taxonomy_version={taxonomy}")));
    assert!(FIXTURE.contains(&format!("fairness_reason_codes_csv={reason_codes_csv}")));

    assert!(CI_STRATEGY_DOC.contains(&format!(
        "fairness_docs_parity_reason_taxonomy_version={taxonomy}"
    )));
    assert!(CI_STRATEGY_DOC.contains(&format!(
        "fairness_docs_parity_reason_codes_csv={reason_codes_csv}"
    )));
}

#[test]
fn regression_fairness_docs_parity_requires_remediation_marker_for_each_reason_code() {
    for reason_code in fairness_reason_codes() {
        assert!(
            CI_STRATEGY_DOC.contains(&format!("fairness_docs_parity_remediation.{reason_code}=")),
            "missing fairness remediation marker for {reason_code}"
        );
    }
}

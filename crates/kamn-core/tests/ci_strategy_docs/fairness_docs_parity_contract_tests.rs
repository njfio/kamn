use super::fairness_deletion_support::{
    assert_contains_all, assert_doc_remediation_markers, assert_reason_codes_non_empty,
};
use super::*;

#[test]
fn doc_contains_fairness_docs_parity_and_remediation_markers() {
    assert_fairness_doc_headers();
    assert_fairness_doc_paths();
    assert_fairness_doc_remediation_examples();
    assert_fairness_doc_commands();
    assert!(DOC.contains("Regression: #4093"));
}

#[test]
fn doc_enforces_fairness_docs_parity_source_taxonomy_markers_remain_deterministic() {
    assert!(
        FAIRNESS_POLICY_SHIM_SOURCE.contains("pub use kamn_runtime_guards::fairness_policy::*;")
    );
    assert!(FAIRNESS_POLICY_SOURCE
        .contains("pub const FAIRNESS_POLICY_REASON_TAXONOMY_VERSION: &str ="));
    assert!(FAIRNESS_POLICY_SOURCE.contains("pub const FAIRNESS_POLICY_REASON_CODES_CSV: &str ="));
    assert!(FAIRNESS_POLICY_SOURCE.contains(FAIRNESS_REASON_TAXONOMY_VERSION));
    assert!(FAIRNESS_POLICY_SOURCE.contains(FAIRNESS_REASON_CODES_CSV));
}

#[test]
fn doc_enforces_fairness_docs_parity_matches_ops_docs_and_fixture_metadata() {
    assert_fairness_strategy_markers();
    assert_fairness_ops_markers();
    assert_fairness_fixture_markers();
}

#[test]
fn doc_enforces_fairness_docs_parity_requires_remediation_marker_for_each_reason_code() {
    assert_doc_remediation_markers(
        "fairness_docs_parity_remediation",
        fairness_reason_codes(),
        "fairness",
    );
}

fn assert_fairness_doc_headers() {
    assert_contains_all(
        DOC,
        &[
            "### Fairness Docs Parity and Remediation Contract",
            "fairness_docs_parity_fixture_schema_version=kamn.runtime.fairness-fixture-matrix.v1",
        ],
        "fairness docs header",
    );
    assert!(DOC.contains(&format!(
        "fairness_docs_parity_reason_taxonomy_version={FAIRNESS_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "fairness_docs_parity_reason_codes_csv={FAIRNESS_REASON_CODES_CSV}"
    )));
}

fn assert_fairness_doc_paths() {
    assert_contains_all(
        DOC,
        &[
            "fairness_docs_parity_fixture_path=fixtures/runtime/starvation_fairness_fixture_matrix.txt",
            "fairness_docs_parity_ops_doc_path=docs/ops/configuration.md",
            "fairness_docs_parity_strategy_doc_path=docs/ci/strategy.md",
            "fairness_docs_parity_remediation_map_version=v1",
        ],
        "fairness docs path",
    );
}

fn assert_fairness_doc_remediation_examples() {
    assert_contains_all(
        DOC,
        &[
            "fairness_docs_parity_remediation.fairness_scope_unknown=use one of control_plane|tenant_interactive|bulk_replication",
            "fairness_docs_parity_remediation.fairness_window_non_positive=set window_seconds >= 1",
            "fairness_docs_parity_remediation.fairness_max_gap_non_positive=set max_weighted_share_gap >= 1",
            "fairness_docs_parity_remediation.fairness_weighted_share_exceeds_gap=reduce active_weighted_share or increase max_weighted_share_gap",
        ],
        "fairness remediation example",
    );
}

fn assert_fairness_doc_commands() {
    assert_contains_all(
        DOC,
        &[
            "cargo test -p kamn-core --test ci_strategy_docs doc_contains_fairness_docs_parity_and_remediation_markers -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_fairness_docs_parity_source_taxonomy_markers_remain_deterministic -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_fairness_docs_parity_matches_ops_docs_and_fixture_metadata -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_fairness_docs_parity_requires_remediation_marker_for_each_reason_code -- --exact",
        ],
        "fairness docs command",
    );
}

fn assert_fairness_strategy_markers() {
    assert!(DOC.contains(&format!(
        "fairness_docs_parity_reason_taxonomy_version={FAIRNESS_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "fairness_docs_parity_reason_codes_csv={FAIRNESS_REASON_CODES_CSV}"
    )));
}

fn assert_fairness_ops_markers() {
    assert!(OPS_DOC.contains(&format!(
        "fairness_reason_taxonomy_version={FAIRNESS_REASON_TAXONOMY_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "fairness_reason_codes_csv={FAIRNESS_REASON_CODES_CSV}"
    )));
}

fn assert_fairness_fixture_markers() {
    assert!(FAIRNESS_FIXTURE.contains(
        "fairness_fixture_matrix_schema_version=kamn.runtime.fairness-fixture-matrix.v1"
    ));
    assert!(FAIRNESS_FIXTURE.contains(&format!(
        "fairness_reason_taxonomy_version={FAIRNESS_REASON_TAXONOMY_VERSION}"
    )));
    assert!(FAIRNESS_FIXTURE.contains(&format!(
        "fairness_reason_codes_csv={FAIRNESS_REASON_CODES_CSV}"
    )));
    assert_reason_codes_non_empty(fairness_reason_codes(), "fairness");
}

use super::fairness_deletion_support::{
    assert_contains_all, assert_docs_and_ops_remediation_markers, assert_reason_codes_non_empty,
};
use super::*;

#[test]
fn doc_contains_deletion_docs_parity_and_remediation_markers() {
    assert_deletion_doc_headers();
    assert_deletion_doc_paths();
    assert_deletion_doc_commands();
    assert!(DOC.contains("Regression: #4078"));
}

#[test]
fn doc_enforces_deletion_docs_parity_matches_ops_docs_and_fixture_metadata() {
    assert_deletion_strategy_markers();
    assert_deletion_ops_markers();
    assert_deletion_fixture_markers();
}

#[test]
fn doc_enforces_deletion_docs_parity_requires_remediation_marker_for_each_reason_code() {
    assert_docs_and_ops_remediation_markers(
        "deletion_docs_parity_remediation",
        deletion_reason_codes(),
        "deletion docs-parity",
    );
}

#[test]
fn doc_enforces_deletion_docs_parity_reason_codes_non_empty() {
    assert_reason_codes_non_empty(deletion_reason_codes(), "deletion");
}

fn assert_deletion_doc_headers() {
    assert_contains_all(
        DOC,
        &[
            "### Deletion Docs/Runbook Parity and Remediation Contract",
            "deletion_docs_parity_fixture_schema_version=kamn.runtime.deletion-proof-fixture-matrix.v1",
        ],
        "deletion docs header",
    );
    assert!(DOC.contains(&format!(
        "deletion_docs_parity_reason_taxonomy_version={DELETION_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "deletion_docs_parity_reason_codes_csv={DELETION_REASON_CODES_CSV}"
    )));
}

fn assert_deletion_doc_paths() {
    assert_contains_all(
        DOC,
        &[
            "deletion_docs_parity_fixture_path=fixtures/runtime/deletion_proof_artifact_fixture_matrix.txt",
            "deletion_docs_parity_ops_doc_path=docs/ops/configuration.md",
            "deletion_docs_parity_strategy_doc_path=docs/ci/strategy.md",
            "deletion_docs_parity_remediation_map_version=v1",
        ],
        "deletion docs path",
    );
}

fn assert_deletion_doc_commands() {
    assert_contains_all(
        DOC,
        &[
            "cargo test -p kamn-core --test ci_strategy_docs doc_contains_deletion_docs_parity_and_remediation_markers -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_deletion_docs_parity_matches_ops_docs_and_fixture_metadata -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_deletion_docs_parity_requires_remediation_marker_for_each_reason_code -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_deletion_docs_parity_reason_codes_non_empty -- --exact",
        ],
        "deletion docs command",
    );
}

fn assert_deletion_strategy_markers() {
    assert!(DOC.contains(&format!(
        "deletion_docs_parity_reason_taxonomy_version={DELETION_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "deletion_docs_parity_reason_codes_csv={DELETION_REASON_CODES_CSV}"
    )));
}

fn assert_deletion_ops_markers() {
    assert!(OPS_DOC.contains(&format!(
        "deletion_proof_reason_taxonomy_version={DELETION_REASON_TAXONOMY_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "deletion_proof_reason_codes_csv={DELETION_REASON_CODES_CSV}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "deletion_docs_parity_reason_taxonomy_version={DELETION_REASON_TAXONOMY_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "deletion_docs_parity_reason_codes_csv={DELETION_REASON_CODES_CSV}"
    )));
}

fn assert_deletion_fixture_markers() {
    assert!(DELETION_FIXTURE.contains("deletion_proof_fixture_matrix_schema_version=kamn.runtime.deletion-proof-fixture-matrix.v1"));
    assert!(DELETION_FIXTURE.contains(&format!(
        "deletion_proof_reason_taxonomy_version={DELETION_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DELETION_FIXTURE.contains(&format!(
        "deletion_proof_reason_codes_csv={DELETION_REASON_CODES_CSV}"
    )));
}

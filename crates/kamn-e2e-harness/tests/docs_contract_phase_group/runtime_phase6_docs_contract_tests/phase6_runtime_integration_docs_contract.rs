use super::super::support::*;

#[test]
fn spec_c11_phase6_runtime_integration_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase6-runtime-integration-gap-analysis.md",
        "phase-6 runtime integration docs marker artifact should exist",
        &[
        "phase6_runtime_integration_status_before=partial",
        "phase6_runtime_integration_guard_contract=implemented",
        "phase6_runtime_integration_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c12_milestone_index_references_active_phase6_runtime_integration_issue() {
    assert_milestone_markers(&[
        "#5600",
    ]);
}

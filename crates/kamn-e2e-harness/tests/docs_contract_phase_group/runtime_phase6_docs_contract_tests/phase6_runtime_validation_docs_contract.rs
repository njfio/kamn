use super::super::support::*;

#[test]
fn spec_c15_phase6_runtime_validation_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase6-runtime-validation-gap-analysis.md",
        "phase-6 runtime validation docs marker artifact should exist",
        &[
        "phase6_runtime_validation_status_before=partial",
        "phase6_runtime_validation_contract=implemented",
        "phase6_runtime_validation_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c16_milestone_index_references_active_phase6_runtime_validation_issue() {
    assert_milestone_markers(&[
        "#5606",
        "Active issue(s): None",
        "25. Phase-6 runtime external validation execution. (Completed)",
    ]);
}

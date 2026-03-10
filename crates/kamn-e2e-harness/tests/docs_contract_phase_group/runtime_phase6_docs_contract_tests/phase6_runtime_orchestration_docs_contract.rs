use super::super::support::*;

#[test]
fn spec_c13_phase6_runtime_orchestration_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase6-runtime-orchestration-gap-analysis.md",
        "phase-6 runtime orchestration docs marker artifact should exist",
        &[
        "phase6_runtime_orchestration_status_before=partial",
        "phase6_runtime_orchestration_contract=implemented",
        "phase6_runtime_orchestration_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c14_milestone_index_references_active_phase6_runtime_orchestration_issue() {
    assert_milestone_markers(&[
        "#5602",
    ]);
}

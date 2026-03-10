use super::super::support::*;

#[test]
fn spec_c10_phase6a_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase6a-gap-analysis.md",
        "phase-6a docs marker artifact should exist",
        &[
        "phase6a_status_before=partial",
        "phase6a_spawn_plan_contract=implemented",
        "phase6a_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c11_milestone_index_references_active_phase6a_issue() {
    assert_milestone_markers(&[
        "#5592",
    ]);
}

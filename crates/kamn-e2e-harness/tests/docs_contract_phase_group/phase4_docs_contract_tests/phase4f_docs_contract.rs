use super::super::support::*;

#[test]
fn spec_c08_phase4f_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase4f-gap-analysis.md",
        "phase-4f docs marker artifact should exist",
        &[
        "phase4f_status_before=partial",
        "phase4f_mode_aware_rules=implemented",
        "phase4f_controlled_fail_path=implemented",
        "phase4f_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c09_milestone_index_references_active_phase4f_issue() {
    assert_milestone_markers(&[
        "#5574",
    ]);
}

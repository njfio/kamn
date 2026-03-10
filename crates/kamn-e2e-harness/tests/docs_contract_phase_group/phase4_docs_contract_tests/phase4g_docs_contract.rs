use super::super::support::*;

#[test]
fn spec_c07_phase4g_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase4g-gap-analysis.md",
        "phase-4g docs marker artifact should exist",
        &[
        "phase4g_status_before=partial",
        "phase4g_lifecycle_summary=implemented",
        "phase4g_fail_path_summary=implemented",
        "phase4g_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c08_milestone_index_references_active_phase4g_issue() {
    assert_milestone_markers(&[
        "#5576",
    ]);
}

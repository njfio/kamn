use super::super::support::*;

#[test]
fn spec_c07_phase4e_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase4e-gap-analysis.md",
        "phase-4e docs marker artifact should exist",
        &[
        "phase4e_status_before=partial",
        "phase4e_step_record_model=implemented",
        "phase4e_infra_step_markers=implemented",
        "phase4e_agent_deploy_step_markers=implemented",
        "phase4e_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c08_milestone_index_references_active_phase4e_issue() {
    assert_milestone_markers(&[
        "#5572",
    ]);
}

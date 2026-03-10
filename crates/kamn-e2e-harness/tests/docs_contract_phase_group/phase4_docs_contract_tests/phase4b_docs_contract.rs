use super::super::support::*;

#[test]
fn spec_c10_phase4b_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase4b-gap-analysis.md",
        "phase-4b docs marker artifact should exist",
        &[
        "phase4b_status_before=partial",
        "phase4b_run_command_contract=implemented",
        "phase4b_verify_command_contract=implemented",
        "phase4b_scenario_csv_validation=implemented",
        "phase4b_verify_output_contract=implemented",
        "phase4b_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c10_milestone_index_references_active_phase4b_issue() {
    assert_milestone_markers(&[
        "#5566",
    ]);
}

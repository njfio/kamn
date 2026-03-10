use super::super::support::*;

#[test]
fn spec_c01_r53_mode_execution_contract_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-r53-mode-execution-contract.md",
        "r53 mode-execution docs marker artifact should exist",
        &[
            "r53_mode_execution_contract_status_before=implicit",
            "r53_mode_execution_contract_contract=implemented",
            "r53_mode_execution_contract_status_after=active",
        ],
    );
}

#[test]
fn spec_c02_r53_milestone_index_references_active_issue_5626() {
    assert_milestone_markers(
        "specs/milestones/r53-e2e-scenario-execution-activation/index.md",
        "r53 milestone index should exist",
        &["#5626"],
    );
}

#[test]
fn spec_c03_r53_milestone_index_marks_milestone_closed() {
    assert_milestone_markers(
        "specs/milestones/r53-e2e-scenario-execution-activation/index.md",
        "r53 milestone index should exist",
        &[
            "Active issue(s): None",
            "Completed issue(s): #5620, #5622, #5624, #5626",
            "4. Mode execution contract parity across sdk-direct/cli-scripted/mcp-* drivers. (Completed)",
        ],
    );
}

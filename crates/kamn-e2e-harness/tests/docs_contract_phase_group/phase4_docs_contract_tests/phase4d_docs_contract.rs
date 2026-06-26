use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c08_phase4d_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase4d-gap-analysis.md",
    "phase-4d docs marker artifact should exist",
    [
        "phase4d_status_before=partial",
        "phase4d_phase_result_model=implemented",
        "phase4d_infra_and_agent_placeholders=implemented",
        "phase4d_status_after=implemented"
    ],
    spec_c09_milestone_index_references_active_phase4d_issue,
    ["#5570"]
);

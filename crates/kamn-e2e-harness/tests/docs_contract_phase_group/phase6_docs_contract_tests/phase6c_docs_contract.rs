use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c14_phase6c_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase6c-gap-analysis.md",
    "phase-6c docs marker artifact should exist",
    [
        "phase6c_status_before=partial",
        "phase6c_live_process_execution_contract=implemented",
        "phase6c_status_after=implemented"
    ],
    spec_c15_milestone_index_references_active_phase6c_issue,
    ["#5596"]
);

use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c09_phase5a_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase5a-gap-analysis.md",
    "phase-5a docs marker artifact should exist",
    [
        "phase5a_status_before=partial",
        "phase5a_process_runtime_contract=implemented",
        "phase5a_status_after=implemented"
    ],
    spec_c10_milestone_index_references_active_phase5a_issue,
    ["#5584"]
);

use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c16_phase6d_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase6d-gap-analysis.md",
    "phase-6d docs marker artifact should exist",
    ["phase6d_status_before=partial", "phase6d_live_execution_contract=implemented", "phase6d_status_after=implemented"],
    spec_c17_milestone_index_references_active_phase6d_issue,
    ["#5598"]
);

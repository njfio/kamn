use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c08_phase4h_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase4h-gap-analysis.md",
    "phase-4h docs marker artifact should exist",
    ["phase4h_status_before=partial", "phase4h_runtime_binary_contract=implemented", "phase4h_status_after=implemented"],
    spec_c09_milestone_index_references_active_phase4h_issue,
    ["#5578"]
);

use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c12_phase6b_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase6b-gap-analysis.md",
    "phase-6b docs marker artifact should exist",
    ["phase6b_status_before=partial", "phase6b_spawn_execution_contract=implemented", "phase6b_status_after=implemented"],
    spec_c13_milestone_index_references_active_phase6b_issue,
    ["#5594"]
);

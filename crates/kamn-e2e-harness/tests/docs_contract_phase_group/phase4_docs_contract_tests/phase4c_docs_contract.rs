use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c07_phase4c_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase4c-gap-analysis.md",
    "phase-4c docs marker artifact should exist",
    [
        "phase4c_status_before=partial",
        "phase4c_orchestration_phase_model=implemented",
        "phase4c_phase_progression_markers=implemented",
        "phase4c_status_after=implemented"
    ],
    spec_c08_milestone_index_references_active_phase4c_issue,
    ["#5568"]
);

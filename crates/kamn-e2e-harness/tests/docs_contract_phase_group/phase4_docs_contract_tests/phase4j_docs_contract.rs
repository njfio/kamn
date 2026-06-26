use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c07_phase4j_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase4j-gap-analysis.md",
    "phase-4j docs marker artifact should exist",
    [
        "phase4j_status_before=partial",
        "phase4j_runtime_readiness_contract=implemented",
        "phase4j_status_after=implemented"
    ],
    spec_c08_milestone_index_references_active_phase4j_issue,
    ["#5582"]
);

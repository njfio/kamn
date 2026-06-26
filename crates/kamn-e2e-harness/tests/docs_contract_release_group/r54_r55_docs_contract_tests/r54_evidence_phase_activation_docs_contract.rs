use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r54_evidence_phase_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r54-evidence-phase-activation.md",
    "r54 evidence-phase docs marker artifact should exist",
    [
        "r54_evidence_phase_status_before=static-skip",
        "r54_evidence_phase_contract=implemented",
        "r54_evidence_phase_status_after=active"
    ],
    spec_c02_r54_milestone_index_references_active_issue_5629,
    "specs/milestones/r54-e2e-evidence-phase-activation/index.md",
    "r54 milestone index should exist",
    ["#5629"]
);

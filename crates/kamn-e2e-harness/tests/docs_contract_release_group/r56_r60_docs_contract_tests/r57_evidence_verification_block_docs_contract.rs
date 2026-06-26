use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r57_evidence_verification_block_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r57-evidence-verification-block-enforcement.md",
    "r57 evidence verification block docs marker artifact should exist",
    [
        "r57_evidence_verification_block_contract_status_before=missing",
        "r57_verify_artifact_verification_marker_enforcement=implemented",
        "r57_evidence_verification_block_contract_status_after=implemented"
    ],
    spec_c02_r57_milestone_index_references_issue_5640,
    "specs/milestones/r57-e2e-evidence-verification-block-enforcement/index.md",
    "r57 milestone index should exist",
    ["#5640"]
);

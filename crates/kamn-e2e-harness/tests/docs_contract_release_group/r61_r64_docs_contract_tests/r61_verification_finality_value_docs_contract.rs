use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r61_verification_finality_value_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r61-verification-finality-value-contract.md",
    "r61 finality value docs marker artifact should exist",
    ["r61_verification_finality_value_contract_status_before=missing", "r61_verify_artifact_finality_value_enforcement=implemented", "r61_verification_finality_value_contract_status_after=implemented"],
    spec_c02_r61_milestone_index_references_issue_5652,
    "specs/milestones/r61-e2e-verification-finality-value-contract/index.md",
    "r61 milestone index should exist",
    ["#5652"]
);

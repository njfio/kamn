use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r62_verification_hash_format_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r62-verification-hash-format-contract.md",
    "r62 hash-format docs marker artifact should exist",
    [
        "r62_verification_hash_format_contract_status_before=missing",
        "r62_verify_artifact_hash_format_enforcement=implemented",
        "r62_verification_hash_format_contract_status_after=implemented"
    ],
    spec_c02_r62_milestone_index_references_issue_5655,
    "specs/milestones/r62-e2e-verification-hash-format-contract/index.md",
    "r62 milestone index should exist",
    ["#5655"]
);

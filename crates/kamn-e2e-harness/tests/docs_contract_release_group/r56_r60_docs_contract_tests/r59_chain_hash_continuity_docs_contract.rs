use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r59_chain_hash_continuity_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r59-chain-hash-continuity-verification.md",
    "r59 chain hash continuity docs marker artifact should exist",
    [
        "r59_chain_hash_continuity_contract_status_before=missing",
        "r59_verify_chain_hash_continuity_enforcement=implemented",
        "r59_chain_hash_continuity_contract_status_after=implemented"
    ],
    spec_c02_r59_milestone_index_references_issue_5646,
    "specs/milestones/r59-e2e-chain-hash-continuity-verification-contract/index.md",
    "r59 milestone index should exist",
    ["#5646"]
);

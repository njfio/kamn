use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r60_chain_genesis_anchor_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r60-chain-genesis-anchor-verification.md",
    "r60 chain genesis anchor docs marker artifact should exist",
    ["r60_chain_genesis_anchor_contract_status_before=missing", "r60_verify_chain_genesis_anchor_enforcement=implemented", "r60_chain_genesis_anchor_contract_status_after=implemented"],
    spec_c02_r60_milestone_index_references_issue_5649,
    "specs/milestones/r60-e2e-chain-genesis-anchor-verification-contract/index.md",
    "r60 milestone index should exist",
    ["#5649"]
);

use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r58_chain_dump_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r58-chain-dump-verification-hardening.md",
    "r58 chain dump docs marker artifact should exist",
    ["r58_chain_dump_marker_contract_status_before=missing", "r58_verify_chain_dump_marker_enforcement=implemented", "r58_chain_dump_marker_contract_status_after=implemented"],
    spec_c02_r58_milestone_index_references_issue_5643,
    "specs/milestones/r58-e2e-chain-dump-verification-contract-hardening/index.md",
    "r58 milestone index should exist",
    ["#5643"]
);

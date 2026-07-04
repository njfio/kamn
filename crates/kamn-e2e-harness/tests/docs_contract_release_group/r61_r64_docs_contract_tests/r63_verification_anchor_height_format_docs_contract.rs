use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r63_verification_anchor_height_format_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r63-verification-anchor-height-format-contract.md",
    "r63 anchor-height format docs marker artifact should exist",
    [
        "r63_verification_anchor_height_format_contract_status_before=missing",
        "r63_verify_anchor_block_height_format_enforcement=implemented",
        "r63_verification_anchor_height_format_contract_status_after=implemented"
    ],
    spec_c02_r63_milestone_index_references_issue_5658,
    "specs/milestones/r63-e2e-verification-anchor-height-format-contract/index.md",
    "r63 milestone index should exist",
    ["#5658"]
);

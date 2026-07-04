use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r64_verification_captured_at_format_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r64-verification-captured-at-format-contract.md",
    "r64 captured-at format docs marker artifact should exist",
    [
        "r64_verification_captured_at_format_contract_status_before=missing",
        "r64_verify_captured_at_format_enforcement=implemented",
        "r64_verification_captured_at_format_contract_status_after=implemented"
    ],
    spec_c02_r64_milestone_index_references_issue_5661,
    "specs/milestones/r64-e2e-verification-captured-at-format-contract/index.md",
    "r64 milestone index should exist",
    ["#5661"]
);

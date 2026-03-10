use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r56_verify_manifest_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r56-verify-manifest-hardening.md",
    "r56 verify-manifest docs marker artifact should exist",
    ["r56_verify_manifest_nested_field_contract_status_before=partial", "r56_verify_manifest_infrastructure_marker_enforcement=implemented", "r56_verify_manifest_summary_marker_enforcement=implemented", "r56_verify_manifest_nested_field_contract_status_after=implemented"],
    spec_c02_r56_milestone_index_references_issue_5637,
    "specs/milestones/r56-e2e-verify-manifest-contract-hardening/index.md",
    "r56 milestone index should exist",
    ["#5637"]
);

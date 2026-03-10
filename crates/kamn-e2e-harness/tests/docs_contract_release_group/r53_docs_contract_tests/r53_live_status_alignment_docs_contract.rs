use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r53_live_status_alignment_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r53-live-status-alignment.md",
    "r53 live-status docs marker artifact should exist",
    ["r53_live_status_alignment_status_before=static-pass", "r53_live_status_alignment_contract=implemented", "r53_live_status_alignment_status_after=active"],
    spec_c02_r53_milestone_index_references_active_issue_5622,
    "specs/milestones/r53-e2e-scenario-execution-activation/index.md",
    "r53 milestone index should exist",
    ["#5622"]
);

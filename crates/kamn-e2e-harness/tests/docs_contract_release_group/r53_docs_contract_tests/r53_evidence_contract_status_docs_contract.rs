use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r53_evidence_contract_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r53-evidence-contract-status.md",
    "r53 evidence-contract docs marker artifact should exist",
    [
        "r53_evidence_contract_status_before=implicit",
        "r53_evidence_contract_contract=implemented",
        "r53_evidence_contract_status_after=active"
    ],
    spec_c02_r53_milestone_index_references_active_issue_5624,
    "specs/milestones/r53-e2e-scenario-execution-activation/index.md",
    "r53 milestone index should exist",
    ["#5624"]
);

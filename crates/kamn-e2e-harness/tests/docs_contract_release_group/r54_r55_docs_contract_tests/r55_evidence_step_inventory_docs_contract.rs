use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r55_evidence_step_inventory_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r55-evidence-step-inventory.md",
    "r55 evidence-step docs marker artifact should exist",
    [
        "r55_evidence_step_inventory_status_before=single-step",
        "r55_evidence_step_inventory_contract=implemented",
        "r55_evidence_step_inventory_status_after=active"
    ],
    spec_c02_r55_milestone_index_references_issue_5634,
    "specs/milestones/r55-e2e-evidence-step-inventory-parity/index.md",
    "r55 milestone index should exist",
    ["#5634"]
);

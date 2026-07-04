use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c12_r52_preflight_executable_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r52-preflight-executable-diagnostics.md",
    "r52 preflight executable docs marker artifact should exist",
    [
        "r52_preflight_executable_status_before=partial",
        "r52_preflight_executable_contract=implemented",
        "r52_preflight_executable_status_after=implemented"
    ],
    spec_c13_r52_milestone_index_references_active_issue,
    "specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md",
    "r52 milestone index should exist",
    ["#5610"]
);

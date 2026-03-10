use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c16_r52_preflight_absolute_path_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r52-preflight-absolute-path-diagnostics.md",
    "r52 preflight absolute-path docs marker artifact should exist",
    ["r52_preflight_absolute_path_status_before=partial", "r52_preflight_absolute_path_contract=implemented", "r52_preflight_absolute_path_status_after=implemented"],
    spec_c17_r52_milestone_index_references_active_issue,
    "specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md",
    "r52 milestone index should exist",
    ["#5615"]
);

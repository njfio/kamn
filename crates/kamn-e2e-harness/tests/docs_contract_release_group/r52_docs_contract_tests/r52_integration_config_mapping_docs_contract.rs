use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c18_r52_integration_config_mapping_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r52-integration-config-mapping-fix.md",
    "r52 integration-config mapping docs marker artifact should exist",
    ["r52_integration_config_mapping_status_before=buggy", "r52_integration_config_mapping_contract=implemented", "r52_integration_config_mapping_status_after=fixed"],
    spec_c19_r52_milestone_index_references_active_issue,
    "specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md",
    "r52 milestone index should exist",
    ["#5617"]
);

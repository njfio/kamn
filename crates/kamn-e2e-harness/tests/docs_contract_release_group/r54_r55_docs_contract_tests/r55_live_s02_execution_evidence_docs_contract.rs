use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r55_live_s02_execution_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r55-live-s02-execution-evidence.md",
    "r55 live s02 execution docs marker artifact should exist",
    ["r55_live_s02_execution_schema_version=kamn.e2e.live-s02-execution.v1", "r55_live_s02_execution_modes_executed=3", "r55_live_s02_execution_scenarios_executed_csv=S-01,S-02,S-04,S-06", "r55_live_s02_execution_sdk_direct_status=pass", "r55_live_s02_execution_cli_scripted_status=pass", "r55_live_s02_execution_mcp_tau_status=pass", "r55_live_s02_execution_s02_sdk_direct_status=pass", "r55_live_s02_execution_s02_cli_scripted_status=pass", "r55_live_s02_execution_s02_mcp_tau_status=pass", "r55_live_s02_execution_overall_status=pass"],
    spec_c02_r52_milestone_index_references_issue_5812,
    "specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md",
    "r52 milestone index should exist",
    ["#5812"]
);

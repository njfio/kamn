use super::super::support::release_doc_contract_tests;

release_doc_contract_tests!(
    spec_c01_r53_scenario_run_execution_docs_markers_present,
    "docs/research/e2e-live-testing-prd-r53-scenario-run-execution.md",
    "r53 scenario-run docs marker artifact should exist",
    ["r53_scenario_run_execution_status_before=scaffold-skip", "r53_scenario_run_execution_contract=implemented", "r53_scenario_run_execution_status_after=active"],
    spec_c02_r53_milestone_index_references_active_issue,
    "specs/milestones/r53-e2e-scenario-execution-activation/index.md",
    "r53 milestone index should exist",
    ["#5620"]
);

use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c15_phase6_runtime_validation_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase6-runtime-validation-gap-analysis.md",
    "phase-6 runtime validation docs marker artifact should exist",
    ["phase6_runtime_validation_status_before=partial", "phase6_runtime_validation_contract=implemented", "phase6_runtime_validation_status_after=implemented"],
    spec_c16_milestone_index_references_active_phase6_runtime_validation_issue,
    ["#5606", "Active issue(s): None", "25. Phase-6 runtime external validation execution. (Completed)"]
);

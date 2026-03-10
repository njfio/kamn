use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c13_phase6_runtime_lifecycle_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase6-runtime-lifecycle-gap-analysis.md",
    "phase-6 runtime lifecycle docs marker artifact should exist",
    ["phase6_runtime_lifecycle_status_before=partial", "phase6_runtime_lifecycle_contract=implemented", "phase6_runtime_lifecycle_status_after=implemented"],
    spec_c14_milestone_index_references_active_phase6_runtime_lifecycle_issue,
    ["#5604"]
);

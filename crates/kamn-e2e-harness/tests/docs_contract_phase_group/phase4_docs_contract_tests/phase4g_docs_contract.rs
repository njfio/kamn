use super::super::support::phase_doc_contract_tests;

phase_doc_contract_tests!(
    spec_c07_phase4g_docs_markers_present,
    "docs/research/e2e-live-testing-prd-phase4g-gap-analysis.md",
    "phase-4g docs marker artifact should exist",
    ["phase4g_status_before=partial", "phase4g_lifecycle_summary=implemented", "phase4g_fail_path_summary=implemented", "phase4g_status_after=implemented"],
    spec_c08_milestone_index_references_active_phase4g_issue,
    ["#5576"]
);

const DOC: &str =
    include_str!("../../../docs/research/e2e-live-testing-prd-phase1-gap-analysis.md");

#[test]
fn spec_c07_phase1_gap_analysis_markers_present() {
    assert!(DOC.contains("phase1_required_paths_total=12"));
    assert!(DOC.contains("phase1_required_paths_present_before=0"));
    assert!(DOC.contains("phase1_required_paths_missing_before=12"));
    assert!(DOC.contains("phase1_required_paths_present_after=12"));
    assert!(DOC.contains("phase1_required_paths_missing_after=0"));
    assert!(DOC.contains("phase1_status_after=implemented"));
    assert!(DOC.contains("phase1_blockers_remaining=0"));
}

#[test]
fn spec_c08_phase1_task_escrow_stub_replacement_markers_present() {
    assert!(DOC.contains("phase1_task_escrow_stub_ops_remaining_before=4"));
    assert!(DOC.contains("phase1_task_escrow_stub_ops_remaining_after=0"));
    assert!(DOC.contains("phase1_task_escrow_route_expansion_status=implemented"));
    assert!(DOC.contains("phase1_agent_lib_stub_replacement_status=implemented"));
}

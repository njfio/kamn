const DETERMINISTIC_PROPOSAL_PLANNER_TEST_SURFACE: &str =
    include_str!("deterministic_proposal_planner_integration.rs");

#[test]
fn contract_deterministic_proposal_planner_surface_exists_with_expected_markers() {
    for marker in [
        "integration_deterministic_proposal_planner_valid_plan_returns_expected_order",
        "integration_deterministic_proposal_planner_invalid_candidates_fail_closed",
        "integration_deterministic_proposal_planner_duplicate_ids_and_stale_state_fail_closed",
    ] {
        assert!(
            DETERMINISTIC_PROPOSAL_PLANNER_TEST_SURFACE.contains(marker),
            "missing marker: {marker}"
        );
    }
}

fn expected_sorted_symmetric_lane_ids() -> Vec<&'static str> {
    vec![
        "listener_approver_parallel_applied",
        "listener_approver_parallel_deferred",
        "processor_listener_parallel_applied",
        "processor_listener_parallel_deferred",
    ]
}

fn expected_sorted_asymmetric_lane_ids() -> Vec<&'static str> {
    vec![
        "listener_approver_asymmetric_parallel_applied",
        "listener_approver_asymmetric_parallel_deferred",
        "processor_listener_asymmetric_parallel_applied",
        "processor_listener_asymmetric_parallel_deferred",
    ]
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_order_invariance_contract_is_canonical(
) {
    let lane_sets_csv = ["symmetric_parallel", "asymmetric_parallel"].join(",");
    assert_eq!(
        lane_sets_csv,
        LIVE_POSTGRES_MATRIX_ORDER_INVARIANCE_LANE_SETS_CSV
    );
    assert_eq!(
        sorted_lane_ids(project_live_postgres_parallel_role_pair_lanes),
        expected_sorted_symmetric_lane_ids()
    );
    assert_eq!(
        sorted_lane_ids(project_live_postgres_asymmetric_parallel_lanes),
        expected_sorted_asymmetric_lane_ids()
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_order_is_invariant(
) {
    let Some(_context) = live_postgres_validation_context() else {
        return;
    };
    assert_lane_order_invariant("symmetric", project_live_postgres_parallel_role_pair_lanes);
    assert_lane_order_invariant("asymmetric", project_live_postgres_asymmetric_parallel_lanes);
}

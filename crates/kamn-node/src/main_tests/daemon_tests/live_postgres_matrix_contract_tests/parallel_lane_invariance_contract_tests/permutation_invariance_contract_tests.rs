fn expected_base_symmetric_lane_ids() -> Vec<&'static str> {
    vec![
        "processor_listener_parallel_applied",
        "processor_listener_parallel_deferred",
        "listener_approver_parallel_applied",
        "listener_approver_parallel_deferred",
    ]
}

fn expected_reverse_symmetric_lane_ids() -> Vec<&'static str> {
    vec![
        "listener_approver_parallel_deferred",
        "listener_approver_parallel_applied",
        "processor_listener_parallel_deferred",
        "processor_listener_parallel_applied",
    ]
}

fn expected_rotate_left_symmetric_lane_ids() -> Vec<&'static str> {
    vec![
        "processor_listener_parallel_deferred",
        "listener_approver_parallel_applied",
        "listener_approver_parallel_deferred",
        "processor_listener_parallel_applied",
    ]
}

fn expected_interleaved_symmetric_lane_ids() -> Vec<&'static str> {
    vec![
        "processor_listener_parallel_applied",
        "listener_approver_parallel_applied",
        "processor_listener_parallel_deferred",
        "listener_approver_parallel_deferred",
    ]
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_permutation_contract_is_canonical(
) {
    let permutations = permutation_ids();
    assert_eq!(LIVE_POSTGRES_MATRIX_PERMUTATION_IDS_CSV, permutations.join(","));
    assert_eq!(
        lane_ids(&project_live_postgres_parallel_role_pair_lanes()),
        expected_base_symmetric_lane_ids()
    );
    assert_eq!(
        permuted_lane_ids(project_live_postgres_parallel_role_pair_lanes, "reverse"),
        expected_reverse_symmetric_lane_ids()
    );
    assert_eq!(
        permuted_lane_ids(project_live_postgres_parallel_role_pair_lanes, "rotate_left_1"),
        expected_rotate_left_symmetric_lane_ids()
    );
    assert_eq!(
        permuted_lane_ids(
            project_live_postgres_parallel_role_pair_lanes,
            "interleaved_even_then_odd",
        ),
        expected_interleaved_symmetric_lane_ids()
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant(
) {
    let Some(_context) = live_postgres_validation_context() else {
        return;
    };
    assert_lane_permutations_invariant(
        "symmetric",
        project_live_postgres_parallel_role_pair_lanes,
    );
    assert_lane_permutations_invariant(
        "asymmetric",
        project_live_postgres_asymmetric_parallel_lanes,
    );
}

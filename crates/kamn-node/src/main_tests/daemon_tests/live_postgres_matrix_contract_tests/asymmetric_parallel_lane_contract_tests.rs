#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_asymmetric_parallel_lane_contract_is_canonical(
) {
    let lanes = project_live_postgres_asymmetric_parallel_lanes();
    let lane_ids_csv = lanes
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        lane_ids_csv,
        LIVE_POSTGRES_MATRIX_ASYMMETRIC_PARALLEL_LANE_IDS_CSV
    );
    assert!(lanes
        .iter()
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(lanes
        .iter()
        .skip(1)
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_asymmetric_parallel_lane_is_deterministic(
) {
    let Some(_context) = live_postgres_validation_context() else {
        return;
    };
    for lane in project_live_postgres_asymmetric_parallel_lanes() {
        assert_parallel_role_pair_projection(&lane, "asymmetric lane");
    }
}

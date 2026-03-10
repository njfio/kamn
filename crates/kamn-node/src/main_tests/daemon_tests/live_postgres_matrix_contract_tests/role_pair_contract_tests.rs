#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_role_pair_matrix_contract_is_canonical()
{
    let pairs = project_live_postgres_role_pair_profiles();
    let pair_ids_csv = pairs
        .iter()
        .map(|pair| pair.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(pair_ids_csv, LIVE_POSTGRES_MATRIX_ROLE_PAIR_IDS_CSV);
    assert!(pairs
        .iter()
        .step_by(2)
        .all(|pair| pair.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(pairs
        .iter()
        .skip(1)
        .step_by(2)
        .all(|pair| pair.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_role_pair_matrix_is_deterministic(
) {
    let Some(_context) = live_postgres_validation_context() else {
        return;
    };
    for pair in project_live_postgres_role_pair_profiles() {
        assert_serial_role_pair_projection(&pair);
    }
}

fn expected_live_postgres_matrix_rows() -> Vec<LivePostgresMatrixRow> {
    vec![
        LivePostgresMatrixRow {
            scenario_id: "env_unset",
            gate_reason_code: LIVE_POSTGRES_ENV_UNSET_REASON_CODE,
            daemon_phase6_reason_code: None,
        },
        LivePostgresMatrixRow {
            scenario_id: "env_set_no_shutdown",
            gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE),
        },
        LivePostgresMatrixRow {
            scenario_id: "env_set_shutdown",
            gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE),
        },
    ]
}

fn daemon_to_matrix_bridge_reason_codes_csv(rows: &[LivePostgresMatrixRow]) -> String {
    rows.iter()
        .filter_map(|row| row.daemon_phase6_reason_code)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical(
) {
    let rows = project_live_postgres_matrix_rows();
    assert_eq!(rows, expected_live_postgres_matrix_rows(), "matrix projection rows must remain canonical and ordered");
    let scenario_csv = rows
        .iter()
        .map(|row| row.scenario_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(scenario_csv, LIVE_POSTGRES_MATRIX_SCENARIOS_CSV);

    let reason_codes_csv = format!(
        "{LIVE_POSTGRES_ENV_UNSET_REASON_CODE},{LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE},{LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE}"
    );
    assert_eq!(reason_codes_csv, LIVE_POSTGRES_MATRIX_REASON_CODES_CSV);
    assert_eq!(
        LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    );
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_matrix_taxonomy_bridge_contract_is_canonical(
) {
    let rows = project_live_postgres_matrix_rows();
    let bridge_reason_codes_csv = daemon_to_matrix_bridge_reason_codes_csv(&rows);
    assert_eq!(
        LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    );
    assert_eq!(
        bridge_reason_codes_csv,
        LIVE_POSTGRES_RUNTIME_TO_MATRIX_BRIDGE_REASON_CODES_CSV
    );
}

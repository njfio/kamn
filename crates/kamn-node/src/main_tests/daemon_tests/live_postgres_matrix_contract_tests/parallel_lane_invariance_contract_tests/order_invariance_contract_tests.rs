#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_order_invariance_contract_is_canonical(
) {
    let lane_sets_csv = ["symmetric_parallel", "asymmetric_parallel"].join(",");
    assert_eq!(
        lane_sets_csv,
        LIVE_POSTGRES_MATRIX_ORDER_INVARIANCE_LANE_SETS_CSV
    );

    let mut expected_symmetric_fingerprints = project_live_postgres_parallel_role_pair_lanes()
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    expected_symmetric_fingerprints.sort();

    let mut expected_asymmetric_fingerprints = project_live_postgres_asymmetric_parallel_lanes()
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    expected_asymmetric_fingerprints.sort();

    assert_eq!(
        expected_symmetric_fingerprints,
        vec![
            "listener_approver_parallel_applied",
            "listener_approver_parallel_deferred",
            "processor_listener_parallel_applied",
            "processor_listener_parallel_deferred"
        ]
    );
    assert_eq!(
        expected_asymmetric_fingerprints,
        vec![
            "listener_approver_asymmetric_parallel_applied",
            "listener_approver_asymmetric_parallel_deferred",
            "processor_listener_asymmetric_parallel_applied",
            "processor_listener_asymmetric_parallel_deferred"
        ]
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_order_is_invariant(
) {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

    let symmetric_baseline =
        run_parallel_lane_set_fingerprints(project_live_postgres_parallel_role_pair_lanes());
    let mut symmetric_permuted_lanes = project_live_postgres_parallel_role_pair_lanes();
    symmetric_permuted_lanes.reverse();
    let symmetric_permuted = run_parallel_lane_set_fingerprints(symmetric_permuted_lanes);
    assert_eq!(
        symmetric_baseline, symmetric_permuted,
        "symmetric parallel lane fingerprints should remain invariant to lane order"
    );

    let asymmetric_baseline =
        run_parallel_lane_set_fingerprints(project_live_postgres_asymmetric_parallel_lanes());
    let mut asymmetric_permuted_lanes = project_live_postgres_asymmetric_parallel_lanes();
    asymmetric_permuted_lanes.reverse();
    let asymmetric_permuted = run_parallel_lane_set_fingerprints(asymmetric_permuted_lanes);
    assert_eq!(
        asymmetric_baseline, asymmetric_permuted,
        "asymmetric parallel lane fingerprints should remain invariant to lane order"
    );
}


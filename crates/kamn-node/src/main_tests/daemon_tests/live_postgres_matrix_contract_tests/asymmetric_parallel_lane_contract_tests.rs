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

    for lane in project_live_postgres_asymmetric_parallel_lanes() {
        let (leg_a_first, leg_b_first) =
            run_parallel_phase6_projections(lane.leg_a_args.clone(), lane.leg_b_args.clone());
        let (leg_a_second, leg_b_second) =
            run_parallel_phase6_projections(lane.leg_a_args, lane.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, lane.expected_reason_code,
            "asymmetric lane {} leg A ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, lane.expected_reason_code,
            "asymmetric lane {} leg B ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "asymmetric lane {} leg A reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "asymmetric lane {} leg B reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "asymmetric lane {} leg A taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "asymmetric lane {} leg B taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "asymmetric lane {} leg A taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "asymmetric lane {} leg B taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "asymmetric lane {} legs should share the same runtime taxonomy version",
            lane.pair_id
        );
    }
}


#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains(
        "\"daemon_phase6_runtime_reason_taxonomy_version\":\"kamn.runtime.daemon.phase6.reason-taxonomy.v1\""
    ));
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_applied\""));
}

#[test]
fn regression_runtime_daemon_live_postgres_validation_slice_reports_unset_env_gate_reason() {
    // Regression: #5340
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _test_postgres_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", None);
    let _database_guard = EnvVarGuard::set("DATABASE_URL", None);
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
    assert!(maybe_database_url.is_none());
}

#[test]
fn unit_runtime_daemon_live_postgres_validation_slice_prefers_kamn_test_postgres_url() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let preferred = "postgres://preferred:5432/kamn_test";
    let fallback = "postgres://fallback:5432/kamn_test";
    let _test_postgres_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some(preferred));
    let _database_guard = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );
    assert_eq!(maybe_database_url.as_deref(), Some(preferred));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_deferred_path() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_deferred\""));
    assert!(rendered.contains("\"daemon_phase6_runtime_deferred_cycles\":1"));
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_env_matrix_contract_is_deterministic() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");

    let _unset_primary = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", None);
    let _unset_fallback = EnvVarGuard::set("DATABASE_URL", None);
    let (reason_unset, url_unset) = resolve_live_postgres_gate_decision();
    assert_eq!(reason_unset, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
    assert!(url_unset.is_none());
    drop(_unset_fallback);
    drop(_unset_primary);

    let preferred = "postgres://preferred:5432/kamn_test";
    let fallback = "postgres://fallback:5432/kamn_test";
    let _preferred_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some(preferred));
    let _fallback_guard = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (reason_preferred, url_preferred) = resolve_live_postgres_gate_decision();
    assert_eq!(
        reason_preferred,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );
    assert_eq!(url_preferred.as_deref(), Some(preferred));
    drop(_fallback_guard);
    drop(_preferred_guard);

    let _blank_primary = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some("   "));
    let _fallback_only = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (reason_fallback, url_fallback) = resolve_live_postgres_gate_decision();
    assert_eq!(reason_fallback, LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE);
    assert_eq!(url_fallback.as_deref(), Some(fallback));
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical(
) {
    let rows = project_live_postgres_matrix_rows();
    assert_eq!(
        rows,
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
        ],
        "matrix projection rows must remain canonical and ordered"
    );
    let scenario_csv = rows
        .iter()
        .map(|row| row.scenario_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(scenario_csv, LIVE_POSTGRES_MATRIX_SCENARIOS_CSV);

    let reason_codes_csv = format!(
        "{},{},{}",
        LIVE_POSTGRES_ENV_UNSET_REASON_CODE,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
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
    let bridge_reason_codes_csv = rows
        .iter()
        .filter_map(|row| row.daemon_phase6_reason_code)
        .collect::<Vec<_>>()
        .join(",");
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

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical(
) {
    let profiles = project_live_postgres_load_profiles();
    let profile_ids_csv = profiles
        .iter()
        .map(|profile| profile.profile_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(profile_ids_csv, LIVE_POSTGRES_MATRIX_LOAD_PROFILE_IDS_CSV);

    assert!(profiles
        .iter()
        .take(3)
        .all(|profile| profile.expected_reason_code
            == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(profiles
        .iter()
        .skip(3)
        .all(|profile| profile.expected_reason_code
            == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_load_profile_matrix_is_deterministic(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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

    for profile in project_live_postgres_load_profiles() {
        let first = run_daemon_for_phase6_projection(profile.args.clone());
        let second = run_daemon_for_phase6_projection(profile.args);
        assert_eq!(
            first.reason_code, profile.expected_reason_code,
            "profile {} should project expected phase6 reason code",
            profile.profile_id
        );
        assert_eq!(
            first.reason_code, second.reason_code,
            "profile {} reason code should remain stable across repeated runs",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "profile {} should remain bridged to runtime taxonomy version",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, second.reason_taxonomy_version,
            "profile {} taxonomy version should remain stable across repeated runs",
            profile.profile_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_role_profile_matrix_contract_is_canonical(
) {
    let profiles = project_live_postgres_role_profiles();
    let profile_ids_csv = profiles
        .iter()
        .map(|profile| profile.profile_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(profile_ids_csv, LIVE_POSTGRES_MATRIX_ROLE_PROFILE_IDS_CSV);

    assert_eq!(
        profiles[0].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[1].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(
        profiles[2].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[3].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(
        profiles[4].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[5].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_role_profile_matrix_is_deterministic(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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

    for profile in project_live_postgres_role_profiles() {
        let first = run_daemon_for_phase6_projection(profile.args.clone());
        let second = run_daemon_for_phase6_projection(profile.args);
        assert_eq!(
            first.reason_code, profile.expected_reason_code,
            "role profile {} should project expected phase6 reason code",
            profile.profile_id
        );
        assert_eq!(
            first.reason_code, second.reason_code,
            "role profile {} reason code should remain stable across repeated runs",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "role profile {} should remain bridged to runtime taxonomy version",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, second.reason_taxonomy_version,
            "role profile {} taxonomy version should remain stable across repeated runs",
            profile.profile_id
        );
    }
}

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
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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

    for pair in project_live_postgres_role_pair_profiles() {
        let leg_a_first = run_daemon_for_phase6_projection(pair.leg_a_args.clone());
        let leg_a_second = run_daemon_for_phase6_projection(pair.leg_a_args);
        let leg_b_first = run_daemon_for_phase6_projection(pair.leg_b_args.clone());
        let leg_b_second = run_daemon_for_phase6_projection(pair.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, pair.expected_reason_code,
            "pair {} leg A ({}) should project expected phase6 reason code",
            pair.pair_id, pair.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, pair.expected_reason_code,
            "pair {} leg B ({}) should project expected phase6 reason code",
            pair.pair_id, pair.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "pair {} leg A reason code should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "pair {} leg B reason code should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "pair {} leg A taxonomy should stay on runtime taxonomy version",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "pair {} leg B taxonomy should stay on runtime taxonomy version",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "pair {} leg A taxonomy should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "pair {} leg B taxonomy should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "pair {} legs should share the same runtime taxonomy version",
            pair.pair_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_role_pair_lane_contract_is_canonical(
) {
    let lanes = project_live_postgres_parallel_role_pair_lanes();
    let lane_ids_csv = lanes
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        lane_ids_csv,
        LIVE_POSTGRES_MATRIX_PARALLEL_ROLE_PAIR_LANE_IDS_CSV
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
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_role_pair_lane_is_deterministic(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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

    for lane in project_live_postgres_parallel_role_pair_lanes() {
        let (leg_a_first, leg_b_first) =
            run_parallel_phase6_projections(lane.leg_a_args.clone(), lane.leg_b_args.clone());
        let (leg_a_second, leg_b_second) =
            run_parallel_phase6_projections(lane.leg_a_args, lane.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, lane.expected_reason_code,
            "parallel lane {} leg A ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, lane.expected_reason_code,
            "parallel lane {} leg B ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "parallel lane {} leg A reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "parallel lane {} leg B reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "parallel lane {} leg A taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "parallel lane {} leg B taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "parallel lane {} leg A taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "parallel lane {} leg B taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "parallel lane {} legs should share the same runtime taxonomy version",
            lane.pair_id
        );
    }
}

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
        .expect("log env lock should guard test mutation");
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
        .expect("log env lock should guard test mutation");
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

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_permutation_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_MATRIX_PERMUTATION_IDS_CSV,
        [
            "baseline",
            "reverse",
            "rotate_left_1",
            "interleaved_even_then_odd"
        ]
        .join(",")
    );
    let symmetric_lanes = project_live_postgres_parallel_role_pair_lanes();
    let symmetric_base_ids = symmetric_lanes
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    assert_eq!(
        symmetric_base_ids,
        vec![
            "processor_listener_parallel_applied",
            "processor_listener_parallel_deferred",
            "listener_approver_parallel_applied",
            "listener_approver_parallel_deferred"
        ]
    );
    let symmetric_reverse_ids =
        permute_role_pair_lanes(project_live_postgres_parallel_role_pair_lanes(), "reverse")
            .iter()
            .map(|lane| lane.pair_id)
            .collect::<Vec<_>>();
    assert_eq!(
        symmetric_reverse_ids,
        vec![
            "listener_approver_parallel_deferred",
            "listener_approver_parallel_applied",
            "processor_listener_parallel_deferred",
            "processor_listener_parallel_applied"
        ]
    );
    let symmetric_rotate_ids = permute_role_pair_lanes(
        project_live_postgres_parallel_role_pair_lanes(),
        "rotate_left_1",
    )
    .iter()
    .map(|lane| lane.pair_id)
    .collect::<Vec<_>>();
    assert_eq!(
        symmetric_rotate_ids,
        vec![
            "processor_listener_parallel_deferred",
            "listener_approver_parallel_applied",
            "listener_approver_parallel_deferred",
            "processor_listener_parallel_applied"
        ]
    );
    let symmetric_interleaved_ids = permute_role_pair_lanes(
        project_live_postgres_parallel_role_pair_lanes(),
        "interleaved_even_then_odd",
    )
    .iter()
    .map(|lane| lane.pair_id)
    .collect::<Vec<_>>();
    assert_eq!(
        symmetric_interleaved_ids,
        vec![
            "processor_listener_parallel_applied",
            "listener_approver_parallel_applied",
            "processor_listener_parallel_deferred",
            "listener_approver_parallel_deferred"
        ]
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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

    let permutation_ids = [
        "baseline",
        "reverse",
        "rotate_left_1",
        "interleaved_even_then_odd",
    ];

    let symmetric_baseline = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
        project_live_postgres_parallel_role_pair_lanes(),
        permutation_ids[0],
    ));
    for permutation in permutation_ids.iter().skip(1) {
        let permuted = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
            project_live_postgres_parallel_role_pair_lanes(),
            permutation,
        ));
        assert_eq!(
            symmetric_baseline, permuted,
            "symmetric parallel lane fingerprints should remain invariant to permutation {permutation}"
        );
    }

    let asymmetric_baseline = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
        project_live_postgres_asymmetric_parallel_lanes(),
        permutation_ids[0],
    ));
    for permutation in permutation_ids.iter().skip(1) {
        let permuted = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
            project_live_postgres_asymmetric_parallel_lanes(),
            permutation,
        ));
        assert_eq!(
            asymmetric_baseline, permuted,
            "asymmetric parallel lane fingerprints should remain invariant to permutation {permutation}"
        );
    }
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice() {
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
        .unwrap_or_else(|error| error.into_inner());
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
        .unwrap_or_else(|error| error.into_inner());
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
        .unwrap_or_else(|error| error.into_inner());

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


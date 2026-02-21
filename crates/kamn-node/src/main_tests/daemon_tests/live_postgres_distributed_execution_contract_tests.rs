#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_multi_host_execution_prerequisite_guard_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.multi-host-execution.reason-taxonomy.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_PREREQUISITE_ENV_KEYS_CSV,
        "KAMN_TEST_POSTGRES_URL|DATABASE_URL,KAMN_TEST_LIVE_POSTGRES_DISTRIBUTED_HOSTS"
    );
    assert_eq!(
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_PREREQUISITE_REASON_CODES_CSV,
        [
            LIVE_POSTGRES_MULTI_HOST_EXECUTION_READY_REASON_CODE,
            LIVE_POSTGRES_MULTI_HOST_EXECUTION_PREREQUISITES_MISSING_REASON_CODE,
            LIVE_POSTGRES_MULTI_HOST_EXECUTION_HOST_PAIR_INVALID_REASON_CODE,
        ]
        .join(",")
    );

    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard multi-host prerequisite marker checks");

    let _unset_primary = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", None);
    let _unset_fallback = EnvVarGuard::set("DATABASE_URL", None);
    let _unset_hosts = EnvVarGuard::set("KAMN_TEST_LIVE_POSTGRES_DISTRIBUTED_HOSTS", None);
    let missing = resolve_live_postgres_multi_host_prerequisite_decision();
    assert_eq!(
        missing.reason_code,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_PREREQUISITES_MISSING_REASON_CODE
    );
    assert_eq!(
        missing.reason_taxonomy_version,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_REASON_TAXONOMY_VERSION
    );
    assert!(missing.host_pair_csv.is_none());
    drop(_unset_hosts);
    drop(_unset_fallback);
    drop(_unset_primary);

    let _db_present = EnvVarGuard::set(
        "KAMN_TEST_POSTGRES_URL",
        Some("postgres://distributed-prereq:5432/kamn_test"),
    );
    let _invalid_hosts = EnvVarGuard::set("KAMN_TEST_LIVE_POSTGRES_DISTRIBUTED_HOSTS", Some("node_alpha"));
    let invalid = resolve_live_postgres_multi_host_prerequisite_decision();
    assert_eq!(
        invalid.reason_code,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_HOST_PAIR_INVALID_REASON_CODE
    );
    assert_eq!(
        invalid.reason_taxonomy_version,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_REASON_TAXONOMY_VERSION
    );
    assert!(invalid.host_pair_csv.is_none());
    drop(_invalid_hosts);
    drop(_db_present);

    let _db_present = EnvVarGuard::set(
        "KAMN_TEST_POSTGRES_URL",
        Some("postgres://distributed-prereq:5432/kamn_test"),
    );
    let _host_pair = EnvVarGuard::set(
        "KAMN_TEST_LIVE_POSTGRES_DISTRIBUTED_HOSTS",
        Some("node_alpha,node_beta"),
    );
    let ready = resolve_live_postgres_multi_host_prerequisite_decision();
    assert_eq!(
        ready.reason_code,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_READY_REASON_CODE
    );
    assert_eq!(
        ready.reason_taxonomy_version,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_REASON_TAXONOMY_VERSION
    );
    assert_eq!(ready.host_pair_csv.as_deref(), Some("node_alpha,node_beta"));
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_multi_host_execution_bundle_selector_rows_are_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX,
        "main_tests::daemon_tests::"
    );

    let rows = project_live_postgres_multi_host_execution_bundle_selector_rows();
    assert_eq!(
        rows.join(","),
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS_CSV
    );
    assert_eq!(rows.len(), 6);
    assert!(rows
        .iter()
        .all(|row| row.contains(LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX)));
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_multi_host_execution_bundle_runtime_row_count_matches_selector_rows(
) {
    let runtime_selector_rows = crate::live_postgres_multi_host_execution_bundle_selector_rows_for_test();
    let fixture_selector_rows = project_live_postgres_multi_host_execution_bundle_selector_rows();
    assert_eq!(
        runtime_selector_rows,
        fixture_selector_rows,
        "runtime selector rows should stay aligned with daemon fixture selectors"
    );

    let runtime_row_count = crate::live_postgres_multi_host_execution_bundle_row_count_for_test();
    assert_eq!(
        runtime_row_count,
        runtime_selector_rows.len(),
        "runtime row count marker should derive from runtime selector row length"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_multi_host_execution_bundle_is_stable(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let _host_pair_guard = EnvVarGuard::set(
        "KAMN_TEST_LIVE_POSTGRES_DISTRIBUTED_HOSTS",
        Some("node_alpha,node_beta"),
    );

    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        let decision = resolve_live_postgres_multi_host_prerequisite_decision();
        assert_eq!(
            decision.reason_code,
            LIVE_POSTGRES_MULTI_HOST_EXECUTION_PREREQUISITES_MISSING_REASON_CODE
        );
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
            .expect("live postgres migrations should apply for distributed-lane validation slice");
    });

    let first = run_live_postgres_multi_host_execution_bundle_projection()
        .expect("distributed multi-host execution projection should be ready after prerequisites");
    let second = run_live_postgres_multi_host_execution_bundle_projection()
        .expect("distributed multi-host execution projection should be repeatable");
    assert_eq!(
        first, second,
        "distributed multi-host execution projection should remain deterministic across repeated runs"
    );
    assert_eq!(
        first.reason_code,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_READY_REASON_CODE
    );
    assert_eq!(
        first.reason_taxonomy_version,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_REASON_TAXONOMY_VERSION
    );
    assert_eq!(
        first.fingerprint_hash_order_normalization_digest_hex,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_FNV1A64_HEX
    );
    assert!(
        first
            .distributed_topology_fingerprint
            .starts_with("distributed_label_parallel#"),
        "distributed topology projection should retain distributed-label topology id"
    );
}

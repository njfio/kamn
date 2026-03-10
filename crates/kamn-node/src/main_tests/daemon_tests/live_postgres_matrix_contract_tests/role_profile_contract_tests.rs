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

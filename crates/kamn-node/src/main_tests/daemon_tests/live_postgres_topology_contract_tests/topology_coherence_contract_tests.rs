#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_cardinality_coherence_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-cardinality-coherence.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_ROWS_CSV,
        "same_host_parallel->same_host->1,distributed_label_parallel->distributed_label->2"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_CONTRACT,
        "topology_id_to_host_mode_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_cardinality_row(&topology_fingerprint),
        "distributed_label_parallel->distributed_label->2"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_cardinality_coherence_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows = collect_parallel_lane_topology_id_host_mode_cardinality_rows(
        permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ),
    );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->2",
            "same_host_parallel->same_host->1"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_mode_cardinality_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-cardinality rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_cardinality_coherence_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-cardinality-coherence.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_ROWS_CSV,
        "same_host_parallel->node_alpha->node_alpha->1,distributed_label_parallel->node_alpha->node_beta->2"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_CONTRACT,
        "topology_id_to_host_pair_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_pair_cardinality_row(&topology_fingerprint),
        "distributed_label_parallel->node_alpha->node_beta->2"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_cardinality_coherence_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows = collect_parallel_lane_topology_id_host_pair_cardinality_rows(
        permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ),
    );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->node_alpha->node_beta->2",
            "same_host_parallel->node_alpha->node_alpha->1"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_pair_cardinality_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-pair-cardinality rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_coherence_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-coherence.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_ROWS_CSV,
        "same_host_parallel->same_host->node_alpha->node_alpha,distributed_label_parallel->distributed_label->node_alpha->node_beta"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_CONTRACT,
        "topology_id_to_host_mode_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_host_pair_row(&topology_fingerprint),
        "distributed_label_parallel->distributed_label->node_alpha->node_beta"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_coherence_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows = collect_parallel_lane_topology_id_host_mode_host_pair_rows(
        permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ),
    );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->node_alpha->node_beta",
            "same_host_parallel->same_host->node_alpha->node_alpha"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_mode_host_pair_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-host-pair rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-cardinality-coherence.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_ROWS_CSV,
        "same_host_parallel->same_host->node_alpha->node_alpha->1,distributed_label_parallel->distributed_label->node_alpha->node_beta->2"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_CONTRACT,
        "topology_id_to_host_mode_host_pair_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_host_pair_cardinality_row(
            &topology_fingerprint
        ),
        "distributed_label_parallel->distributed_label->node_alpha->node_beta->2"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows = collect_parallel_lane_topology_id_host_mode_host_pair_cardinality_rows(
        permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ),
    );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->node_alpha->node_beta->2",
            "same_host_parallel->same_host->node_alpha->node_alpha->1"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_mode_host_pair_cardinality_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-host-pair-cardinality rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-count-coherence.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_ROWS_CSV,
        "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->4,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->4"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_CONTRACT,
        "topology_id_to_host_mode_host_pair_lane_set_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_row(
            &topology_fingerprint
        ),
        "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->1"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows =
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation_ids[0],
            ),
        );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->4",
            "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->4"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows =
            collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_rows(
                permute_parallel_lane_topology_profiles(
                    project_live_postgres_parallel_lane_topology_profiles(),
                    permutation,
                ),
            );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-host-pair-lane-set-lane-count rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-id-bundle-coherence.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_ROWS_CSV,
        "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied+listener_approver_parallel_deferred+processor_listener_parallel_applied+processor_listener_parallel_deferred,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied+listener_approver_asymmetric_parallel_deferred+processor_listener_asymmetric_parallel_applied+processor_listener_asymmetric_parallel_deferred"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_CONTRACT,
        "topology_id_to_host_mode_host_pair_lane_set_lane_id_bundle_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_asymmetric_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_row(
            &topology_fingerprint
        ),
        "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->processor_listener_asymmetric_parallel_applied"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows =
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation_ids[0],
            ),
        );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied+listener_approver_asymmetric_parallel_deferred+processor_listener_asymmetric_parallel_applied+processor_listener_asymmetric_parallel_deferred",
            "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied+listener_approver_parallel_deferred+processor_listener_parallel_applied+processor_listener_parallel_deferred"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows =
            collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_rows(
                permute_parallel_lane_topology_profiles(
                    project_live_postgres_parallel_lane_topology_profiles(),
                    permutation,
                ),
            );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-host-pair-lane-set-lane-id-bundle rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_BUNDLE_COHERENCE_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-bundle-coherence.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_BUNDLE_COHERENCE_ROWS_CSV,
        "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+listener_approver_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+listener_approver_asymmetric_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_asymmetric_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_BUNDLE_COHERENCE_CONTRACT,
        "topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_bundle_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_asymmetric_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_row(
            &topology_fingerprint
        ),
        "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->processor_listener_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows =
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation_ids[0],
            ),
        );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+listener_approver_asymmetric_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_asymmetric_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1",
            "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+listener_approver_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_BUNDLE_COHERENCE_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-host-pair-lane-set-lane-fingerprint-bundle rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_COHERENCE_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-coherence.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_COHERENCE_ROWS_CSV,
        "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_COHERENCE_CONTRACT,
        "topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_asymmetric_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_row(
            &topology_fingerprint
        ),
        "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->218fd301449431fd"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows =
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation_ids[0],
            ),
        );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e",
            "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_COHERENCE_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows =
            collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows(
                permute_parallel_lane_topology_profiles(
                    project_live_postgres_parallel_lane_topology_profiles(),
                    permutation,
                ),
            );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-host-pair-lane-set-lane-fingerprint-hash rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-order-normalization.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_ROWS_CSV,
        "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_CONTRACT,
        "topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_must_remain_canonically_sorted_after_order_normalization"
    );

    let applied_lane_fingerprint = format_parallel_lane_fingerprint(
        "lane_a",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let deferred_lane_fingerprint = format_parallel_lane_fingerprint(
        "lane_b",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let sorted_topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![
            applied_lane_fingerprint.clone(),
            deferred_lane_fingerprint.clone(),
        ],
    );
    let unsorted_topology_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![deferred_lane_fingerprint, applied_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_row(
            &sorted_topology_fingerprint
        ),
        extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_row(
            &unsorted_topology_fingerprint
        ),
        "lane-fingerprint hash rows should normalize bundle ordering before hashing"
    );

    let baseline_rows = collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalized_rows(
        project_live_postgres_parallel_lane_topology_profiles(),
    );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e",
            "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_ROWS_CSV
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_rows = collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalized_rows(
        permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ),
    );
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e",
            "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalized_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-host-pair-lane-set-lane-fingerprint-hash order-normalization rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-order-normalization-digest.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_CSV,
        "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_FNV1A64_HEX,
        "25b9729eaeb44fe9"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_CONTRACT,
        "topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_digest_must_remain_stable_under_order_normalization_and_permutations"
    );

    let (baseline_rows, baseline_digest) =
        project_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest(
            project_live_postgres_parallel_lane_topology_profiles(),
        );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_CSV
    );
    assert_eq!(
        baseline_digest,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_FNV1A64_HEX
    );

    let mut reordered_rows = baseline_rows.clone();
    reordered_rows.reverse();
    let reordered_digest = deterministic_fnv1a64_hex(&reordered_rows.join(","));
    assert_ne!(
        baseline_digest, reordered_digest,
        "digest should change when row ordering drifts from canonical order-normalized projection"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let (baseline_rows, baseline_digest) =
        project_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation_ids[0],
            ),
        );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_CSV
    );
    assert_eq!(
        baseline_digest,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_FNV1A64_HEX
    );

    for permutation in permutation_ids.iter().skip(1) {
        let (permuted_rows, permuted_digest) =
            project_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest(
                permute_parallel_lane_topology_profiles(
                    project_live_postgres_parallel_lane_topology_profiles(),
                    permutation,
                ),
            );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-host-pair-lane-set-lane-fingerprint-hash order-normalization rows should remain stable under permutation {permutation}"
        );
        assert_eq!(
            baseline_digest, permuted_digest,
            "topology-id to host-mode-host-pair-lane-set-lane-fingerprint-hash order-normalization digest should remain stable under permutation {permutation}"
        );
    }
}


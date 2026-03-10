use super::*;
#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_REQUIRED_HOST_PAIR_IDS_CSV,
        ["node_alpha->node_alpha", "node_alpha->node_beta"].join(",")
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CONTRACT,
        "host_pair_ids_must_remain_stable_under_repeated_runs_and_topology_permutations"
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
        "same_host_parallel",
        "node_alpha",
        "node_alpha",
        vec![sample_lane_fingerprint],
    );
    assert_eq!(
        extract_parallel_lane_topology_host_pair_id(&topology_fingerprint),
        "node_alpha->node_alpha"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pairs_are_stable(
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

    let permutation_ids = ["baseline", "reverse", "rotate_left_1"];
    let baseline_host_pair_ids =
        collect_parallel_lane_topology_host_pair_ids(permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ));
    assert_eq!(
        baseline_host_pair_ids,
        vec!["node_alpha->node_alpha", "node_alpha->node_beta"]
    );
    assert_eq!(
        baseline_host_pair_ids.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_REQUIRED_HOST_PAIR_IDS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_host_pair_ids =
            collect_parallel_lane_topology_host_pair_ids(permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ));
        assert_eq!(
            baseline_host_pair_ids, permuted_host_pair_ids,
            "topology host-pair ids should remain stable under permutation {permutation}"
        );
    }
}

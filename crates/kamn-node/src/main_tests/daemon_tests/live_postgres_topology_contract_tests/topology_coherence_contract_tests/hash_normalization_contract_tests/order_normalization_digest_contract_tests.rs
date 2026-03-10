use super::super::*;
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


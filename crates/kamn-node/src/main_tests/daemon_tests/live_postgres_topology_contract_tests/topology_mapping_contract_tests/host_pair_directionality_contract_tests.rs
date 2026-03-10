use super::*;
#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-directionality.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_EXTRACTION_RULE,
        "host_a_to_host_b_arrow_notation_non_commutative"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_FORBIDDEN_REVERSE_PAIRS_CSV,
        "node_beta->node_alpha"
    );

    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "listener_approver_asymmetric_parallel_applied",
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
        &LivePostgresPhase6Projection {
            reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
            reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
        },
    );
    let distributed_fingerprint = format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    );
    let canonical = extract_parallel_lane_topology_host_pair_id(&distributed_fingerprint);
    let reverse = extract_parallel_lane_topology_host_pair_reverse_id(&distributed_fingerprint);
    assert_eq!(canonical, "node_alpha->node_beta");
    assert_eq!(reverse, "node_beta->node_alpha");
    assert_ne!(
        canonical, reverse,
        "host-pair extraction should remain non-commutative"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_is_stable(
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
    let baseline_topology_fingerprints =
        run_parallel_lane_topology_fingerprints(permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ));
    let mut baseline_canonical_ids = baseline_topology_fingerprints
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_host_pair_id(fingerprint))
        .collect::<Vec<_>>();
    baseline_canonical_ids.sort();
    assert_eq!(
        baseline_canonical_ids,
        vec!["node_alpha->node_alpha", "node_alpha->node_beta"]
    );

    let forbidden_reverse_pairs =
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_FORBIDDEN_REVERSE_PAIRS_CSV
            .split(',')
            .collect::<Vec<_>>();

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_topology_fingerprints =
            run_parallel_lane_topology_fingerprints(permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ));
        let mut permuted_canonical_ids = permuted_topology_fingerprints
            .iter()
            .map(|fingerprint| extract_parallel_lane_topology_host_pair_id(fingerprint))
            .collect::<Vec<_>>();
        permuted_canonical_ids.sort();
        assert_eq!(
            baseline_canonical_ids, permuted_canonical_ids,
            "canonical host-pair ids should remain stable under permutation {permutation}"
        );

        for topology_fingerprint in &permuted_topology_fingerprints {
            let reverse_id =
                extract_parallel_lane_topology_host_pair_reverse_id(topology_fingerprint);
            assert!(
                !forbidden_reverse_pairs.contains(&reverse_id.as_str()),
                "reverse host-pair id {} must remain forbidden under directionality contract",
                reverse_id
            );
        }
    }
}

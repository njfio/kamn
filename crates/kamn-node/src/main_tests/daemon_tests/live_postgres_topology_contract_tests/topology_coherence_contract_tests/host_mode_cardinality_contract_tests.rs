use super::*;
#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_cardinality_coherence_contract_is_canonical(
) {
    assert_host_mode_cardinality_contract_metadata();
    assert_host_mode_cardinality_sample_row();
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_cardinality_coherence_is_stable(
) {
    with_live_postgres_validation_database_url(|database_url| {
        apply_live_postgres_validation_migrations(database_url);
        let baseline_rows = host_mode_cardinality_rows("baseline");
        assert_host_mode_cardinality_baseline(&baseline_rows);
        assert_host_mode_cardinality_permutations(&baseline_rows);
    });
}

fn assert_host_mode_cardinality_contract_metadata() {
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
}

fn assert_host_mode_cardinality_sample_row() {
    let topology_fingerprint = sample_host_mode_cardinality_topology_fingerprint();
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_cardinality_row(&topology_fingerprint),
        "distributed_label_parallel->distributed_label->2"
    );
}

fn sample_host_mode_cardinality_topology_fingerprint() -> String {
    let sample_lane_fingerprint = format_parallel_lane_fingerprint(
        "processor_listener_parallel_applied",
        &sample_phase6_projection(),
        &sample_phase6_projection(),
    );
    format_parallel_lane_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        vec![sample_lane_fingerprint],
    )
}

fn sample_phase6_projection() -> LivePostgresPhase6Projection {
    LivePostgresPhase6Projection {
        reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE.to_owned(),
        reason_taxonomy_version: LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION.to_owned(),
    }
}

fn with_live_postgres_validation_database_url(run: impl FnOnce(String)) {
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
    run(database_url);
}

fn apply_live_postgres_validation_migrations(database_url: String) {
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
}

fn host_mode_cardinality_rows(permutation: &str) -> Vec<String> {
    collect_parallel_lane_topology_id_host_mode_cardinality_rows(
        permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation,
        ),
    )
}

fn assert_host_mode_cardinality_baseline(baseline_rows: &[String]) {
    assert_eq!(
        baseline_rows,
        [
            "distributed_label_parallel->distributed_label->2".to_owned(),
            "same_host_parallel->same_host->1".to_owned(),
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_ROWS_CSV
    );
}

fn assert_host_mode_cardinality_permutations(baseline_rows: &[String]) {
    for permutation in ["reverse", "rotate_left_1"] {
        let permuted_rows = host_mode_cardinality_rows(permutation);
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode-cardinality rows should remain stable under permutation {permutation}"
        );
    }
}

use super::super::*;
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


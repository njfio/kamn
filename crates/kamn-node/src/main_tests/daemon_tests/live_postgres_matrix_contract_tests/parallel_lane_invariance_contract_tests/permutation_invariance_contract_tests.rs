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

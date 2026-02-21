#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_fingerprint_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-fingerprint.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_ORDER_CSV,
        [
            "lane_id",
            "leg_a_reason",
            "leg_a_taxonomy",
            "leg_b_reason",
            "leg_b_taxonomy"
        ]
        .join(",")
    );
    assert_eq!(LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER, '|');
    assert_eq!(LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_COUNT, 5);

    let fingerprint = format_parallel_lane_fingerprint(
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
    assert_eq!(
        fingerprint,
        "processor_listener_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    );
    assert_parallel_lane_fingerprint_schema(&fingerprint, &["processor_listener_parallel_applied"]);
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_fingerprint_schema_is_stable(
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

    let symmetric_lane_ids = project_live_postgres_parallel_role_pair_lanes()
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    let symmetric_first =
        run_parallel_lane_set_fingerprints(project_live_postgres_parallel_role_pair_lanes());
    let symmetric_second =
        run_parallel_lane_set_fingerprints(project_live_postgres_parallel_role_pair_lanes());
    assert_eq!(
        symmetric_first, symmetric_second,
        "symmetric parallel lane fingerprints should remain stable across repeated runs"
    );
    assert_eq!(symmetric_first.len(), symmetric_lane_ids.len());
    for fingerprint in &symmetric_first {
        assert_parallel_lane_fingerprint_schema(fingerprint, &symmetric_lane_ids);
    }

    let asymmetric_lane_ids = project_live_postgres_asymmetric_parallel_lanes()
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    let asymmetric_first =
        run_parallel_lane_set_fingerprints(project_live_postgres_asymmetric_parallel_lanes());
    let asymmetric_second =
        run_parallel_lane_set_fingerprints(project_live_postgres_asymmetric_parallel_lanes());
    assert_eq!(
        asymmetric_first, asymmetric_second,
        "asymmetric parallel lane fingerprints should remain stable across repeated runs"
    );
    assert_eq!(asymmetric_first.len(), asymmetric_lane_ids.len());
    for fingerprint in &asymmetric_first {
        assert_parallel_lane_fingerprint_schema(fingerprint, &asymmetric_lane_ids);
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_scope_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_IDS_CSV,
        ["same_host_parallel", "distributed_label_parallel"].join(",")
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_CONTRACT,
        "topology_labels_must_preserve_sorted_lane_reason_taxonomy_fingerprints_under_repeated_runs"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_ORDER_CSV,
        ["topology_id", "host_a", "host_b", "lane_fingerprint_bundle"].join(",")
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER,
        '#'
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_BUNDLE_DELIMITER,
        ';'
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        4
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
        vec![sample_lane_fingerprint.clone()],
    );
    assert_eq!(
        topology_fingerprint,
        "same_host_parallel#node_alpha#node_alpha#processor_listener_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    );
    let fields = parse_parallel_lane_topology_fingerprint_fields(&topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT
    );
    assert_eq!(fields[0], "same_host_parallel");
    assert_eq!(fields[1], "node_alpha");
    assert_eq!(fields[2], "node_alpha");
    let lane_bundle = parse_parallel_lane_topology_bundle_fields(fields[3]);
    assert_eq!(lane_bundle, vec![sample_lane_fingerprint.as_str()]);
    assert_parallel_lane_fingerprint_schema(
        lane_bundle[0],
        &["processor_listener_parallel_applied"],
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable(
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

    let first = run_parallel_lane_topology_fingerprints(
        project_live_postgres_parallel_lane_topology_profiles(),
    );
    let second = run_parallel_lane_topology_fingerprints(
        project_live_postgres_parallel_lane_topology_profiles(),
    );
    assert_eq!(
        first, second,
        "topology-labeled parallel lane fingerprints should remain stable across repeated runs"
    );
    assert_eq!(first.len(), 2);

    for fingerprint in &first {
        let fields = parse_parallel_lane_topology_fingerprint_fields(fingerprint);
        assert_eq!(
            fields.len(),
            LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
            "topology fingerprint should keep canonical field count"
        );
        assert!(
            ["same_host_parallel", "distributed_label_parallel"].contains(&fields[0]),
            "topology id should remain canonical"
        );
        assert!(
            !fields[1].is_empty(),
            "host A label should remain non-empty"
        );
        assert!(
            !fields[2].is_empty(),
            "host B label should remain non-empty"
        );

        let lane_bundle = parse_parallel_lane_topology_bundle_fields(fields[3]);
        assert!(
            !lane_bundle.is_empty(),
            "topology fingerprint should include at least one lane fingerprint"
        );
        match fields[0] {
            "same_host_parallel" => {
                assert_eq!(fields[1], "node_alpha");
                assert_eq!(fields[2], "node_alpha");
                assert_eq!(
                    lane_bundle.len(),
                    project_live_postgres_parallel_role_pair_lanes().len()
                );
                let lane_ids = project_live_postgres_parallel_role_pair_lanes()
                    .iter()
                    .map(|lane| lane.pair_id)
                    .collect::<Vec<_>>();
                for lane_fingerprint in lane_bundle {
                    assert_parallel_lane_fingerprint_schema(lane_fingerprint, &lane_ids);
                }
            }
            "distributed_label_parallel" => {
                assert_eq!(fields[1], "node_alpha");
                assert_eq!(fields[2], "node_beta");
                assert_eq!(
                    lane_bundle.len(),
                    project_live_postgres_asymmetric_parallel_lanes().len()
                );
                let lane_ids = project_live_postgres_asymmetric_parallel_lanes()
                    .iter()
                    .map(|lane| lane.pair_id)
                    .collect::<Vec<_>>();
                for lane_fingerprint in lane_bundle {
                    assert_parallel_lane_fingerprint_schema(lane_fingerprint, &lane_ids);
                }
            }
            _ => panic!("unexpected topology id {}", fields[0]),
        }
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_permutation_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_PERMUTATION_IDS_CSV,
        ["baseline", "reverse", "rotate_left_1"].join(",")
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_PERMUTATION_CONTRACT,
        "deterministic_topology_profile_permutations_must_preserve_sorted_topology_fingerprint_bundles"
    );

    let baseline_ids = permute_parallel_lane_topology_profiles(
        project_live_postgres_parallel_lane_topology_profiles(),
        "baseline",
    )
    .iter()
    .map(|profile| profile.topology_id)
    .collect::<Vec<_>>();
    assert_eq!(
        baseline_ids,
        vec!["same_host_parallel", "distributed_label_parallel"]
    );

    let reverse_ids = permute_parallel_lane_topology_profiles(
        project_live_postgres_parallel_lane_topology_profiles(),
        "reverse",
    )
    .iter()
    .map(|profile| profile.topology_id)
    .collect::<Vec<_>>();
    assert_eq!(
        reverse_ids,
        vec!["distributed_label_parallel", "same_host_parallel"]
    );

    let rotate_ids = permute_parallel_lane_topology_profiles(
        project_live_postgres_parallel_lane_topology_profiles(),
        "rotate_left_1",
    )
    .iter()
    .map(|profile| profile.topology_id)
    .collect::<Vec<_>>();
    assert_eq!(
        rotate_ids,
        vec!["distributed_label_parallel", "same_host_parallel"]
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_permutations_are_invariant(
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
    let baseline =
        run_parallel_lane_topology_fingerprints(permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ));
    for permutation in permutation_ids.iter().skip(1) {
        let permuted =
            run_parallel_lane_topology_fingerprints(permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ));
        assert_eq!(
            baseline, permuted,
            "topology fingerprints should remain invariant to topology permutation {permutation}"
        );
    }
}


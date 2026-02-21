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

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_mapping_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-mapping.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_ROWS_CSV,
        "same_host_parallel->node_alpha->node_alpha,distributed_label_parallel->node_alpha->node_beta"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_CONTRACT,
        "topology_id_to_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations"
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
        extract_parallel_lane_topology_id_host_pair_row(&topology_fingerprint),
        "same_host_parallel->node_alpha->node_alpha"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_mapping_is_stable(
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
        collect_parallel_lane_topology_id_host_pair_rows(permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ));
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->node_alpha->node_beta",
            "same_host_parallel->node_alpha->node_alpha"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_pair_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-pair rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_lane_set_mapping_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-set-mapping.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_ROWS_CSV,
        "same_host_parallel->symmetric_parallel,distributed_label_parallel->asymmetric_parallel"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_CONTRACT,
        "topology_id_to_lane_set_rows_must_remain_stable_under_repeated_runs_and_permutations"
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
        extract_parallel_lane_topology_id_lane_set_row(&topology_fingerprint),
        "same_host_parallel->symmetric_parallel"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_lane_set_mapping_is_stable(
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
        collect_parallel_lane_topology_id_lane_set_rows(permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ));
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->asymmetric_parallel",
            "same_host_parallel->symmetric_parallel"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_lane_set_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to lane-set rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_lane_count_mapping_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-count-mapping.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_ROWS_CSV,
        "same_host_parallel->4,distributed_label_parallel->4"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_CONTRACT,
        "topology_id_to_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations"
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
        extract_parallel_lane_topology_id_lane_count_row(&topology_fingerprint),
        "same_host_parallel->1"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_lane_count_mapping_is_stable(
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
        collect_parallel_lane_topology_id_lane_count_rows(permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ));
    assert_eq!(
        baseline_rows,
        vec!["distributed_label_parallel->4", "same_host_parallel->4"]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_lane_count_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to lane-count rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_mapping_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-mapping.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_ROWS_CSV,
        "same_host_parallel->same_host,distributed_label_parallel->distributed_label"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_CONTRACT,
        "topology_id_to_host_mode_rows_must_remain_stable_under_repeated_runs_and_permutations"
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
        extract_parallel_lane_topology_id_host_mode_row(&topology_fingerprint),
        "same_host_parallel->same_host"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_mapping_is_stable(
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
        collect_parallel_lane_topology_id_host_mode_rows(permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ));
    assert_eq!(
        baseline_rows,
        vec![
            "distributed_label_parallel->distributed_label",
            "same_host_parallel->same_host"
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_mode_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-mode rows should remain stable under permutation {permutation}"
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_cardinality_mapping_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-cardinality-mapping.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_ROWS_CSV,
        "same_host_parallel->1,distributed_label_parallel->2"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_CONTRACT,
        "topology_id_to_unique_host_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations"
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
        extract_parallel_lane_topology_id_host_cardinality_row(&topology_fingerprint),
        "distributed_label_parallel->2"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_cardinality_mapping_is_stable(
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
    let baseline_rows = collect_parallel_lane_topology_id_host_cardinality_rows(
        permute_parallel_lane_topology_profiles(
            project_live_postgres_parallel_lane_topology_profiles(),
            permutation_ids[0],
        ),
    );
    assert_eq!(
        baseline_rows,
        vec!["distributed_label_parallel->2", "same_host_parallel->1"]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_ROWS_CSV
    );

    for permutation in permutation_ids.iter().skip(1) {
        let permuted_rows = collect_parallel_lane_topology_id_host_cardinality_rows(
            permute_parallel_lane_topology_profiles(
                project_live_postgres_parallel_lane_topology_profiles(),
                permutation,
            ),
        );
        assert_eq!(
            baseline_rows, permuted_rows,
            "topology-id to host-cardinality rows should remain stable under permutation {permutation}"
        );
    }
}

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

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs(
) {
    let Some((applied_first, applied_second, deferred_first, deferred_second)) =
        run_live_postgres_matrix_repeated_run_projections()
    else {
        return;
    };
    assert_eq!(
        applied_first.reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        applied_first.reason_code, applied_second.reason_code,
        "applied scenario reason should remain stable across repeated runs"
    );

    assert_eq!(
        deferred_first.reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(
        deferred_first.reason_code, deferred_second.reason_code,
        "deferred scenario reason should remain stable across repeated runs"
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_taxonomy_versions_are_stable_across_repeated_runs(
) {
    let Some((applied_first, applied_second, deferred_first, deferred_second)) =
        run_live_postgres_matrix_repeated_run_projections()
    else {
        return;
    };
    assert_eq!(
        applied_first.reason_taxonomy_version,
        LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION
    );
    assert_eq!(
        applied_first.reason_taxonomy_version, applied_second.reason_taxonomy_version,
        "applied scenario taxonomy version should remain stable across repeated runs"
    );
    assert_eq!(
        deferred_first.reason_taxonomy_version,
        LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION
    );
    assert_eq!(
        deferred_first.reason_taxonomy_version, deferred_second.reason_taxonomy_version,
        "deferred scenario taxonomy version should remain stable across repeated runs"
    );
    assert_eq!(
        applied_first.reason_taxonomy_version, deferred_first.reason_taxonomy_version,
        "applied/deferred scenarios should remain bridged to the same runtime taxonomy version"
    );
}

#[test]
fn regression_daemon_phase6_runtime_projection_fail_closed_reason_is_stable_on_clock_regression() {
    // Regression: #5299
    let (reason_code, fail_closed_cycles) =
        crate::execute_daemon_phase6_runtime_projection_for_test(5, 25, false, Some(1_700_000_119))
            .expect("phase6 runtime projection helper should return deterministic snapshot");
    assert_eq!(
        reason_code, "m10_phase6_scheduler_signal_invalid",
        "clock-regression path must remain fail-closed with stable reason marker"
    );
    assert_eq!(fail_closed_cycles, 1);
}

#[test]
fn regression_daemon_convergence_projection_fail_closed_reason_is_stable() {
    // Regression: #5301
    let first =
        crate::execute_daemon_convergence_projection_for_test(true, true, true, false, true);
    let second =
        crate::execute_daemon_convergence_projection_for_test(true, true, true, false, true);
    assert_eq!(
        first, second,
        "convergence projection must remain deterministic"
    );
    assert_eq!(first.0, "no_go");
    assert_eq!(first.1, "convergence_performance_budget_exceeded");
}

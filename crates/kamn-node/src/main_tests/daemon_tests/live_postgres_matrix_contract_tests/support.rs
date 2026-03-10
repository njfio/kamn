struct LivePostgresValidationContext {
    _lock: std::sync::MutexGuard<'static, ()>,
    _level_guard: EnvVarGuard,
    _format_guard: EnvVarGuard,
}

fn hold_log_env_lock() -> std::sync::MutexGuard<'static, ()> {
    log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn assert_gate_resolution(
    test_postgres_url: Option<&str>,
    database_url: Option<&str>,
    expected_reason_code: &str,
    expected_database_url: Option<&str>,
) {
    let _test_postgres_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", test_postgres_url);
    let _database_guard = EnvVarGuard::set("DATABASE_URL", database_url);
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    assert_eq!(gate_reason_code, expected_reason_code);
    assert_eq!(maybe_database_url.as_deref(), expected_database_url);
}

fn live_postgres_validation_context() -> Option<LivePostgresValidationContext> {
    let lock = hold_log_env_lock();
    let level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return None;
    };
    assert_eq!(gate_reason_code, LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE);
    prepare_live_postgres_database(database_url);
    Some(LivePostgresValidationContext {
        _lock: lock,
        _level_guard: level_guard,
        _format_guard: format_guard,
    })
}

fn prepare_live_postgres_database(database_url: String) {
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

fn daemon_json_report(extra_args: &[&str]) -> String {
    let mut args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
    ];
    args.extend(extra_args.iter().map(|arg| (*arg).to_owned()));
    args.push("--output".to_owned());
    args.push("json".to_owned());
    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    render_bootstrap_report(&report, OutputMode::json())
}

fn assert_rendered_phase6_reason(rendered: &str, expected_reason_code: &str) {
    assert!(rendered.contains(
        "\"daemon_phase6_runtime_reason_taxonomy_version\":\"kamn.runtime.daemon.phase6.reason-taxonomy.v1\""
    ));
    assert!(
        rendered.contains(&format!(
            "\"daemon_phase6_runtime_reason_code\":\"{expected_reason_code}\""
        )),
        "rendered daemon report should contain phase6 reason {expected_reason_code}"
    );
}

fn assert_rendered_deferred_cycles(rendered: &str, expected_cycles: usize) {
    assert!(rendered.contains(&format!(
        "\"daemon_phase6_runtime_deferred_cycles\":{expected_cycles}"
    )));
}

fn assert_profile_projection_stable(
    label: &str,
    profile_id: &str,
    args: Vec<String>,
    expected_reason_code: &str,
) {
    let first = run_daemon_for_phase6_projection(args.clone());
    let second = run_daemon_for_phase6_projection(args);
    assert_eq!(first.reason_code, expected_reason_code, "{label} {profile_id} should project expected phase6 reason code");
    assert_eq!(first.reason_code, second.reason_code, "{label} {profile_id} reason code should remain stable across repeated runs");
    assert_eq!(first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION, "{label} {profile_id} should remain bridged to runtime taxonomy version");
    assert_eq!(first.reason_taxonomy_version, second.reason_taxonomy_version, "{label} {profile_id} taxonomy version should remain stable across repeated runs");
}

fn assert_role_pair_leg_projection(
    pair_id: &str,
    leg_label: &str,
    profile_id: &str,
    expected_reason_code: &str,
    first: &LivePostgresPhase6Projection,
    second: &LivePostgresPhase6Projection,
) {
    assert_eq!(first.reason_code, expected_reason_code, "pair {pair_id} {leg_label} ({profile_id}) should project expected phase6 reason code");
    assert_eq!(first.reason_code, second.reason_code, "pair {pair_id} {leg_label} reason code should remain stable across repeated runs");
    assert_eq!(first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION, "pair {pair_id} {leg_label} taxonomy should stay on runtime taxonomy version");
    assert_eq!(first.reason_taxonomy_version, second.reason_taxonomy_version, "pair {pair_id} {leg_label} taxonomy should remain stable across repeated runs");
}

fn assert_serial_role_pair_projection(pair: &LivePostgresRolePairProfile) {
    let leg_a_first = run_daemon_for_phase6_projection(pair.leg_a_args.clone());
    let leg_a_second = run_daemon_for_phase6_projection(pair.leg_a_args.clone());
    let leg_b_first = run_daemon_for_phase6_projection(pair.leg_b_args.clone());
    let leg_b_second = run_daemon_for_phase6_projection(pair.leg_b_args.clone());
    assert_role_pair_leg_projection(pair.pair_id, "leg A", pair.leg_a_profile_id, pair.expected_reason_code, &leg_a_first, &leg_a_second);
    assert_role_pair_leg_projection(pair.pair_id, "leg B", pair.leg_b_profile_id, pair.expected_reason_code, &leg_b_first, &leg_b_second);
    assert_eq!(leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version, "pair {} legs should share the same runtime taxonomy version", pair.pair_id);
}

fn assert_parallel_role_pair_projection(lane: &LivePostgresRolePairProfile, label: &str) {
    let (leg_a_first, leg_b_first) =
        run_parallel_phase6_projections(lane.leg_a_args.clone(), lane.leg_b_args.clone());
    let (leg_a_second, leg_b_second) =
        run_parallel_phase6_projections(lane.leg_a_args.clone(), lane.leg_b_args.clone());
    assert_role_pair_leg_projection(lane.pair_id, "leg A", lane.leg_a_profile_id, lane.expected_reason_code, &leg_a_first, &leg_a_second);
    assert_role_pair_leg_projection(lane.pair_id, "leg B", lane.leg_b_profile_id, lane.expected_reason_code, &leg_b_first, &leg_b_second);
    assert_eq!(leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version, "{label} {} legs should share the same runtime taxonomy version", lane.pair_id);
}

fn lane_ids(lanes: &[LivePostgresRolePairProfile]) -> Vec<&'static str> {
    lanes.iter().map(|lane| lane.pair_id).collect::<Vec<_>>()
}

fn sorted_lane_ids(load_lanes: fn() -> Vec<LivePostgresRolePairProfile>) -> Vec<&'static str> {
    let mut ids = lane_ids(&load_lanes());
    ids.sort();
    ids
}

fn assert_lane_order_invariant(
    label: &str,
    load_lanes: fn() -> Vec<LivePostgresRolePairProfile>,
) {
    let baseline = run_parallel_lane_set_fingerprints(load_lanes());
    let mut reversed = load_lanes();
    reversed.reverse();
    let permuted = run_parallel_lane_set_fingerprints(reversed);
    assert_eq!(baseline, permuted, "{label} parallel lane fingerprints should remain invariant to lane order");
}

fn permutation_ids() -> [&'static str; 4] {
    ["baseline", "reverse", "rotate_left_1", "interleaved_even_then_odd"]
}

fn permuted_lane_ids(
    load_lanes: fn() -> Vec<LivePostgresRolePairProfile>,
    permutation: &str,
) -> Vec<&'static str> {
    lane_ids(&permute_role_pair_lanes(load_lanes(), permutation))
}

fn assert_lane_permutations_invariant(
    label: &str,
    load_lanes: fn() -> Vec<LivePostgresRolePairProfile>,
) {
    let permutations = permutation_ids();
    let baseline = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
        load_lanes(),
        permutations[0],
    ));
    for permutation in permutations.iter().skip(1) {
        let permuted = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
            load_lanes(),
            permutation,
        ));
        assert_eq!(baseline, permuted, "{label} parallel lane fingerprints should remain invariant to permutation {permutation}");
    }
}

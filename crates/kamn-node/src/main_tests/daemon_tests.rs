use super::*;
use crate::daemon_test_env_lock;
#[cfg(unix)]
use crate::{configure_os_signal_test_triggers, OsSignalTestKind, OsSignalTestTrigger};

const LIVE_POSTGRES_ENV_UNSET_REASON_CODE: &str = "live_postgres_env_unset";
const LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE: &str = "live_postgres_adapter_connected";
const LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.daemon.phase6.reason-taxonomy.v1";
const LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1";
const LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE: &str = "m10_phase6_scheduler_cycle_applied";
const LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_deferred";
const LIVE_POSTGRES_RUNTIME_TO_MATRIX_BRIDGE_REASON_CODES_CSV: &str =
    "m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred";
const LIVE_POSTGRES_MATRIX_LOAD_PROFILE_IDS_CSV: &str = "applied_t3_i10,applied_t5_i25,applied_t9_i40,deferred_t5_i25_s3_d2_to4,deferred_t7_i25_s3_d2_to4,deferred_t9_i40_s3_d2_to4";
const LIVE_POSTGRES_MATRIX_ROLE_PROFILE_IDS_CSV: &str = "processor_applied,processor_deferred,listener_applied,listener_deferred,approver_applied,approver_deferred";
const LIVE_POSTGRES_MATRIX_ROLE_PAIR_IDS_CSV: &str = "processor_to_listener_applied,processor_to_listener_deferred,listener_to_approver_applied,listener_to_approver_deferred,approver_to_processor_applied,approver_to_processor_deferred";
const LIVE_POSTGRES_MATRIX_PARALLEL_ROLE_PAIR_LANE_IDS_CSV: &str = "processor_listener_parallel_applied,processor_listener_parallel_deferred,listener_approver_parallel_applied,listener_approver_parallel_deferred";
const LIVE_POSTGRES_MATRIX_ASYMMETRIC_PARALLEL_LANE_IDS_CSV: &str = "processor_listener_asymmetric_parallel_applied,processor_listener_asymmetric_parallel_deferred,listener_approver_asymmetric_parallel_applied,listener_approver_asymmetric_parallel_deferred";
const LIVE_POSTGRES_MATRIX_PERMUTATION_IDS_CSV: &str =
    "baseline,reverse,rotate_left_1,interleaved_even_then_odd";
const LIVE_POSTGRES_MATRIX_ORDER_INVARIANCE_LANE_SETS_CSV: &str =
    "symmetric_parallel,asymmetric_parallel";
const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-fingerprint.v1";
const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_ORDER_CSV: &str =
    "lane_id,leg_a_reason,leg_a_taxonomy,leg_b_reason,leg_b_taxonomy";
const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER: char = '|';
const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_COUNT: usize = 5;
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology.v1";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_IDS_CSV: &str =
    "same_host_parallel,distributed_label_parallel";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_CONTRACT: &str =
    "topology_labels_must_preserve_sorted_lane_reason_taxonomy_fingerprints_under_repeated_runs";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_PERMUTATION_IDS_CSV: &str =
    "baseline,reverse,rotate_left_1";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_PERMUTATION_CONTRACT: &str =
    "deterministic_topology_profile_permutations_must_preserve_sorted_topology_fingerprint_bundles";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair.v1";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_REQUIRED_HOST_PAIR_IDS_CSV: &str =
    "node_alpha->node_alpha,node_alpha->node_beta";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CONTRACT: &str =
    "host_pair_ids_must_remain_stable_under_repeated_runs_and_topology_permutations";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-directionality.v1";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_EXTRACTION_RULE: &str =
    "host_a_to_host_b_arrow_notation_non_commutative";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_FORBIDDEN_REVERSE_PAIRS_CSV:
    &str = "node_beta->node_alpha";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_ORDER_CSV: &str =
    "topology_id,host_a,host_b,lane_fingerprint_bundle";
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER: char = '#';
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_BUNDLE_DELIMITER: char = ';';
const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT: usize = 4;
const LIVE_POSTGRES_MATRIX_REASON_CODES_CSV: &str =
    "live_postgres_env_unset,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred";
const LIVE_POSTGRES_MATRIX_SCENARIOS_CSV: &str = "env_unset,env_set_no_shutdown,env_set_shutdown";

fn parse_args_with_clean_daemon_env(args: Vec<String>) -> Result<crate::NodeCli, ConfigError> {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .expect("daemon env lock should guard process-level overrides");
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", None);
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", None);
    parse_args(args)
}

fn live_postgres_url() -> Option<String> {
    let preferred = std::env::var("KAMN_TEST_POSTGRES_URL")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let fallback = std::env::var("DATABASE_URL")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    preferred.or(fallback)
}

fn resolve_live_postgres_gate_decision() -> (&'static str, Option<String>) {
    match live_postgres_url() {
        Some(database_url) => (
            LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            Some(database_url),
        ),
        None => (LIVE_POSTGRES_ENV_UNSET_REASON_CODE, None),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LivePostgresMatrixRow {
    scenario_id: &'static str,
    gate_reason_code: &'static str,
    daemon_phase6_reason_code: Option<&'static str>,
}

#[derive(Debug, PartialEq, Eq)]
struct LivePostgresPhase6Projection {
    reason_code: String,
    reason_taxonomy_version: String,
}

#[derive(Debug, PartialEq, Eq)]
struct LivePostgresLoadProfile {
    profile_id: &'static str,
    args: Vec<String>,
    expected_reason_code: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
struct LivePostgresRolePairProfile {
    pair_id: &'static str,
    leg_a_profile_id: &'static str,
    leg_a_args: Vec<String>,
    leg_b_profile_id: &'static str,
    leg_b_args: Vec<String>,
    expected_reason_code: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
struct LivePostgresParallelLaneTopologyProfile {
    topology_id: &'static str,
    host_a: &'static str,
    host_b: &'static str,
    lanes: Vec<LivePostgresRolePairProfile>,
}

fn project_live_postgres_matrix_rows() -> Vec<LivePostgresMatrixRow> {
    vec![
        LivePostgresMatrixRow {
            scenario_id: "env_unset",
            gate_reason_code: LIVE_POSTGRES_ENV_UNSET_REASON_CODE,
            daemon_phase6_reason_code: None,
        },
        LivePostgresMatrixRow {
            scenario_id: "env_set_no_shutdown",
            gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE),
        },
        LivePostgresMatrixRow {
            scenario_id: "env_set_shutdown",
            gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE),
        },
    ]
}

fn run_daemon_for_phase6_projection(mut args: Vec<String>) -> LivePostgresPhase6Projection {
    args.push("--output".to_owned());
    args.push("json".to_owned());
    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    LivePostgresPhase6Projection {
        reason_code: extract_json_string_field(
            rendered.as_str(),
            "daemon_phase6_runtime_reason_code",
        )
        .expect("daemon report should expose phase6 reason code"),
        reason_taxonomy_version: extract_json_string_field(
            rendered.as_str(),
            "daemon_phase6_runtime_reason_taxonomy_version",
        )
        .expect("daemon report should expose phase6 reason taxonomy version"),
    }
}

fn run_parallel_phase6_projections(
    leg_a_args: Vec<String>,
    leg_b_args: Vec<String>,
) -> (LivePostgresPhase6Projection, LivePostgresPhase6Projection) {
    let leg_a_handle = std::thread::spawn(move || run_daemon_for_phase6_projection(leg_a_args));
    let leg_b_handle = std::thread::spawn(move || run_daemon_for_phase6_projection(leg_b_args));
    let leg_a_projection = leg_a_handle
        .join()
        .expect("parallel role-pair lane leg A should complete");
    let leg_b_projection = leg_b_handle
        .join()
        .expect("parallel role-pair lane leg B should complete");
    (leg_a_projection, leg_b_projection)
}

fn format_parallel_lane_fingerprint(
    lane_id: &str,
    leg_a_projection: &LivePostgresPhase6Projection,
    leg_b_projection: &LivePostgresPhase6Projection,
) -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}",
        lane_id,
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER,
        leg_a_projection.reason_code.as_str(),
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER,
        leg_a_projection.reason_taxonomy_version.as_str(),
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER,
        leg_b_projection.reason_code.as_str(),
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER,
        leg_b_projection.reason_taxonomy_version.as_str()
    )
}

fn parse_parallel_lane_fingerprint_fields(fingerprint: &str) -> Vec<&str> {
    fingerprint
        .split(LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER)
        .collect::<Vec<_>>()
}

fn assert_parallel_lane_fingerprint_schema(fingerprint: &str, expected_lane_ids: &[&str]) {
    let fields = parse_parallel_lane_fingerprint_fields(fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_COUNT,
        "fingerprint should contain the canonical number of schema fields"
    );
    assert!(
        expected_lane_ids
            .iter()
            .any(|lane_id| *lane_id == fields[0]),
        "fingerprint lane id {} should be one of {:?}",
        fields[0],
        expected_lane_ids
    );
    assert!(
        [
            LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
            LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
        ]
        .contains(&fields[1]),
        "fingerprint leg A reason should remain in the canonical reason taxonomy set"
    );
    assert_eq!(
        fields[1], fields[3],
        "parallel lane fingerprint should keep leg A and leg B reason codes aligned"
    );
    assert_eq!(
        fields[2], LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
        "fingerprint leg A taxonomy should remain canonical"
    );
    assert_eq!(
        fields[4], LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
        "fingerprint leg B taxonomy should remain canonical"
    );
}

fn format_parallel_lane_topology_fingerprint(
    topology_id: &str,
    host_a: &str,
    host_b: &str,
    lane_fingerprints: Vec<String>,
) -> String {
    format!(
        "{}{}{}{}{}{}{}",
        topology_id,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER,
        host_a,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER,
        host_b,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER,
        lane_fingerprints
            .join(&LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_BUNDLE_DELIMITER.to_string())
    )
}

fn parse_parallel_lane_topology_fingerprint_fields(fingerprint: &str) -> Vec<&str> {
    fingerprint
        .split(LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER)
        .collect::<Vec<_>>()
}

fn parse_parallel_lane_topology_bundle_fields(bundle: &str) -> Vec<&str> {
    if bundle.is_empty() {
        Vec::new()
    } else {
        bundle
            .split(LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_BUNDLE_DELIMITER)
            .collect::<Vec<_>>()
    }
}

fn extract_parallel_lane_topology_host_pair_id(topology_fingerprint: &str) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for host-pair extraction"
    );
    format!("{}->{}", fields[1], fields[2])
}

fn extract_parallel_lane_topology_host_pair_reverse_id(topology_fingerprint: &str) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for reverse host-pair extraction"
    );
    format!("{}->{}", fields[2], fields[1])
}

fn permute_parallel_lane_topology_profiles(
    mut topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
    permutation_id: &str,
) -> Vec<LivePostgresParallelLaneTopologyProfile> {
    match permutation_id {
        "baseline" => topology_profiles,
        "reverse" => {
            topology_profiles.reverse();
            topology_profiles
        }
        "rotate_left_1" => {
            if !topology_profiles.is_empty() {
                topology_profiles.rotate_left(1);
            }
            topology_profiles
        }
        _ => panic!("unknown topology permutation: {permutation_id}"),
    }
}

fn run_parallel_lane_set_fingerprints(lanes: Vec<LivePostgresRolePairProfile>) -> Vec<String> {
    let mut fingerprints = Vec::with_capacity(lanes.len());
    for lane in lanes {
        let (leg_a_projection, leg_b_projection) =
            run_parallel_phase6_projections(lane.leg_a_args, lane.leg_b_args);
        assert_eq!(
            leg_a_projection.reason_code, lane.expected_reason_code,
            "lane {} leg A ({}) should project expected reason code",
            lane.pair_id, lane.leg_a_profile_id
        );
        assert_eq!(
            leg_b_projection.reason_code, lane.expected_reason_code,
            "lane {} leg B ({}) should project expected reason code",
            lane.pair_id, lane.leg_b_profile_id
        );
        assert_eq!(
            leg_a_projection.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "lane {} leg A taxonomy should remain stable",
            lane.pair_id
        );
        assert_eq!(
            leg_b_projection.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "lane {} leg B taxonomy should remain stable",
            lane.pair_id
        );
        fingerprints.push(format_parallel_lane_fingerprint(
            lane.pair_id,
            &leg_a_projection,
            &leg_b_projection,
        ));
    }
    fingerprints.sort();
    fingerprints
}

fn run_parallel_lane_topology_fingerprints(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut topology_fingerprints = Vec::with_capacity(topology_profiles.len());
    for topology_profile in topology_profiles {
        let expected_lane_ids = topology_profile
            .lanes
            .iter()
            .map(|lane| lane.pair_id)
            .collect::<Vec<_>>();
        let lane_fingerprints = run_parallel_lane_set_fingerprints(topology_profile.lanes);
        for lane_fingerprint in &lane_fingerprints {
            assert_parallel_lane_fingerprint_schema(lane_fingerprint, &expected_lane_ids);
        }
        topology_fingerprints.push(format_parallel_lane_topology_fingerprint(
            topology_profile.topology_id,
            topology_profile.host_a,
            topology_profile.host_b,
            lane_fingerprints,
        ));
    }
    topology_fingerprints.sort();
    topology_fingerprints
}

fn collect_parallel_lane_topology_host_pair_ids(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut host_pair_ids = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_host_pair_id(fingerprint))
        .collect::<Vec<_>>();
    host_pair_ids.sort();
    host_pair_ids
}

fn permute_role_pair_lanes(
    mut lanes: Vec<LivePostgresRolePairProfile>,
    permutation_id: &str,
) -> Vec<LivePostgresRolePairProfile> {
    match permutation_id {
        "baseline" => lanes,
        "reverse" => {
            lanes.reverse();
            lanes
        }
        "rotate_left_1" => {
            if !lanes.is_empty() {
                lanes.rotate_left(1);
            }
            lanes
        }
        "interleaved_even_then_odd" => {
            let mut even = Vec::with_capacity(lanes.len());
            let mut odd = Vec::with_capacity(lanes.len());
            for (idx, lane) in lanes.into_iter().enumerate() {
                if idx % 2 == 0 {
                    even.push(lane);
                } else {
                    odd.push(lane);
                }
            }
            even.extend(odd);
            even
        }
        _ => panic!("unknown lane permutation: {permutation_id}"),
    }
}

fn run_live_postgres_matrix_repeated_run_projections() -> Option<(
    LivePostgresPhase6Projection,
    LivePostgresPhase6Projection,
    LivePostgresPhase6Projection,
    LivePostgresPhase6Projection,
)> {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return None;
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

    let applied_args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    let applied_first = run_daemon_for_phase6_projection(applied_args.clone());
    let applied_second = run_daemon_for_phase6_projection(applied_args);

    let deferred_args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ];
    let deferred_first = run_daemon_for_phase6_projection(deferred_args.clone());
    let deferred_second = run_daemon_for_phase6_projection(deferred_args);
    Some((
        applied_first,
        applied_second,
        deferred_first,
        deferred_second,
    ))
}

fn daemon_args_for_live_postgres_profile(
    role: &'static str,
    max_ticks: &'static str,
    tick_interval_ms: &'static str,
    shutdown: Option<(&'static str, &'static str, &'static str)>,
) -> Vec<String> {
    let mut args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        role.to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        max_ticks.to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        tick_interval_ms.to_owned(),
    ];
    if let Some((signal_tick, drain_ticks, timeout_ticks)) = shutdown {
        args.push("--daemon-shutdown-signal-tick".to_owned());
        args.push(signal_tick.to_owned());
        args.push("--daemon-shutdown-drain-ticks".to_owned());
        args.push(drain_ticks.to_owned());
        args.push("--daemon-shutdown-timeout-ticks".to_owned());
        args.push(timeout_ticks.to_owned());
    }
    args
}

fn project_live_postgres_load_profiles() -> Vec<LivePostgresLoadProfile> {
    vec![
        LivePostgresLoadProfile {
            profile_id: "applied_t3_i10",
            args: daemon_args_for_live_postgres_profile("processor", "3", "10", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "applied_t5_i25",
            args: daemon_args_for_live_postgres_profile("processor", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "applied_t9_i40",
            args: daemon_args_for_live_postgres_profile("processor", "9", "40", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "deferred_t5_i25_s3_d2_to4",
            args: daemon_args_for_live_postgres_profile(
                "processor",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "deferred_t7_i25_s3_d2_to4",
            args: daemon_args_for_live_postgres_profile(
                "processor",
                "7",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "deferred_t9_i40_s3_d2_to4",
            args: daemon_args_for_live_postgres_profile(
                "processor",
                "9",
                "40",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
    ]
}

fn project_live_postgres_role_profiles() -> Vec<LivePostgresLoadProfile> {
    vec![
        LivePostgresLoadProfile {
            profile_id: "processor_applied",
            args: daemon_args_for_live_postgres_profile("processor", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "processor_deferred",
            args: daemon_args_for_live_postgres_profile(
                "processor",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "listener_applied",
            args: daemon_args_for_live_postgres_profile("listener", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "listener_deferred",
            args: daemon_args_for_live_postgres_profile(
                "listener",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "approver_applied",
            args: daemon_args_for_live_postgres_profile("approver", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresLoadProfile {
            profile_id: "approver_deferred",
            args: daemon_args_for_live_postgres_profile(
                "approver",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
    ]
}

fn project_live_postgres_role_pair_profiles() -> Vec<LivePostgresRolePairProfile> {
    vec![
        LivePostgresRolePairProfile {
            pair_id: "processor_to_listener_applied",
            leg_a_profile_id: "processor_applied",
            leg_a_args: daemon_args_for_live_postgres_profile("processor", "5", "25", None),
            leg_b_profile_id: "listener_applied",
            leg_b_args: daemon_args_for_live_postgres_profile("listener", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "processor_to_listener_deferred",
            leg_a_profile_id: "processor_deferred",
            leg_a_args: daemon_args_for_live_postgres_profile(
                "processor",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            leg_b_profile_id: "listener_deferred",
            leg_b_args: daemon_args_for_live_postgres_profile(
                "listener",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "listener_to_approver_applied",
            leg_a_profile_id: "listener_applied",
            leg_a_args: daemon_args_for_live_postgres_profile("listener", "5", "25", None),
            leg_b_profile_id: "approver_applied",
            leg_b_args: daemon_args_for_live_postgres_profile("approver", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "listener_to_approver_deferred",
            leg_a_profile_id: "listener_deferred",
            leg_a_args: daemon_args_for_live_postgres_profile(
                "listener",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            leg_b_profile_id: "approver_deferred",
            leg_b_args: daemon_args_for_live_postgres_profile(
                "approver",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "approver_to_processor_applied",
            leg_a_profile_id: "approver_applied",
            leg_a_args: daemon_args_for_live_postgres_profile("approver", "5", "25", None),
            leg_b_profile_id: "processor_applied",
            leg_b_args: daemon_args_for_live_postgres_profile("processor", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "approver_to_processor_deferred",
            leg_a_profile_id: "approver_deferred",
            leg_a_args: daemon_args_for_live_postgres_profile(
                "approver",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            leg_b_profile_id: "processor_deferred",
            leg_b_args: daemon_args_for_live_postgres_profile(
                "processor",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
    ]
}

fn project_live_postgres_parallel_role_pair_lanes() -> Vec<LivePostgresRolePairProfile> {
    vec![
        LivePostgresRolePairProfile {
            pair_id: "processor_listener_parallel_applied",
            leg_a_profile_id: "processor_applied",
            leg_a_args: daemon_args_for_live_postgres_profile("processor", "5", "25", None),
            leg_b_profile_id: "listener_applied",
            leg_b_args: daemon_args_for_live_postgres_profile("listener", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "processor_listener_parallel_deferred",
            leg_a_profile_id: "processor_deferred",
            leg_a_args: daemon_args_for_live_postgres_profile(
                "processor",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            leg_b_profile_id: "listener_deferred",
            leg_b_args: daemon_args_for_live_postgres_profile(
                "listener",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "listener_approver_parallel_applied",
            leg_a_profile_id: "listener_applied",
            leg_a_args: daemon_args_for_live_postgres_profile("listener", "5", "25", None),
            leg_b_profile_id: "approver_applied",
            leg_b_args: daemon_args_for_live_postgres_profile("approver", "5", "25", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "listener_approver_parallel_deferred",
            leg_a_profile_id: "listener_deferred",
            leg_a_args: daemon_args_for_live_postgres_profile(
                "listener",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            leg_b_profile_id: "approver_deferred",
            leg_b_args: daemon_args_for_live_postgres_profile(
                "approver",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
    ]
}

fn project_live_postgres_asymmetric_parallel_lanes() -> Vec<LivePostgresRolePairProfile> {
    vec![
        LivePostgresRolePairProfile {
            pair_id: "processor_listener_asymmetric_parallel_applied",
            leg_a_profile_id: "processor_applied_t3_i10",
            leg_a_args: daemon_args_for_live_postgres_profile("processor", "3", "10", None),
            leg_b_profile_id: "listener_applied_t9_i40",
            leg_b_args: daemon_args_for_live_postgres_profile("listener", "9", "40", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "processor_listener_asymmetric_parallel_deferred",
            leg_a_profile_id: "processor_deferred_t5_i25",
            leg_a_args: daemon_args_for_live_postgres_profile(
                "processor",
                "5",
                "25",
                Some(("3", "2", "4")),
            ),
            leg_b_profile_id: "listener_deferred_t9_i40",
            leg_b_args: daemon_args_for_live_postgres_profile(
                "listener",
                "9",
                "40",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "listener_approver_asymmetric_parallel_applied",
            leg_a_profile_id: "listener_applied_t7_i25",
            leg_a_args: daemon_args_for_live_postgres_profile("listener", "7", "25", None),
            leg_b_profile_id: "approver_applied_t9_i40",
            leg_b_args: daemon_args_for_live_postgres_profile("approver", "9", "40", None),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        },
        LivePostgresRolePairProfile {
            pair_id: "listener_approver_asymmetric_parallel_deferred",
            leg_a_profile_id: "listener_deferred_t7_i25",
            leg_a_args: daemon_args_for_live_postgres_profile(
                "listener",
                "7",
                "25",
                Some(("3", "2", "4")),
            ),
            leg_b_profile_id: "approver_deferred_t9_i40",
            leg_b_args: daemon_args_for_live_postgres_profile(
                "approver",
                "9",
                "40",
                Some(("3", "2", "4")),
            ),
            expected_reason_code: LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        },
    ]
}

fn project_live_postgres_parallel_lane_topology_profiles(
) -> Vec<LivePostgresParallelLaneTopologyProfile> {
    vec![
        LivePostgresParallelLaneTopologyProfile {
            topology_id: "same_host_parallel",
            host_a: "node_alpha",
            host_b: "node_alpha",
            lanes: project_live_postgres_parallel_role_pair_lanes(),
        },
        LivePostgresParallelLaneTopologyProfile {
            topology_id: "distributed_label_parallel",
            host_a: "node_alpha",
            host_b: "node_beta",
            lanes: project_live_postgres_asymmetric_parallel_lanes(),
        },
    ]
}

#[test]
fn functional_runtime_daemon_emits_structured_transition_markers() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "99".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
    ])
    .expect("daemon args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");

    let start_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.start\""))
        .expect("daemon execution should emit structured start marker");
    assert_eq!(
        extract_json_string_field(start_line, "runtime_mode").as_deref(),
        Some("daemon")
    );
    assert_eq!(
        extract_json_string_field(start_line, "max_ticks").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(start_line, "tick_interval_ms").as_deref(),
        Some("25")
    );
    let start_execution_id = extract_json_string_field(start_line, "execution_id")
        .expect("daemon start marker should include execution_id");

    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "runtime_mode").as_deref(),
        Some("daemon")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "executed_ticks").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("tick-budget-exhausted;ignored_signals=1")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("not-signaled")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-not-requested")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("none")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("1")
    );
    let complete_execution_id = extract_json_string_field(complete_line, "execution_id")
        .expect("daemon completion marker should include execution_id");
    assert_eq!(start_execution_id, complete_execution_id);
}

#[test]
fn functional_runtime_daemon_graceful_shutdown_emits_structured_drain_markers() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ])
    .expect("daemon args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");

    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("completed")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("2")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("4")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-flushed")
    );
}

#[test]
pub(super) fn regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "7".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "4".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "2".to_owned(),
    ])
    .expect("daemon timeout args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon timeout execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_convergence_decision\":\"no_go\""));
    assert!(rendered.contains(
        "\"daemon_convergence_reason_code\":\"convergence_performance_budget_exceeded\""
    ));

    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("timeout")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("7")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("4")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("2")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-flush-timeout")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_decision").as_deref(),
        Some("no_go")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_reason_code").as_deref(),
        Some("convergence_performance_budget_exceeded")
    );
}

#[test]
fn parses_runtime_mode_daemon_with_bounded_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "99".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "start-connect".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::daemon());
    assert_eq!(parsed.daemon_max_ticks, Some(3));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_peer_id, Some("peer-alpha".to_owned()));
    assert_eq!(parsed.daemon_lifecycle_events.len(), 2);
}

#[test]
fn parses_runtime_mode_daemon_with_shutdown_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "8".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with shutdown controls should parse");
    assert_eq!(parsed.daemon_shutdown_signal_ticks, vec![3]);
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_daemon_with_os_signal_shutdown_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "12".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--daemon-shutdown-os-signals".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with os signal controls should parse");
    assert_eq!(parsed.daemon_shutdown_signal_ticks, Vec::<u64>::new());
    assert!(parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_daemon_with_observability_endpoint_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "12".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--observability-endpoint-bind".to_owned(),
        "127.0.0.1:9108".to_owned(),
        "--observability-endpoint-metrics-path".to_owned(),
        "/runtime/metrics".to_owned(),
        "--observability-endpoint-health-path".to_owned(),
        "/runtime/health".to_owned(),
        "--observability-endpoint-max-requests".to_owned(),
        "3".to_owned(),
        "--observability-endpoint-idle-timeout-ms".to_owned(),
        "1200".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with observability endpoint should parse");
    assert_eq!(
        parsed.observability_endpoint_bind_addr,
        Some("127.0.0.1:9108".to_owned())
    );
    assert_eq!(
        parsed.observability_endpoint_metrics_path,
        "/runtime/metrics"
    );
    assert_eq!(parsed.observability_endpoint_health_path, "/runtime/health");
    assert_eq!(parsed.observability_endpoint_max_requests, 3);
    assert_eq!(parsed.observability_endpoint_idle_timeout_ms, 1200);
}

#[test]
fn env_only_daemon_controls_parse_without_config_file() {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .expect("daemon env lock should guard process-level overrides");
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", Some("12"));
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", Some("25"));

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
    ];

    let parsed = parse_args(args).expect("env-only daemon controls should parse");
    assert_eq!(parsed.daemon_max_ticks, Some(12));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
}

#[test]
fn regression_3202_invalid_daemon_env_override_fails_closed_without_config_file() {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .expect("daemon env lock should guard process-level overrides");
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", Some("invalid"));
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", Some("25"));

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
    ];

    let parse_result = parse_args(args);
    assert!(
        matches!(
            parse_result,
            Err(ConfigError::InvalidDaemonControlArgument(value)) if value == "invalid"
        ),
        "invalid daemon env override must fail closed with typed daemon control error"
    );
}

#[test]
fn integration_runtime_daemon_renders_bounded_completion_output() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "99".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "start-connect".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "heartbeat-missed".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "heartbeat-restored".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"daemon\""));
    assert!(rendered.contains("\"daemon_max_ticks\":3"));
    assert!(rendered.contains("\"daemon_tick_interval_ms\":25"));
    assert!(rendered.contains("\"daemon_executed_ticks\":3"));
    assert!(rendered
        .contains("\"daemon_completion_reason\":\"tick-budget-exhausted;ignored_signals=1\""));
    assert!(rendered.contains("\"daemon_observability_latency_p50_ms\":25"));
    assert!(rendered.contains("\"daemon_observability_latency_p99_ms\":50"));
    assert!(rendered.contains("\"daemon_observability_throughput_tps\":2000"));
    assert!(rendered.contains("\"daemon_observability_error_rate_bps\":50"));
    assert!(rendered.contains("\"daemon_observability_availability_bps\":9990"));
    assert!(rendered.contains("\"daemon_observability_health\":\"healthy\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":0"));
    assert!(rendered.contains("\"daemon_observability_reason_code\":\"none\""));
    assert!(rendered.contains("\"daemon_observability_transport_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_signer_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_commit_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_peer_id\":\"peer-alpha\""));
    assert!(rendered.contains("\"daemon_peer_lifecycle_final_state\":\"active\""));
    assert!(
        rendered.contains(
            "\"daemon_peer_lifecycle_applied_events\":[\"start-connect\",\"handshake-succeeded\",\"heartbeat-missed\",\"heartbeat-restored\"]"
        )
    );
}

#[test]
fn functional_runtime_daemon_applies_graceful_shutdown_signal() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon shutdown args should parse");
    let report = execute(parsed).expect("daemon graceful shutdown execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":5"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=0\""
    ));
    assert!(rendered.contains("\"daemon_observability_health\":\"healthy\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":0"));
}

#[test]
fn integration_runtime_daemon_shutdown_timeout_is_fail_closed() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "7".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "4".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "2".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon timeout args should parse");
    let report = execute(parsed).expect("daemon timeout execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":9"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0\""
    ));
    assert!(rendered.contains("\"daemon_observability_latency_p50_ms\":145"));
    assert!(rendered.contains("\"daemon_observability_latency_p99_ms\":425"));
    assert!(rendered.contains("\"daemon_observability_throughput_tps\":900"));
    assert!(rendered.contains("\"daemon_observability_error_rate_bps\":250"));
    assert!(rendered.contains("\"daemon_observability_availability_bps\":9800"));
    assert!(rendered.contains("\"daemon_observability_health\":\"critical\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":4"));
    assert!(rendered.contains("\"daemon_observability_reason_code\":\"daemon_shutdown_timeout\""));
    assert!(rendered.contains("\"daemon_observability_transport_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_signer_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_commit_checkpoint_failures\":1"));
}

#[cfg(unix)]
#[test]
pub(super) fn integration_runtime_daemon_applies_graceful_shutdown_on_os_signal() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "100".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-os-signals".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "5".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed =
        parse_args_with_clean_daemon_env(args).expect("daemon os-signal args should parse");
    configure_os_signal_test_triggers(vec![OsSignalTestTrigger::new(5, OsSignalTestKind::Sigterm)]);
    let report = execute(parsed).expect("daemon os-signal execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_completion_reason\":\"graceful-shutdown:signal@"));
}

#[test]
fn functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains(
        "\"daemon_phase6_runtime_reason_taxonomy_version\":\"kamn.runtime.daemon.phase6.reason-taxonomy.v1\""
    ));
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_applied\""));
    assert!(rendered.contains(
        "\"daemon_convergence_reason_taxonomy_version\":\"kamn.runtime.daemon.convergence.reason-taxonomy.v1\""
    ));
    assert!(rendered.contains("\"daemon_convergence_decision\":\"go\""));
    assert!(
        rendered.contains("\"daemon_convergence_reason_code\":\"convergence_promotion_gate_go\"")
    );
    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "phase6_reason_taxonomy_version").as_deref(),
        Some("kamn.runtime.daemon.phase6.reason-taxonomy.v1")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "phase6_reason_codes_csv").as_deref(),
        Some("m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded")
    );
    assert!(
        extract_json_string_field(complete_line, "phase6_reason_code").as_deref()
            == Some("m10_phase6_scheduler_cycle_applied")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_reason_taxonomy_version").as_deref(),
        Some("kamn.runtime.daemon.convergence.reason-taxonomy.v1")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_decision").as_deref(),
        Some("go")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_reason_code").as_deref(),
        Some("convergence_promotion_gate_go")
    );
}

#[test]
fn functional_runtime_daemon_projects_phase6_deferred_runtime_markers_when_shutdown_signals_are_present(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_deferred\""));
    assert!(rendered.contains("\"daemon_phase6_runtime_deferred_cycles\":1"));
    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert!(
        extract_json_string_field(complete_line, "phase6_reason_code").as_deref()
            == Some("m10_phase6_scheduler_cycle_deferred")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "phase6_deferred_cycles").as_deref(),
        Some("1")
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice() {
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

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains(
        "\"daemon_phase6_runtime_reason_taxonomy_version\":\"kamn.runtime.daemon.phase6.reason-taxonomy.v1\""
    ));
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_applied\""));
}

#[test]
fn regression_runtime_daemon_live_postgres_validation_slice_reports_unset_env_gate_reason() {
    // Regression: #5340
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _test_postgres_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", None);
    let _database_guard = EnvVarGuard::set("DATABASE_URL", None);
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
    assert!(maybe_database_url.is_none());
}

#[test]
fn unit_runtime_daemon_live_postgres_validation_slice_prefers_kamn_test_postgres_url() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let preferred = "postgres://preferred:5432/kamn_test";
    let fallback = "postgres://fallback:5432/kamn_test";
    let _test_postgres_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some(preferred));
    let _database_guard = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );
    assert_eq!(maybe_database_url.as_deref(), Some(preferred));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_deferred_path() {
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

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_deferred\""));
    assert!(rendered.contains("\"daemon_phase6_runtime_deferred_cycles\":1"));
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_env_matrix_contract_is_deterministic() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");

    let _unset_primary = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", None);
    let _unset_fallback = EnvVarGuard::set("DATABASE_URL", None);
    let (reason_unset, url_unset) = resolve_live_postgres_gate_decision();
    assert_eq!(reason_unset, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
    assert!(url_unset.is_none());
    drop(_unset_fallback);
    drop(_unset_primary);

    let preferred = "postgres://preferred:5432/kamn_test";
    let fallback = "postgres://fallback:5432/kamn_test";
    let _preferred_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some(preferred));
    let _fallback_guard = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (reason_preferred, url_preferred) = resolve_live_postgres_gate_decision();
    assert_eq!(
        reason_preferred,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );
    assert_eq!(url_preferred.as_deref(), Some(preferred));
    drop(_fallback_guard);
    drop(_preferred_guard);

    let _blank_primary = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some("   "));
    let _fallback_only = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (reason_fallback, url_fallback) = resolve_live_postgres_gate_decision();
    assert_eq!(reason_fallback, LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE);
    assert_eq!(url_fallback.as_deref(), Some(fallback));
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical(
) {
    let rows = project_live_postgres_matrix_rows();
    assert_eq!(
        rows,
        vec![
            LivePostgresMatrixRow {
                scenario_id: "env_unset",
                gate_reason_code: LIVE_POSTGRES_ENV_UNSET_REASON_CODE,
                daemon_phase6_reason_code: None,
            },
            LivePostgresMatrixRow {
                scenario_id: "env_set_no_shutdown",
                gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
                daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE),
            },
            LivePostgresMatrixRow {
                scenario_id: "env_set_shutdown",
                gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
                daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE),
            },
        ],
        "matrix projection rows must remain canonical and ordered"
    );
    let scenario_csv = rows
        .iter()
        .map(|row| row.scenario_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(scenario_csv, LIVE_POSTGRES_MATRIX_SCENARIOS_CSV);

    let reason_codes_csv = format!(
        "{},{},{}",
        LIVE_POSTGRES_ENV_UNSET_REASON_CODE,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(reason_codes_csv, LIVE_POSTGRES_MATRIX_REASON_CODES_CSV);
    assert_eq!(
        LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    );
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_matrix_taxonomy_bridge_contract_is_canonical(
) {
    let rows = project_live_postgres_matrix_rows();
    let bridge_reason_codes_csv = rows
        .iter()
        .filter_map(|row| row.daemon_phase6_reason_code)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    );
    assert_eq!(
        bridge_reason_codes_csv,
        LIVE_POSTGRES_RUNTIME_TO_MATRIX_BRIDGE_REASON_CODES_CSV
    );
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical(
) {
    let profiles = project_live_postgres_load_profiles();
    let profile_ids_csv = profiles
        .iter()
        .map(|profile| profile.profile_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(profile_ids_csv, LIVE_POSTGRES_MATRIX_LOAD_PROFILE_IDS_CSV);

    assert!(profiles
        .iter()
        .take(3)
        .all(|profile| profile.expected_reason_code
            == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(profiles
        .iter()
        .skip(3)
        .all(|profile| profile.expected_reason_code
            == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_load_profile_matrix_is_deterministic(
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

    for profile in project_live_postgres_load_profiles() {
        let first = run_daemon_for_phase6_projection(profile.args.clone());
        let second = run_daemon_for_phase6_projection(profile.args);
        assert_eq!(
            first.reason_code, profile.expected_reason_code,
            "profile {} should project expected phase6 reason code",
            profile.profile_id
        );
        assert_eq!(
            first.reason_code, second.reason_code,
            "profile {} reason code should remain stable across repeated runs",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "profile {} should remain bridged to runtime taxonomy version",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, second.reason_taxonomy_version,
            "profile {} taxonomy version should remain stable across repeated runs",
            profile.profile_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_role_profile_matrix_contract_is_canonical(
) {
    let profiles = project_live_postgres_role_profiles();
    let profile_ids_csv = profiles
        .iter()
        .map(|profile| profile.profile_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(profile_ids_csv, LIVE_POSTGRES_MATRIX_ROLE_PROFILE_IDS_CSV);

    assert_eq!(
        profiles[0].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[1].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(
        profiles[2].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[3].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(
        profiles[4].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[5].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_role_profile_matrix_is_deterministic(
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

    for profile in project_live_postgres_role_profiles() {
        let first = run_daemon_for_phase6_projection(profile.args.clone());
        let second = run_daemon_for_phase6_projection(profile.args);
        assert_eq!(
            first.reason_code, profile.expected_reason_code,
            "role profile {} should project expected phase6 reason code",
            profile.profile_id
        );
        assert_eq!(
            first.reason_code, second.reason_code,
            "role profile {} reason code should remain stable across repeated runs",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "role profile {} should remain bridged to runtime taxonomy version",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, second.reason_taxonomy_version,
            "role profile {} taxonomy version should remain stable across repeated runs",
            profile.profile_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_role_pair_matrix_contract_is_canonical()
{
    let pairs = project_live_postgres_role_pair_profiles();
    let pair_ids_csv = pairs
        .iter()
        .map(|pair| pair.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(pair_ids_csv, LIVE_POSTGRES_MATRIX_ROLE_PAIR_IDS_CSV);
    assert!(pairs
        .iter()
        .step_by(2)
        .all(|pair| pair.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(pairs
        .iter()
        .skip(1)
        .step_by(2)
        .all(|pair| pair.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_role_pair_matrix_is_deterministic(
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

    for pair in project_live_postgres_role_pair_profiles() {
        let leg_a_first = run_daemon_for_phase6_projection(pair.leg_a_args.clone());
        let leg_a_second = run_daemon_for_phase6_projection(pair.leg_a_args);
        let leg_b_first = run_daemon_for_phase6_projection(pair.leg_b_args.clone());
        let leg_b_second = run_daemon_for_phase6_projection(pair.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, pair.expected_reason_code,
            "pair {} leg A ({}) should project expected phase6 reason code",
            pair.pair_id, pair.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, pair.expected_reason_code,
            "pair {} leg B ({}) should project expected phase6 reason code",
            pair.pair_id, pair.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "pair {} leg A reason code should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "pair {} leg B reason code should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "pair {} leg A taxonomy should stay on runtime taxonomy version",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "pair {} leg B taxonomy should stay on runtime taxonomy version",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "pair {} leg A taxonomy should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "pair {} leg B taxonomy should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "pair {} legs should share the same runtime taxonomy version",
            pair.pair_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_role_pair_lane_contract_is_canonical(
) {
    let lanes = project_live_postgres_parallel_role_pair_lanes();
    let lane_ids_csv = lanes
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        lane_ids_csv,
        LIVE_POSTGRES_MATRIX_PARALLEL_ROLE_PAIR_LANE_IDS_CSV
    );
    assert!(lanes
        .iter()
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(lanes
        .iter()
        .skip(1)
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_role_pair_lane_is_deterministic(
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

    for lane in project_live_postgres_parallel_role_pair_lanes() {
        let (leg_a_first, leg_b_first) =
            run_parallel_phase6_projections(lane.leg_a_args.clone(), lane.leg_b_args.clone());
        let (leg_a_second, leg_b_second) =
            run_parallel_phase6_projections(lane.leg_a_args, lane.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, lane.expected_reason_code,
            "parallel lane {} leg A ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, lane.expected_reason_code,
            "parallel lane {} leg B ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "parallel lane {} leg A reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "parallel lane {} leg B reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "parallel lane {} leg A taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "parallel lane {} leg B taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "parallel lane {} leg A taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "parallel lane {} leg B taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "parallel lane {} legs should share the same runtime taxonomy version",
            lane.pair_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_asymmetric_parallel_lane_contract_is_canonical(
) {
    let lanes = project_live_postgres_asymmetric_parallel_lanes();
    let lane_ids_csv = lanes
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        lane_ids_csv,
        LIVE_POSTGRES_MATRIX_ASYMMETRIC_PARALLEL_LANE_IDS_CSV
    );
    assert!(lanes
        .iter()
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(lanes
        .iter()
        .skip(1)
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_asymmetric_parallel_lane_is_deterministic(
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

    for lane in project_live_postgres_asymmetric_parallel_lanes() {
        let (leg_a_first, leg_b_first) =
            run_parallel_phase6_projections(lane.leg_a_args.clone(), lane.leg_b_args.clone());
        let (leg_a_second, leg_b_second) =
            run_parallel_phase6_projections(lane.leg_a_args, lane.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, lane.expected_reason_code,
            "asymmetric lane {} leg A ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, lane.expected_reason_code,
            "asymmetric lane {} leg B ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "asymmetric lane {} leg A reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "asymmetric lane {} leg B reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "asymmetric lane {} leg A taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "asymmetric lane {} leg B taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "asymmetric lane {} leg A taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "asymmetric lane {} leg B taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "asymmetric lane {} legs should share the same runtime taxonomy version",
            lane.pair_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_order_invariance_contract_is_canonical(
) {
    let lane_sets_csv = ["symmetric_parallel", "asymmetric_parallel"].join(",");
    assert_eq!(
        lane_sets_csv,
        LIVE_POSTGRES_MATRIX_ORDER_INVARIANCE_LANE_SETS_CSV
    );

    let mut expected_symmetric_fingerprints = project_live_postgres_parallel_role_pair_lanes()
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    expected_symmetric_fingerprints.sort();

    let mut expected_asymmetric_fingerprints = project_live_postgres_asymmetric_parallel_lanes()
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    expected_asymmetric_fingerprints.sort();

    assert_eq!(
        expected_symmetric_fingerprints,
        vec![
            "listener_approver_parallel_applied",
            "listener_approver_parallel_deferred",
            "processor_listener_parallel_applied",
            "processor_listener_parallel_deferred"
        ]
    );
    assert_eq!(
        expected_asymmetric_fingerprints,
        vec![
            "listener_approver_asymmetric_parallel_applied",
            "listener_approver_asymmetric_parallel_deferred",
            "processor_listener_asymmetric_parallel_applied",
            "processor_listener_asymmetric_parallel_deferred"
        ]
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_order_is_invariant(
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

    let symmetric_baseline =
        run_parallel_lane_set_fingerprints(project_live_postgres_parallel_role_pair_lanes());
    let mut symmetric_permuted_lanes = project_live_postgres_parallel_role_pair_lanes();
    symmetric_permuted_lanes.reverse();
    let symmetric_permuted = run_parallel_lane_set_fingerprints(symmetric_permuted_lanes);
    assert_eq!(
        symmetric_baseline, symmetric_permuted,
        "symmetric parallel lane fingerprints should remain invariant to lane order"
    );

    let asymmetric_baseline =
        run_parallel_lane_set_fingerprints(project_live_postgres_asymmetric_parallel_lanes());
    let mut asymmetric_permuted_lanes = project_live_postgres_asymmetric_parallel_lanes();
    asymmetric_permuted_lanes.reverse();
    let asymmetric_permuted = run_parallel_lane_set_fingerprints(asymmetric_permuted_lanes);
    assert_eq!(
        asymmetric_baseline, asymmetric_permuted,
        "asymmetric parallel lane fingerprints should remain invariant to lane order"
    );
}

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

use super::*;

pub(super) const LIVE_POSTGRES_ENV_UNSET_REASON_CODE: &str = "live_postgres_env_unset";
pub(super) const LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE: &str =
    "live_postgres_adapter_connected";
pub(super) const LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.daemon.phase6.reason-taxonomy.v1";
pub(super) const LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1";
pub(super) const LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_applied";
pub(super) const LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_deferred";
pub(super) const LIVE_POSTGRES_RUNTIME_TO_MATRIX_BRIDGE_REASON_CODES_CSV: &str =
    "m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred";
pub(super) const LIVE_POSTGRES_MATRIX_LOAD_PROFILE_IDS_CSV: &str = "applied_t3_i10,applied_t5_i25,applied_t9_i40,deferred_t5_i25_s3_d2_to4,deferred_t7_i25_s3_d2_to4,deferred_t9_i40_s3_d2_to4";
pub(super) const LIVE_POSTGRES_MATRIX_ROLE_PROFILE_IDS_CSV: &str = "processor_applied,processor_deferred,listener_applied,listener_deferred,approver_applied,approver_deferred";
pub(super) const LIVE_POSTGRES_MATRIX_ROLE_PAIR_IDS_CSV: &str = "processor_to_listener_applied,processor_to_listener_deferred,listener_to_approver_applied,listener_to_approver_deferred,approver_to_processor_applied,approver_to_processor_deferred";
pub(super) const LIVE_POSTGRES_MATRIX_PARALLEL_ROLE_PAIR_LANE_IDS_CSV: &str = "processor_listener_parallel_applied,processor_listener_parallel_deferred,listener_approver_parallel_applied,listener_approver_parallel_deferred";
pub(super) const LIVE_POSTGRES_MATRIX_ASYMMETRIC_PARALLEL_LANE_IDS_CSV: &str = "processor_listener_asymmetric_parallel_applied,processor_listener_asymmetric_parallel_deferred,listener_approver_asymmetric_parallel_applied,listener_approver_asymmetric_parallel_deferred";
pub(super) const LIVE_POSTGRES_MATRIX_PERMUTATION_IDS_CSV: &str =
    "baseline,reverse,rotate_left_1,interleaved_even_then_odd";
pub(super) const LIVE_POSTGRES_MATRIX_ORDER_INVARIANCE_LANE_SETS_CSV: &str =
    "symmetric_parallel,asymmetric_parallel";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-fingerprint.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_ORDER_CSV: &str =
    "lane_id,leg_a_reason,leg_a_taxonomy,leg_b_reason,leg_b_taxonomy";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER: char = '|';
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_COUNT: usize = 5;
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_IDS_CSV: &str =
    "same_host_parallel,distributed_label_parallel";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_CONTRACT: &str =
    "topology_labels_must_preserve_sorted_lane_reason_taxonomy_fingerprints_under_repeated_runs";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_PERMUTATION_IDS_CSV: &str =
    "baseline,reverse,rotate_left_1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_PERMUTATION_CONTRACT: &str =
    "deterministic_topology_profile_permutations_must_preserve_sorted_topology_fingerprint_bundles";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_REQUIRED_HOST_PAIR_IDS_CSV: &str =
    "node_alpha->node_alpha,node_alpha->node_beta";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CONTRACT: &str =
    "host_pair_ids_must_remain_stable_under_repeated_runs_and_topology_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_SCHEMA_VERSION:
    &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-directionality.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_EXTRACTION_RULE:
    &str = "host_a_to_host_b_arrow_notation_non_commutative";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_FORBIDDEN_REVERSE_PAIRS_CSV:
    &str = "node_beta->node_alpha";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-mapping.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->node_alpha->node_alpha,distributed_label_parallel->node_alpha->node_beta";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_CONTRACT: &str =
    "topology_id_to_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-set-mapping.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->symmetric_parallel,distributed_label_parallel->asymmetric_parallel";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_CONTRACT: &str =
    "topology_id_to_lane_set_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-count-mapping.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->4,distributed_label_parallel->4";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_CONTRACT: &str =
    "topology_id_to_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-mapping.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->same_host,distributed_label_parallel->distributed_label";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_CONTRACT: &str =
    "topology_id_to_host_mode_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_SCHEMA_VERSION:
    &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-cardinality-mapping.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->1,distributed_label_parallel->2";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_CONTRACT: &str =
    "topology_id_to_unique_host_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-cardinality-coherence.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_ROWS_CSV:
    &str = "same_host_parallel->same_host->1,distributed_label_parallel->distributed_label->2";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-cardinality-coherence.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->node_alpha->node_alpha->1,distributed_label_parallel->node_alpha->node_beta->2";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_pair_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-coherence.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha,distributed_label_parallel->distributed_label->node_alpha->node_beta";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-cardinality-coherence.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha->1,distributed_label_parallel->distributed_label->node_alpha->node_beta->2";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-count-coherence.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->4,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->4";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_lane_set_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-id-bundle-coherence.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied+listener_approver_parallel_deferred+processor_listener_parallel_applied+processor_listener_parallel_deferred,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied+listener_approver_asymmetric_parallel_deferred+processor_listener_asymmetric_parallel_applied+processor_listener_asymmetric_parallel_deferred";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_lane_set_lane_id_bundle_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_BUNDLE_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-bundle-coherence.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_BUNDLE_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+listener_approver_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+listener_approver_asymmetric_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_asymmetric_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_BUNDLE_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_bundle_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-coherence.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_COHERENCE_ROWS_CSV: &str =
    "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-order-normalization.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_ROWS_CSV: &str =
    "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_must_remain_canonically_sorted_after_order_normalization";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-order-normalization-digest.v1";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_CSV: &str =
    "distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_FNV1A64_HEX: &str =
    "25b9729eaeb44fe9";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_digest_must_remain_stable_under_order_normalization_and_permutations";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_ORDER_CSV: &str =
    "topology_id,host_a,host_b,lane_fingerprint_bundle";
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER: char = '#';
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_BUNDLE_DELIMITER: char = ';';
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT: usize = 4;
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_ID_BUNDLE_DELIMITER: char = '+';
pub(super) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_FINGERPRINT_BUNDLE_DELIMITER: char = '+';
pub(super) const LIVE_POSTGRES_MATRIX_REASON_CODES_CSV: &str =
    "live_postgres_env_unset,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred";
pub(super) const LIVE_POSTGRES_MATRIX_SCENARIOS_CSV: &str =
    "env_unset,env_set_no_shutdown,env_set_shutdown";

pub(super) fn parse_args_with_clean_daemon_env(
    args: Vec<String>,
) -> Result<crate::NodeCli, ConfigError> {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .expect("daemon env lock should guard process-level overrides");
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", None);
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", None);
    parse_args(args)
}

pub(super) fn live_postgres_url() -> Option<String> {
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

pub(super) fn resolve_live_postgres_gate_decision() -> (&'static str, Option<String>) {
    match live_postgres_url() {
        Some(database_url) => (
            LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            Some(database_url),
        ),
        None => (LIVE_POSTGRES_ENV_UNSET_REASON_CODE, None),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct LivePostgresMatrixRow {
    pub(super) scenario_id: &'static str,
    pub(super) gate_reason_code: &'static str,
    pub(super) daemon_phase6_reason_code: Option<&'static str>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct LivePostgresPhase6Projection {
    pub(super) reason_code: String,
    pub(super) reason_taxonomy_version: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct LivePostgresLoadProfile {
    pub(super) profile_id: &'static str,
    pub(super) args: Vec<String>,
    pub(super) expected_reason_code: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct LivePostgresRolePairProfile {
    pub(super) pair_id: &'static str,
    pub(super) leg_a_profile_id: &'static str,
    pub(super) leg_a_args: Vec<String>,
    pub(super) leg_b_profile_id: &'static str,
    pub(super) leg_b_args: Vec<String>,
    pub(super) expected_reason_code: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct LivePostgresParallelLaneTopologyProfile {
    pub(super) topology_id: &'static str,
    pub(super) host_a: &'static str,
    pub(super) host_b: &'static str,
    pub(super) lanes: Vec<LivePostgresRolePairProfile>,
}

pub(super) fn project_live_postgres_matrix_rows() -> Vec<LivePostgresMatrixRow> {
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

pub(super) fn run_daemon_for_phase6_projection(
    mut args: Vec<String>,
) -> LivePostgresPhase6Projection {
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

pub(super) fn run_parallel_phase6_projections(
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

pub(super) fn format_parallel_lane_fingerprint(
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

pub(super) fn parse_parallel_lane_fingerprint_fields(fingerprint: &str) -> Vec<&str> {
    fingerprint
        .split(LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER)
        .collect::<Vec<_>>()
}

pub(super) fn assert_parallel_lane_fingerprint_schema(
    fingerprint: &str,
    expected_lane_ids: &[&str],
) {
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

pub(super) fn format_parallel_lane_topology_fingerprint(
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

pub(super) fn parse_parallel_lane_topology_fingerprint_fields(fingerprint: &str) -> Vec<&str> {
    fingerprint
        .split(LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER)
        .collect::<Vec<_>>()
}

pub(super) fn parse_parallel_lane_topology_bundle_fields(bundle: &str) -> Vec<&str> {
    if bundle.is_empty() {
        Vec::new()
    } else {
        bundle
            .split(LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_BUNDLE_DELIMITER)
            .collect::<Vec<_>>()
    }
}

pub(super) fn extract_parallel_lane_topology_host_pair_id(topology_fingerprint: &str) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for host-pair extraction"
    );
    format!("{}->{}", fields[1], fields[2])
}

pub(super) fn extract_parallel_lane_topology_host_pair_reverse_id(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for reverse host-pair extraction"
    );
    format!("{}->{}", fields[2], fields[1])
}

pub(super) fn extract_parallel_lane_topology_id_host_pair_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-pair mapping extraction"
    );
    format!("{}->{}->{}", fields[0], fields[1], fields[2])
}

pub(super) fn extract_parallel_lane_topology_id_lane_set_row(topology_fingerprint: &str) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id lane-set mapping extraction"
    );
    let lane_set = match fields[0] {
        "same_host_parallel" => "symmetric_parallel",
        "distributed_label_parallel" => "asymmetric_parallel",
        _ => panic!("unknown topology id {} for lane-set mapping", fields[0]),
    };
    format!("{}->{}", fields[0], lane_set)
}

pub(super) fn extract_parallel_lane_topology_id_lane_count_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id lane-count mapping extraction"
    );
    let lane_count = parse_parallel_lane_topology_bundle_fields(fields[3]).len();
    format!("{}->{}", fields[0], lane_count)
}

pub(super) fn extract_parallel_lane_topology_id_host_mode_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-mode mapping extraction"
    );
    let host_mode = if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    };
    format!("{}->{}", fields[0], host_mode)
}

pub(super) fn extract_parallel_lane_topology_id_host_cardinality_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-cardinality mapping extraction"
    );
    let unique_host_cardinality = if fields[1] == fields[2] { 1 } else { 2 };
    format!("{}->{}", fields[0], unique_host_cardinality)
}

pub(super) fn extract_parallel_lane_topology_id_host_mode_cardinality_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-mode-cardinality coherence extraction"
    );
    let host_mode = if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    };
    let unique_host_cardinality = if fields[1] == fields[2] { 1 } else { 2 };
    format!("{}->{}->{}", fields[0], host_mode, unique_host_cardinality)
}

pub(super) fn extract_parallel_lane_topology_id_host_pair_cardinality_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-pair-cardinality coherence extraction"
    );
    let unique_host_cardinality = if fields[1] == fields[2] { 1 } else { 2 };
    format!(
        "{}->{}->{}->{}",
        fields[0], fields[1], fields[2], unique_host_cardinality
    )
}

pub(super) fn extract_parallel_lane_topology_id_host_mode_host_pair_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair coherence extraction"
    );
    let host_mode = if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    };
    format!("{}->{}->{}->{}", fields[0], host_mode, fields[1], fields[2])
}

pub(super) fn extract_parallel_lane_topology_id_host_mode_host_pair_cardinality_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-cardinality coherence extraction"
    );
    let host_mode = if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    };
    let unique_host_cardinality = if fields[1] == fields[2] { 1 } else { 2 };
    format!(
        "{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], unique_host_cardinality
    )
}

pub(super) fn extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-lane-set-lane-count coherence extraction"
    );
    let host_mode = if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    };
    let lane_set = match fields[0] {
        "same_host_parallel" => "symmetric_parallel",
        "distributed_label_parallel" => "asymmetric_parallel",
        _ => panic!(
            "unknown topology id {} for host-mode-host-pair-lane-set-lane-count coherence extraction",
            fields[0]
        ),
    };
    let lane_count = parse_parallel_lane_topology_bundle_fields(fields[3]).len();
    format!(
        "{}->{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], lane_set, lane_count
    )
}

pub(super) fn extract_parallel_lane_topology_lane_id_bundle(topology_fingerprint: &str) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology lane-id-bundle extraction"
    );
    let mut lane_ids = parse_parallel_lane_topology_bundle_fields(fields[3])
        .iter()
        .map(|lane_fingerprint| {
            let lane_fields = parse_parallel_lane_fingerprint_fields(lane_fingerprint);
            assert_eq!(
                lane_fields.len(),
                LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_COUNT,
                "lane fingerprint should keep canonical field count for lane-id-bundle extraction"
            );
            lane_fields[0].to_owned()
        })
        .collect::<Vec<_>>();
    lane_ids.sort();
    lane_ids.join(&LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_ID_BUNDLE_DELIMITER.to_string())
}

pub(super) fn extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-lane-set-lane-id-bundle coherence extraction"
    );
    let host_mode = if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    };
    let lane_set = match fields[0] {
        "same_host_parallel" => "symmetric_parallel",
        "distributed_label_parallel" => "asymmetric_parallel",
        _ => panic!(
            "unknown topology id {} for host-mode-host-pair-lane-set-lane-id-bundle coherence extraction",
            fields[0]
        ),
    };
    let lane_id_bundle = extract_parallel_lane_topology_lane_id_bundle(topology_fingerprint);
    format!(
        "{}->{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], lane_set, lane_id_bundle
    )
}

pub(super) fn extract_parallel_lane_topology_lane_fingerprint_bundle(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology lane-fingerprint-bundle extraction"
    );
    let mut lane_fingerprints = parse_parallel_lane_topology_bundle_fields(fields[3])
        .iter()
        .map(|lane_fingerprint| (*lane_fingerprint).to_owned())
        .collect::<Vec<_>>();
    lane_fingerprints.sort();
    lane_fingerprints
        .join(&LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_FINGERPRINT_BUNDLE_DELIMITER.to_string())
}

pub(super) fn deterministic_fnv1a64_hex(input: &str) -> String {
    const FNV_OFFSET_BASIS_64: u64 = 0xcbf29ce484222325;
    const FNV_PRIME_64: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET_BASIS_64;
    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    format!("{hash:016x}")
}

pub(super) fn extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-lane-set-lane-fingerprint-hash coherence extraction"
    );
    let host_mode = if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    };
    let lane_set = match fields[0] {
        "same_host_parallel" => "symmetric_parallel",
        "distributed_label_parallel" => "asymmetric_parallel",
        _ => panic!(
            "unknown topology id {} for host-mode-host-pair-lane-set-lane-fingerprint-hash coherence extraction",
            fields[0]
        ),
    };
    let lane_fingerprint_bundle =
        extract_parallel_lane_topology_lane_fingerprint_bundle(topology_fingerprint);
    let lane_fingerprint_hash = deterministic_fnv1a64_hex(&lane_fingerprint_bundle);
    format!(
        "{}->{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], lane_set, lane_fingerprint_hash
    )
}

pub(super) fn extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-lane-set-lane-fingerprint-bundle coherence extraction"
    );
    let host_mode = if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    };
    let lane_set = match fields[0] {
        "same_host_parallel" => "symmetric_parallel",
        "distributed_label_parallel" => "asymmetric_parallel",
        _ => panic!(
            "unknown topology id {} for host-mode-host-pair-lane-set-lane-fingerprint-bundle coherence extraction",
            fields[0]
        ),
    };
    let lane_fingerprint_bundle =
        extract_parallel_lane_topology_lane_fingerprint_bundle(topology_fingerprint);
    format!(
        "{}->{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], lane_set, lane_fingerprint_bundle
    )
}

pub(super) fn permute_parallel_lane_topology_profiles(
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

pub(super) fn run_parallel_lane_set_fingerprints(
    lanes: Vec<LivePostgresRolePairProfile>,
) -> Vec<String> {
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

pub(super) fn run_parallel_lane_topology_fingerprints(
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

pub(super) fn collect_parallel_lane_topology_host_pair_ids(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut host_pair_ids = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_host_pair_id(fingerprint))
        .collect::<Vec<_>>();
    host_pair_ids.sort();
    host_pair_ids
}

pub(super) fn collect_parallel_lane_topology_id_host_pair_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_pair_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_lane_set_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_lane_set_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_lane_count_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_lane_count_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_mode_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_cardinality_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_cardinality_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_cardinality_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_mode_cardinality_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_pair_cardinality_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_pair_cardinality_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_host_pair_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_mode_host_pair_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_host_pair_cardinality_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_cardinality_row(fingerprint)
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_row(
                fingerprint,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_row(
                fingerprint,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_row(
                fingerprint,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_row(
                fingerprint,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(super) fn assert_parallel_lane_topology_rows_are_canonically_sorted(
    rows: &[String],
    context: &str,
) {
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort();
    assert_eq!(
        rows, sorted_rows,
        "{context} should remain lexicographically canonical after order-normalization"
    );
}

pub(super) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalized_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let rows =
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows(
            topology_profiles,
        );
    assert_parallel_lane_topology_rows_are_canonically_sorted(
        &rows,
        "topology-id to host-mode-host-pair-lane-set-lane-fingerprint-hash rows",
    );
    rows
}

pub(super) fn project_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> (Vec<String>, String) {
    let rows =
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalized_rows(
            topology_profiles,
        );
    let digest = deterministic_fnv1a64_hex(&rows.join(","));
    (rows, digest)
}

pub(super) fn permute_role_pair_lanes(
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

pub(super) fn run_live_postgres_matrix_repeated_run_projections() -> Option<(
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

pub(super) fn daemon_args_for_live_postgres_profile(
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

pub(super) fn project_live_postgres_load_profiles() -> Vec<LivePostgresLoadProfile> {
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

pub(super) fn project_live_postgres_role_profiles() -> Vec<LivePostgresLoadProfile> {
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

pub(super) fn project_live_postgres_role_pair_profiles() -> Vec<LivePostgresRolePairProfile> {
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

pub(super) fn project_live_postgres_parallel_role_pair_lanes() -> Vec<LivePostgresRolePairProfile> {
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

pub(super) fn project_live_postgres_asymmetric_parallel_lanes() -> Vec<LivePostgresRolePairProfile>
{
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

pub(super) fn project_live_postgres_parallel_lane_topology_profiles(
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

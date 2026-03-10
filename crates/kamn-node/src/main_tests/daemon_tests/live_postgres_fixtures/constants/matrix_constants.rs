pub(crate) const LIVE_POSTGRES_ENV_UNSET_REASON_CODE: &str = "live_postgres_env_unset";
pub(crate) const LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE: &str =
    "live_postgres_adapter_connected";
pub(crate) const LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.daemon.phase6.reason-taxonomy.v1";
pub(crate) const LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1";
pub(crate) const LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_applied";
pub(crate) const LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_deferred";
pub(crate) const LIVE_POSTGRES_RUNTIME_TO_MATRIX_BRIDGE_REASON_CODES_CSV: &str =
    "m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred";
pub(crate) const LIVE_POSTGRES_MATRIX_LOAD_PROFILE_IDS_CSV: &str = "applied_t3_i10,applied_t5_i25,applied_t9_i40,deferred_t5_i25_s3_d2_to4,deferred_t7_i25_s3_d2_to4,deferred_t9_i40_s3_d2_to4";
pub(crate) const LIVE_POSTGRES_MATRIX_ROLE_PROFILE_IDS_CSV: &str = "processor_applied,processor_deferred,listener_applied,listener_deferred,approver_applied,approver_deferred";
pub(crate) const LIVE_POSTGRES_MATRIX_ROLE_PAIR_IDS_CSV: &str = "processor_to_listener_applied,processor_to_listener_deferred,listener_to_approver_applied,listener_to_approver_deferred,approver_to_processor_applied,approver_to_processor_deferred";
pub(crate) const LIVE_POSTGRES_MATRIX_PARALLEL_ROLE_PAIR_LANE_IDS_CSV: &str = "processor_listener_parallel_applied,processor_listener_parallel_deferred,listener_approver_parallel_applied,listener_approver_parallel_deferred";
pub(crate) const LIVE_POSTGRES_MATRIX_ASYMMETRIC_PARALLEL_LANE_IDS_CSV: &str = "processor_listener_asymmetric_parallel_applied,processor_listener_asymmetric_parallel_deferred,listener_approver_asymmetric_parallel_applied,listener_approver_asymmetric_parallel_deferred";
pub(crate) const LIVE_POSTGRES_MATRIX_PERMUTATION_IDS_CSV: &str =
    "baseline,reverse,rotate_left_1,interleaved_even_then_odd";
pub(crate) const LIVE_POSTGRES_MATRIX_ORDER_INVARIANCE_LANE_SETS_CSV: &str =
    "symmetric_parallel,asymmetric_parallel";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-fingerprint.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_ORDER_CSV: &str =
    "lane_id,leg_a_reason,leg_a_taxonomy,leg_b_reason,leg_b_taxonomy";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER: char = '|';
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_COUNT: usize = 5;

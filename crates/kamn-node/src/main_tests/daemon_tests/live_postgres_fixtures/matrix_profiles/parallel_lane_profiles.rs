use super::super::constants::*;
use super::super::models::*;
use super::role_pair_profiles::*;

const PARALLEL_ROLE_PAIR_PROFILE_SPECS: [RolePairProfileSpec; 4] = [
    (
        "processor_listener_parallel_applied",
        "processor_applied",
        ("processor", "5", "25", None),
        "listener_applied",
        ("listener", "5", "25", None),
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "processor_listener_parallel_deferred",
        "processor_deferred",
        ("processor", "5", "25", Some(("3", "2", "4"))),
        "listener_deferred",
        ("listener", "5", "25", Some(("3", "2", "4"))),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
    (
        "listener_approver_parallel_applied",
        "listener_applied",
        ("listener", "5", "25", None),
        "approver_applied",
        ("approver", "5", "25", None),
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "listener_approver_parallel_deferred",
        "listener_deferred",
        ("listener", "5", "25", Some(("3", "2", "4"))),
        "approver_deferred",
        ("approver", "5", "25", Some(("3", "2", "4"))),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
];

const ASYMMETRIC_PARALLEL_LANE_SPECS: [RolePairProfileSpec; 4] = [
    (
        "processor_listener_asymmetric_parallel_applied",
        "processor_applied_t3_i10",
        ("processor", "3", "10", None),
        "listener_applied_t9_i40",
        ("listener", "9", "40", None),
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "processor_listener_asymmetric_parallel_deferred",
        "processor_deferred_t5_i25",
        ("processor", "5", "25", Some(("3", "2", "4"))),
        "listener_deferred_t9_i40",
        ("listener", "9", "40", Some(("3", "2", "4"))),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
    (
        "listener_approver_asymmetric_parallel_applied",
        "listener_applied_t7_i25",
        ("listener", "7", "25", None),
        "approver_applied_t9_i40",
        ("approver", "9", "40", None),
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "listener_approver_asymmetric_parallel_deferred",
        "listener_deferred_t7_i25",
        ("listener", "7", "25", Some(("3", "2", "4"))),
        "approver_deferred_t9_i40",
        ("approver", "9", "40", Some(("3", "2", "4"))),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
];

pub(crate) fn project_live_postgres_parallel_role_pair_lanes() -> Vec<LivePostgresRolePairProfile> {
    role_pair_profiles_from_specs(&PARALLEL_ROLE_PAIR_PROFILE_SPECS)
}

pub(crate) fn project_live_postgres_asymmetric_parallel_lanes() -> Vec<LivePostgresRolePairProfile>
{
    role_pair_profiles_from_specs(&ASYMMETRIC_PARALLEL_LANE_SPECS)
}

pub(crate) fn project_live_postgres_parallel_lane_topology_profiles(
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

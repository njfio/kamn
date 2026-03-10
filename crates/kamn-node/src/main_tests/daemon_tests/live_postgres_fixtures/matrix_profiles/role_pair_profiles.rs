use super::super::constants::*;
use super::super::models::*;
use super::daemon_args::*;

pub(crate) type RoleArgsSpec = (
    &'static str,
    &'static str,
    &'static str,
    Option<ShutdownArgsSpec>,
);
pub(crate) type RolePairProfileSpec = (
    &'static str,
    &'static str,
    RoleArgsSpec,
    &'static str,
    RoleArgsSpec,
    &'static str,
);

const ROLE_PAIR_PROFILE_SPECS: [RolePairProfileSpec; 6] = [
    (
        "processor_to_listener_applied",
        "processor_applied",
        ("processor", "5", "25", None),
        "listener_applied",
        ("listener", "5", "25", None),
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "processor_to_listener_deferred",
        "processor_deferred",
        ("processor", "5", "25", Some(("3", "2", "4"))),
        "listener_deferred",
        ("listener", "5", "25", Some(("3", "2", "4"))),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
    (
        "listener_to_approver_applied",
        "listener_applied",
        ("listener", "5", "25", None),
        "approver_applied",
        ("approver", "5", "25", None),
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "listener_to_approver_deferred",
        "listener_deferred",
        ("listener", "5", "25", Some(("3", "2", "4"))),
        "approver_deferred",
        ("approver", "5", "25", Some(("3", "2", "4"))),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
    (
        "approver_to_processor_applied",
        "approver_applied",
        ("approver", "5", "25", None),
        "processor_applied",
        ("processor", "5", "25", None),
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "approver_to_processor_deferred",
        "approver_deferred",
        ("approver", "5", "25", Some(("3", "2", "4"))),
        "processor_deferred",
        ("processor", "5", "25", Some(("3", "2", "4"))),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
];

fn build_role_args(spec: RoleArgsSpec) -> Vec<String> {
    let (role, ticks, interval, shutdown) = spec;
    daemon_args_for_live_postgres_profile(role, ticks, interval, shutdown)
}

pub(crate) fn role_pair_profile(
    pair_id: &'static str,
    leg_a_profile_id: &'static str,
    leg_a: RoleArgsSpec,
    leg_b_profile_id: &'static str,
    leg_b: RoleArgsSpec,
    expected_reason_code: &'static str,
) -> LivePostgresRolePairProfile {
    LivePostgresRolePairProfile {
        pair_id,
        leg_a_profile_id,
        leg_a_args: build_role_args(leg_a),
        leg_b_profile_id,
        leg_b_args: build_role_args(leg_b),
        expected_reason_code,
    }
}

pub(crate) fn role_pair_profiles_from_specs(
    specs: &[RolePairProfileSpec],
) -> Vec<LivePostgresRolePairProfile> {
    specs
        .iter()
        .copied()
        .map(
            |(pair_id, leg_a_profile_id, leg_a, leg_b_profile_id, leg_b, expected_reason_code)| {
                role_pair_profile(
                    pair_id,
                    leg_a_profile_id,
                    leg_a,
                    leg_b_profile_id,
                    leg_b,
                    expected_reason_code,
                )
            },
        )
        .collect()
}

pub(crate) fn project_live_postgres_role_pair_profiles() -> Vec<LivePostgresRolePairProfile> {
    role_pair_profiles_from_specs(&ROLE_PAIR_PROFILE_SPECS)
}

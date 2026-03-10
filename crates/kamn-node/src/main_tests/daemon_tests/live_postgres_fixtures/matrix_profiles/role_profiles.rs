use super::super::constants::*;
use super::super::models::*;
use super::load_profiles::*;

type RoleProfileSpec = (
    &'static str,
    &'static str,
    Option<(&'static str, &'static str, &'static str)>,
    &'static str,
);

const ROLE_PROFILE_SPECS: [RoleProfileSpec; 6] = [
    (
        "processor_applied",
        "processor",
        None,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "processor_deferred",
        "processor",
        Some(("3", "2", "4")),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
    (
        "listener_applied",
        "listener",
        None,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "listener_deferred",
        "listener",
        Some(("3", "2", "4")),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
    (
        "approver_applied",
        "approver",
        None,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "approver_deferred",
        "approver",
        Some(("3", "2", "4")),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
];

fn build_role_profiles(specs: &[RoleProfileSpec]) -> Vec<LivePostgresLoadProfile> {
    specs
        .iter()
        .copied()
        .map(|(profile_id, role, shutdown, reason)| {
            load_profile(profile_id, role, "5", "25", shutdown, reason)
        })
        .collect()
}

pub(crate) fn project_live_postgres_role_profiles() -> Vec<LivePostgresLoadProfile> {
    build_role_profiles(&ROLE_PROFILE_SPECS)
}

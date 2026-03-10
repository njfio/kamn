use super::super::constants::*;
use super::super::models::*;
use super::daemon_args::*;
type LoadProfileSpec = (
    &'static str,
    &'static str,
    &'static str,
    Option<ShutdownArgsSpec>,
    &'static str,
);

const APPLIED_LOAD_PROFILE_SPECS: [LoadProfileSpec; 3] = [
    (
        "applied_t3_i10",
        "3",
        "10",
        None,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "applied_t5_i25",
        "5",
        "25",
        None,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
    (
        "applied_t9_i40",
        "9",
        "40",
        None,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
    ),
];

const DEFERRED_LOAD_PROFILE_SPECS: [LoadProfileSpec; 3] = [
    (
        "deferred_t5_i25_s3_d2_to4",
        "5",
        "25",
        Some(("3", "2", "4")),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
    (
        "deferred_t7_i25_s3_d2_to4",
        "7",
        "25",
        Some(("3", "2", "4")),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
    (
        "deferred_t9_i40_s3_d2_to4",
        "9",
        "40",
        Some(("3", "2", "4")),
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
    ),
];
pub(crate) fn load_profile(
    profile_id: &'static str,
    role: &'static str,
    max_ticks: &'static str,
    tick_interval_ms: &'static str,
    shutdown: Option<ShutdownArgsSpec>,
    expected_reason_code: &'static str,
) -> LivePostgresLoadProfile {
    LivePostgresLoadProfile {
        profile_id,
        args: daemon_args_for_live_postgres_profile(role, max_ticks, tick_interval_ms, shutdown),
        expected_reason_code,
    }
}

fn build_load_profiles(specs: &[LoadProfileSpec]) -> Vec<LivePostgresLoadProfile> {
    specs
        .iter()
        .copied()
        .map(|(profile_id, max_ticks, interval, shutdown, reason)| {
            load_profile(
                profile_id,
                "processor",
                max_ticks,
                interval,
                shutdown,
                reason,
            )
        })
        .collect()
}

fn applied_load_profiles() -> Vec<LivePostgresLoadProfile> {
    build_load_profiles(&APPLIED_LOAD_PROFILE_SPECS)
}

fn deferred_load_profiles() -> Vec<LivePostgresLoadProfile> {
    build_load_profiles(&DEFERRED_LOAD_PROFILE_SPECS)
}

pub(crate) fn project_live_postgres_load_profiles() -> Vec<LivePostgresLoadProfile> {
    let mut profiles = applied_load_profiles();
    profiles.extend(deferred_load_profiles());
    profiles
}

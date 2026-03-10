fn load_profile_ids_csv(profiles: &[LivePostgresLoadProfile]) -> String {
    profiles
        .iter()
        .map(|profile| profile.profile_id)
        .collect::<Vec<_>>()
        .join(",")
}

fn load_profile_reason_codes(profiles: &[LivePostgresLoadProfile]) -> Vec<&'static str> {
    profiles
        .iter()
        .map(|profile| profile.expected_reason_code)
        .collect::<Vec<_>>()
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical(
) {
    let profiles = project_live_postgres_load_profiles();
    assert_eq!(load_profile_ids_csv(&profiles), LIVE_POSTGRES_MATRIX_LOAD_PROFILE_IDS_CSV);
    assert_eq!(
        load_profile_reason_codes(&profiles),
        vec![
            LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
            LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
            LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
            LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
            LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
            LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        ]
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_load_profile_matrix_is_deterministic(
) {
    let Some(_context) = live_postgres_validation_context() else {
        return;
    };
    for profile in project_live_postgres_load_profiles() {
        assert_profile_projection_stable(
            "profile",
            profile.profile_id,
            profile.args,
            profile.expected_reason_code,
        );
    }
}

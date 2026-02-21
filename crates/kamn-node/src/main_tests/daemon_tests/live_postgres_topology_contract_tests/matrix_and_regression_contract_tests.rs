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

#[test]
fn functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic() {
    let canonical_rows = crate::live_postgres_multi_host_execution_bundle_selector_rows_for_test();
    assert_selector_bundle_accepts_canonical_rows(&canonical_rows);
    assert_selector_bundle_rejects_duplicate_rows(&canonical_rows);
    assert_selector_bundle_rejects_prefix_violation();
    assert_selector_bundle_rejects_row_format_violation();
    assert_selector_bundle_rejects_row_id_violation();
    assert_selector_bundle_rejects_row_count_mismatch(&canonical_rows);
}

#[test]
fn functional_runtime_daemon_projects_phase6_deferred_runtime_markers_when_shutdown_signals_are_present(
) {
    let (rendered, complete_line) = capture_deferred_phase6_json_and_complete_log();
    assert_rendered_contains_all(
        &rendered,
        &[
            "\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_deferred\"",
            "\"daemon_phase6_runtime_deferred_cycles\":1",
        ],
    );
    assert_json_log_fields(
        &complete_line,
        &[
            ("phase6_reason_code", "m10_phase6_scheduler_cycle_deferred"),
            ("phase6_deferred_cycles", "1"),
        ],
    );
}

#[test]
fn regression_runtime_daemon_deferred_phase6_log_selection_uses_execution_id() {
    let target_execution_id = "node-runtime:daemon:phase6-deferred-contract:processor";
    let captured_logs = vec![
        format!(
            "{{\"event\":\"node.runtime.daemon.execute.complete\",\"execution_id\":\"node-runtime:daemon:foreign:processor\",\"phase6_reason_code\":\"m10_phase6_scheduler_cycle_applied\"}}"
        ),
        format!(
            "{{\"event\":\"node.runtime.daemon.execute.complete\",\"execution_id\":\"{target_execution_id}\",\"phase6_reason_code\":\"m10_phase6_scheduler_cycle_deferred\"}}"
        ),
    ];
    let selected = find_daemon_complete_log_for_execution(
        &captured_logs,
        target_execution_id,
    );
    assert_json_log_field(selected, "execution_id", target_execution_id);
    assert_json_log_field(
        selected,
        "phase6_reason_code",
        "m10_phase6_scheduler_cycle_deferred",
    );
}

fn assert_selector_bundle_accepts_canonical_rows(canonical_rows: &[String]) {
    let canonical_fingerprint =
        crate::live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test();
    assert_eq!(
        canonical_fingerprint,
        deterministic_fnv1a64_hex(LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS_CSV)
    );
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(canonical_rows, 6),
        Ok(())
    );
}

fn assert_selector_bundle_rejects_duplicate_rows(canonical_rows: &[String]) {
    let mut duplicate_rows = canonical_rows.to_vec();
    duplicate_rows.push(canonical_rows[0].clone());
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(&duplicate_rows, 7),
        Err("live_postgres_selector_bundle_duplicate_rows")
    );
}

fn assert_selector_bundle_rejects_prefix_violation() {
    let rows = vec![
        "b01_runtime_matrix_bundle->invalid_prefix::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs".to_owned(),
    ];
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(&rows, 1),
        Err("live_postgres_selector_bundle_prefix_violation")
    );
}

fn assert_selector_bundle_rejects_row_format_violation() {
    let rows = vec!["b01_runtime_matrix_bundle".to_owned()];
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(&rows, 1),
        Err("live_postgres_selector_bundle_row_format_violation")
    );
}

fn assert_selector_bundle_rejects_row_id_violation() {
    let rows = vec![
        "b99_unknown_bundle->main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs".to_owned(),
    ];
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(&rows, 1),
        Err("live_postgres_selector_bundle_row_id_violation")
    );
}

fn assert_selector_bundle_rejects_row_count_mismatch(canonical_rows: &[String]) {
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(canonical_rows, 999),
        Err("live_postgres_selector_bundle_row_count_mismatch")
    );
}

fn capture_deferred_phase6_json_and_complete_log() -> (String, String) {
    let _guards = runtime_json_log_guards();
    let (rendered, captured_logs) = capture_daemon_json_and_logs(&[
        "--daemon-max-ticks",
        "5",
        "--daemon-tick-interval-ms",
        "25",
        "--daemon-shutdown-signal-tick",
        "3",
        "--daemon-shutdown-drain-ticks",
        "2",
        "--daemon-shutdown-timeout-ticks",
        "4",
        "--output",
        "json",
    ]);
    let complete_line = find_daemon_complete_log_for_execution(
        &captured_logs,
        "node-runtime:daemon:kamn-devnet:processor",
    );
    (rendered, complete_line.to_owned())
}

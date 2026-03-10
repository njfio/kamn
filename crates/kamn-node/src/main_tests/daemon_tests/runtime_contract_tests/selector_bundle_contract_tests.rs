fn functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic() {
    let canonical_rows = crate::live_postgres_multi_host_execution_bundle_selector_rows_for_test();
    let canonical_fingerprint =
        crate::live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test();
    assert_eq!(
        canonical_fingerprint,
        deterministic_fnv1a64_hex(LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS_CSV)
    );
    assert_eq!(
        canonical_fingerprint,
        crate::live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test()
    );
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(canonical_rows.as_slice(), 6),
        Ok(())
    );

    let mut duplicate_rows = canonical_rows.clone();
    duplicate_rows.push(canonical_rows[0].clone());
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(duplicate_rows.as_slice(), 7),
        Err("live_postgres_selector_bundle_duplicate_rows")
    );

    let prefix_violation_rows = vec![
        "b01_runtime_matrix_bundle->invalid_prefix::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs"
            .to_owned(),
    ];
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(prefix_violation_rows.as_slice(), 1),
        Err("live_postgres_selector_bundle_prefix_violation")
    );

    let row_format_violation_rows = vec!["b01_runtime_matrix_bundle".to_owned()];
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(
            row_format_violation_rows.as_slice(),
            1
        ),
        Err("live_postgres_selector_bundle_row_format_violation")
    );

    let row_id_violation_rows = vec![
        "b99_unknown_bundle->main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs"
            .to_owned(),
    ];
    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(row_id_violation_rows.as_slice(), 1),
        Err("live_postgres_selector_bundle_row_id_violation")
    );

    assert_eq!(
        crate::validate_live_postgres_selector_bundle_for_test(canonical_rows.as_slice(), 999),
        Err("live_postgres_selector_bundle_row_count_mismatch")
    );
}

#[test]
fn functional_runtime_daemon_projects_phase6_deferred_runtime_markers_when_shutdown_signals_are_present(
) {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
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

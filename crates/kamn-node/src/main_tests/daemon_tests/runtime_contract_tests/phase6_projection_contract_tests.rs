fn functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output() {
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
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    let rendered_text = render_bootstrap_report(&report, OutputMode::text());
    let expected_selector_rows_fingerprint =
        crate::live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test();
    assert!(rendered.contains(
        "\"daemon_phase6_runtime_reason_taxonomy_version\":\"kamn.runtime.daemon.phase6.reason-taxonomy.v1\""
    ));
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_applied\""));
    assert!(rendered.contains(
        "\"daemon_convergence_reason_taxonomy_version\":\"kamn.runtime.daemon.convergence.reason-taxonomy.v1\""
    ));
    assert!(rendered.contains("\"daemon_convergence_decision\":\"go\""));
    assert!(
        rendered.contains("\"daemon_convergence_reason_code\":\"convergence_promotion_gate_go\"")
    );
    assert!(rendered.contains(
        "\"daemon_live_postgres_multi_host_execution_bundle_schema_version\":\"kamn.runtime.daemon.phase6-live-postgres.multi-host-execution-bundle.v1\""
    ));
    assert!(rendered.contains(
        "\"daemon_live_postgres_multi_host_execution_bundle_selector_prefix\":\"main_tests::daemon_tests::\""
    ));
    assert!(rendered.contains(
        "\"daemon_live_postgres_multi_host_execution_bundle_row_count\":6"
    ));
    assert!(rendered.contains(
        format!(
            "\"daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint\":\"{expected_selector_rows_fingerprint}\""
        )
        .as_str()
    ));
    assert!(rendered_text.contains(
        "daemon_live_postgres_multi_host_execution_bundle_schema_version: kamn.runtime.daemon.phase6-live-postgres.multi-host-execution-bundle.v1"
    ));
    assert!(rendered_text.contains(
        "daemon_live_postgres_multi_host_execution_bundle_selector_prefix: main_tests::daemon_tests::"
    ));
    assert!(
        rendered_text.contains("daemon_live_postgres_multi_host_execution_bundle_row_count: 6")
    );
    assert!(rendered_text.contains(
        format!(
            "daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint: {expected_selector_rows_fingerprint}"
        )
        .as_str()
    ));
    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "phase6_reason_taxonomy_version").as_deref(),
        Some("kamn.runtime.daemon.phase6.reason-taxonomy.v1")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "phase6_reason_codes_csv").as_deref(),
        Some("m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded")
    );
    assert!(
        extract_json_string_field(complete_line, "phase6_reason_code").as_deref()
            == Some("m10_phase6_scheduler_cycle_applied")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_reason_taxonomy_version").as_deref(),
        Some("kamn.runtime.daemon.convergence.reason-taxonomy.v1")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_decision").as_deref(),
        Some("go")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_reason_code").as_deref(),
        Some("convergence_promotion_gate_go")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "multi_host_execution_bundle_schema_version")
            .as_deref(),
        Some("kamn.runtime.daemon.phase6-live-postgres.multi-host-execution-bundle.v1")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "multi_host_execution_bundle_selector_prefix")
            .as_deref(),
        Some("main_tests::daemon_tests::")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "multi_host_execution_bundle_row_count")
            .as_deref(),
        Some("6")
    );
    let selector_rows_csv = extract_json_string_field(
        complete_line,
        "multi_host_execution_bundle_selector_rows_csv",
    )
    .expect("daemon runtime completion log should include selector rows csv marker");
    assert_eq!(
        selector_rows_csv,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS_CSV
    );
    let selector_rows = selector_rows_csv
        .split(',')
        .map(str::trim)
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(selector_rows.len(), 6);
    assert!(selector_rows
        .iter()
        .all(|row| row.contains(LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX)));
    let selector_rows_fingerprint = extract_json_string_field(
        complete_line,
        "multi_host_execution_bundle_selector_rows_fingerprint",
    )
    .expect("daemon runtime completion log should include selector rows fingerprint marker");
    assert_eq!(
        selector_rows_fingerprint,
        crate::live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test()
    );
}

#[test]

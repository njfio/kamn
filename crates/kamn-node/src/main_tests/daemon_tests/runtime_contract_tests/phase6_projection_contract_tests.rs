#[test]
fn functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output() {
    let (rendered, rendered_text, complete_line) = capture_phase6_renderings_and_complete_log();
    let expected_selector_rows_fingerprint = expected_selector_rows_fingerprint();
    assert_phase6_rendered_markers(&rendered);
    assert_phase6_bundle_render_markers(&rendered, &expected_selector_rows_fingerprint);
    assert_phase6_text_render_markers(&rendered_text, &expected_selector_rows_fingerprint);
    assert_phase6_complete_log_fields(&complete_line);
    assert_phase6_selector_rows(&complete_line);
}

fn expected_selector_rows_fingerprint() -> String {
    crate::live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test()
}

fn assert_phase6_rendered_markers(rendered: &str) {
    assert_rendered_contains_all(
        rendered,
        &[
            "\"daemon_phase6_runtime_reason_taxonomy_version\":\"kamn.runtime.daemon.phase6.reason-taxonomy.v1\"",
            "\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_applied\"",
            "\"daemon_convergence_reason_taxonomy_version\":\"kamn.runtime.daemon.convergence.reason-taxonomy.v1\"",
            "\"daemon_convergence_decision\":\"go\"",
            "\"daemon_convergence_reason_code\":\"convergence_promotion_gate_go\"",
        ],
    );
}

fn assert_phase6_bundle_render_markers(rendered: &str, fingerprint: &str) {
    assert_rendered_contains_all(
        rendered,
        &[
            "\"daemon_live_postgres_multi_host_execution_bundle_schema_version\":\"kamn.runtime.daemon.phase6-live-postgres.multi-host-execution-bundle.v1\"",
            "\"daemon_live_postgres_multi_host_execution_bundle_selector_prefix\":\"main_tests::daemon_tests::\"",
            "\"daemon_live_postgres_multi_host_execution_bundle_row_count\":6",
        ],
    );
    assert!(rendered.contains(
        format!(
            "\"daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint\":\"{fingerprint}\""
        )
        .as_str()
    ));
}

fn assert_phase6_text_render_markers(rendered_text: &str, fingerprint: &str) {
    assert_rendered_contains_all(
        rendered_text,
        &[
            "daemon_live_postgres_multi_host_execution_bundle_schema_version: kamn.runtime.daemon.phase6-live-postgres.multi-host-execution-bundle.v1",
            "daemon_live_postgres_multi_host_execution_bundle_selector_prefix: main_tests::daemon_tests::",
            "daemon_live_postgres_multi_host_execution_bundle_row_count: 6",
        ],
    );
    assert!(rendered_text.contains(
        format!(
            "daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint: {fingerprint}"
        )
        .as_str()
    ));
}

fn assert_phase6_complete_log_fields(complete_line: &str) {
    assert_json_log_fields(
        complete_line,
        &[
            ("phase6_reason_taxonomy_version", "kamn.runtime.daemon.phase6.reason-taxonomy.v1"),
            ("phase6_reason_codes_csv", "m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded"),
            ("phase6_reason_code", "m10_phase6_scheduler_cycle_applied"),
            ("convergence_reason_taxonomy_version", "kamn.runtime.daemon.convergence.reason-taxonomy.v1"),
            ("convergence_decision", "go"),
            ("convergence_reason_code", "convergence_promotion_gate_go"),
            ("multi_host_execution_bundle_schema_version", "kamn.runtime.daemon.phase6-live-postgres.multi-host-execution-bundle.v1"),
            ("multi_host_execution_bundle_selector_prefix", "main_tests::daemon_tests::"),
            ("multi_host_execution_bundle_row_count", "6"),
        ],
    );
}

fn assert_phase6_selector_rows(complete_line: &str) {
    let selector_rows_csv = selector_rows_csv(complete_line);
    assert_eq!(
        selector_rows_csv,
        LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS_CSV
    );
    assert_selector_rows_shape(&selector_rows_csv);
    assert_selector_rows_fingerprint(complete_line);
}

fn selector_rows_csv(complete_line: &str) -> String {
    extract_json_string_field(
        complete_line,
        "multi_host_execution_bundle_selector_rows_csv",
    )
    .expect("daemon runtime completion log should include selector rows csv marker")
}

fn assert_selector_rows_shape(selector_rows_csv: &str) {
    let selector_rows = selector_rows_csv
        .split(',')
        .map(str::trim)
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(selector_rows.len(), 6);
    assert!(selector_rows
        .iter()
        .all(|row| row.contains(LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX)));
}

fn assert_selector_rows_fingerprint(complete_line: &str) {
    let selector_rows_fingerprint = extract_json_string_field(
        complete_line,
        "multi_host_execution_bundle_selector_rows_fingerprint",
    )
    .expect("daemon runtime completion log should include selector rows fingerprint marker");
    assert_eq!(
        selector_rows_fingerprint,
        expected_selector_rows_fingerprint()
    );
}

fn capture_phase6_renderings_and_complete_log() -> (String, String, String) {
    let _guards = runtime_json_log_guards();
    let (rendered_json, rendered_text, captured_logs) = capture_daemon_renderings_and_logs(&[
        "--daemon-max-ticks",
        "5",
        "--daemon-tick-interval-ms",
        "25",
        "--output",
        "json",
    ]);
    let complete_line = find_first_daemon_log(
        &captured_logs,
        "\"event\":\"node.runtime.daemon.execute.complete\"",
    );
    (rendered_json, rendered_text, complete_line.to_owned())
}

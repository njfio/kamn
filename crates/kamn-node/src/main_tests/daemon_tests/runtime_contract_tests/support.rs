struct RuntimeJsonLogGuards {
    _lock: std::sync::MutexGuard<'static, ()>,
    _level_guard: EnvVarGuard,
    _format_guard: EnvVarGuard,
}

fn runtime_json_log_guards() -> RuntimeJsonLogGuards {
    let lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    RuntimeJsonLogGuards {
        _lock: lock,
        _level_guard: level_guard,
        _format_guard: format_guard,
    }
}

fn daemon_args(extra_args: &[&str]) -> Vec<String> {
    let mut args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
    ];
    args.extend(extra_args.iter().map(|arg| (*arg).to_owned()));
    args
}

fn daemon_args_with_chain(chain_id: &str, extra_args: &[&str]) -> Vec<String> {
    let mut args = daemon_args(&["--chain-id", chain_id]);
    args.extend(extra_args.iter().map(|arg| (*arg).to_owned()));
    args
}

fn parse_daemon(extra_args: &[&str]) -> crate::NodeCli {
    parse_args_with_clean_daemon_env(daemon_args(extra_args)).expect("daemon args should parse")
}

fn parse_daemon_with_chain(chain_id: &str, extra_args: &[&str]) -> crate::NodeCli {
    parse_args_with_clean_daemon_env(daemon_args_with_chain(chain_id, extra_args))
        .expect("daemon args should parse")
}

fn execute_daemon_json(extra_args: &[&str]) -> String {
    let report = execute(parse_daemon(extra_args)).expect("daemon execution should succeed");
    render_bootstrap_report(&report, OutputMode::json())
}

fn capture_daemon_json_with_chain(
    chain_id: &str,
    extra_args: &[&str],
) -> (String, Vec<String>, String) {
    let parsed = parse_daemon_with_chain(chain_id, extra_args);
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    let execution_id = format!("node-runtime:daemon:{chain_id}:processor");
    (rendered, captured_logs, execution_id)
}

fn capture_daemon_renderings_and_logs(extra_args: &[&str]) -> (String, String, Vec<String>) {
    let parsed = parse_daemon(extra_args);
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    let rendered_json = render_bootstrap_report(&report, OutputMode::json());
    let rendered_text = render_bootstrap_report(&report, OutputMode::text());
    (rendered_json, rendered_text, captured_logs)
}

fn find_daemon_log<'a>(captured_logs: &'a [String], event: &str, execution_id: &str) -> &'a str {
    captured_logs
        .iter()
        .find(|line| {
            line.contains(event)
                && extract_json_string_field(line, "execution_id").as_deref() == Some(execution_id)
        })
        .map(|line| line.as_str())
        .expect("daemon execution should emit structured log marker")
}

fn find_first_daemon_log<'a>(captured_logs: &'a [String], event: &str) -> &'a str {
    captured_logs
        .iter()
        .find(|line| line.contains(event))
        .map(|line| line.as_str())
        .expect("daemon execution should emit structured log marker")
}

fn find_daemon_complete_log_for_execution<'a>(
    captured_logs: &'a [String],
    execution_id: &str,
) -> &'a str {
    find_daemon_log(
        captured_logs,
        "\"event\":\"node.runtime.daemon.execute.complete\"",
        execution_id,
    )
}

fn assert_json_log_field(log_line: &str, field: &str, expected: &str) {
    assert_eq!(
        extract_json_string_field(log_line, field).as_deref(),
        Some(expected)
    );
}

fn assert_json_log_fields(log_line: &str, fields: &[(&str, &str)]) {
    for (field, expected) in fields {
        assert_json_log_field(log_line, field, expected);
    }
}

fn assert_rendered_contains_all(rendered: &str, markers: &[&str]) {
    for marker in markers {
        assert!(
            rendered.contains(marker),
            "rendered output missing marker: {marker}"
        );
    }
}

use super::*;
use crate::daemon_test_env_lock;
#[cfg(unix)]
use crate::{configure_os_signal_test_triggers, OsSignalTestKind, OsSignalTestTrigger};

mod live_postgres_fixtures;
use live_postgres_fixtures::*;

#[test]
fn functional_runtime_daemon_emits_structured_transition_markers() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "99".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
    ])
    .expect("daemon args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");

    let start_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.start\""))
        .expect("daemon execution should emit structured start marker");
    assert_eq!(
        extract_json_string_field(start_line, "runtime_mode").as_deref(),
        Some("daemon")
    );
    assert_eq!(
        extract_json_string_field(start_line, "max_ticks").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(start_line, "tick_interval_ms").as_deref(),
        Some("25")
    );
    let start_execution_id = extract_json_string_field(start_line, "execution_id")
        .expect("daemon start marker should include execution_id");

    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "runtime_mode").as_deref(),
        Some("daemon")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "executed_ticks").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("tick-budget-exhausted;ignored_signals=1")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("not-signaled")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-not-requested")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("none")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("1")
    );
    let complete_execution_id = extract_json_string_field(complete_line, "execution_id")
        .expect("daemon completion marker should include execution_id");
    assert_eq!(start_execution_id, complete_execution_id);
}

#[test]
fn functional_runtime_daemon_graceful_shutdown_emits_structured_drain_markers() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ])
    .expect("daemon args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");

    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("completed")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("2")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("4")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-flushed")
    );
}

#[test]
pub(super) fn regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "7".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "4".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "2".to_owned(),
    ])
    .expect("daemon timeout args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon timeout execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_convergence_decision\":\"no_go\""));
    assert!(rendered.contains(
        "\"daemon_convergence_reason_code\":\"convergence_performance_budget_exceeded\""
    ));

    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("timeout")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("7")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("4")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("2")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-flush-timeout")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_decision").as_deref(),
        Some("no_go")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_reason_code").as_deref(),
        Some("convergence_performance_budget_exceeded")
    );
}

#[test]
fn parses_runtime_mode_daemon_with_bounded_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "99".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "start-connect".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::daemon());
    assert_eq!(parsed.daemon_max_ticks, Some(3));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_peer_id, Some("peer-alpha".to_owned()));
    assert_eq!(parsed.daemon_lifecycle_events.len(), 2);
}

#[test]
fn parses_runtime_mode_daemon_with_shutdown_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "8".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with shutdown controls should parse");
    assert_eq!(parsed.daemon_shutdown_signal_ticks, vec![3]);
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_daemon_with_os_signal_shutdown_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "12".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--daemon-shutdown-os-signals".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with os signal controls should parse");
    assert_eq!(parsed.daemon_shutdown_signal_ticks, Vec::<u64>::new());
    assert!(parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_daemon_with_observability_endpoint_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "12".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--observability-endpoint-bind".to_owned(),
        "127.0.0.1:9108".to_owned(),
        "--observability-endpoint-metrics-path".to_owned(),
        "/runtime/metrics".to_owned(),
        "--observability-endpoint-health-path".to_owned(),
        "/runtime/health".to_owned(),
        "--observability-endpoint-max-requests".to_owned(),
        "3".to_owned(),
        "--observability-endpoint-idle-timeout-ms".to_owned(),
        "1200".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with observability endpoint should parse");
    assert_eq!(
        parsed.observability_endpoint_bind_addr,
        Some("127.0.0.1:9108".to_owned())
    );
    assert_eq!(
        parsed.observability_endpoint_metrics_path,
        "/runtime/metrics"
    );
    assert_eq!(parsed.observability_endpoint_health_path, "/runtime/health");
    assert_eq!(parsed.observability_endpoint_max_requests, 3);
    assert_eq!(parsed.observability_endpoint_idle_timeout_ms, 1200);
}

#[test]
fn env_only_daemon_controls_parse_without_config_file() {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .expect("daemon env lock should guard process-level overrides");
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", Some("12"));
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", Some("25"));

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
    ];

    let parsed = parse_args(args).expect("env-only daemon controls should parse");
    assert_eq!(parsed.daemon_max_ticks, Some(12));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
}

#[test]
fn regression_3202_invalid_daemon_env_override_fails_closed_without_config_file() {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .expect("daemon env lock should guard process-level overrides");
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", Some("invalid"));
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", Some("25"));

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
    ];

    let parse_result = parse_args(args);
    assert!(
        matches!(
            parse_result,
            Err(ConfigError::InvalidDaemonControlArgument(value)) if value == "invalid"
        ),
        "invalid daemon env override must fail closed with typed daemon control error"
    );
}

#[test]
fn integration_runtime_daemon_renders_bounded_completion_output() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "99".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "start-connect".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "heartbeat-missed".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "heartbeat-restored".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"daemon\""));
    assert!(rendered.contains("\"daemon_max_ticks\":3"));
    assert!(rendered.contains("\"daemon_tick_interval_ms\":25"));
    assert!(rendered.contains("\"daemon_executed_ticks\":3"));
    assert!(rendered
        .contains("\"daemon_completion_reason\":\"tick-budget-exhausted;ignored_signals=1\""));
    assert!(rendered.contains("\"daemon_observability_latency_p50_ms\":25"));
    assert!(rendered.contains("\"daemon_observability_latency_p99_ms\":50"));
    assert!(rendered.contains("\"daemon_observability_throughput_tps\":2000"));
    assert!(rendered.contains("\"daemon_observability_error_rate_bps\":50"));
    assert!(rendered.contains("\"daemon_observability_availability_bps\":9990"));
    assert!(rendered.contains("\"daemon_observability_health\":\"healthy\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":0"));
    assert!(rendered.contains("\"daemon_observability_reason_code\":\"none\""));
    assert!(rendered.contains("\"daemon_observability_transport_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_signer_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_commit_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_peer_id\":\"peer-alpha\""));
    assert!(rendered.contains("\"daemon_peer_lifecycle_final_state\":\"active\""));
    assert!(
        rendered.contains(
            "\"daemon_peer_lifecycle_applied_events\":[\"start-connect\",\"handshake-succeeded\",\"heartbeat-missed\",\"heartbeat-restored\"]"
        )
    );
}

#[test]
fn functional_runtime_daemon_applies_graceful_shutdown_signal() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
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

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon shutdown args should parse");
    let report = execute(parsed).expect("daemon graceful shutdown execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":5"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=0\""
    ));
    assert!(rendered.contains("\"daemon_observability_health\":\"healthy\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":0"));
}

#[test]
fn integration_runtime_daemon_shutdown_timeout_is_fail_closed() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "7".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "4".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "2".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon timeout args should parse");
    let report = execute(parsed).expect("daemon timeout execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":9"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0\""
    ));
    assert!(rendered.contains("\"daemon_observability_latency_p50_ms\":145"));
    assert!(rendered.contains("\"daemon_observability_latency_p99_ms\":425"));
    assert!(rendered.contains("\"daemon_observability_throughput_tps\":900"));
    assert!(rendered.contains("\"daemon_observability_error_rate_bps\":250"));
    assert!(rendered.contains("\"daemon_observability_availability_bps\":9800"));
    assert!(rendered.contains("\"daemon_observability_health\":\"critical\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":4"));
    assert!(rendered.contains("\"daemon_observability_reason_code\":\"daemon_shutdown_timeout\""));
    assert!(rendered.contains("\"daemon_observability_transport_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_signer_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_commit_checkpoint_failures\":1"));
}

#[cfg(unix)]
#[test]
pub(super) fn integration_runtime_daemon_applies_graceful_shutdown_on_os_signal() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "100".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-os-signals".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "5".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed =
        parse_args_with_clean_daemon_env(args).expect("daemon os-signal args should parse");
    configure_os_signal_test_triggers(vec![OsSignalTestTrigger::new(5, OsSignalTestKind::Sigterm)]);
    let report = execute(parsed).expect("daemon os-signal execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_completion_reason\":\"graceful-shutdown:signal@"));
}

#[test]
fn functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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
}

#[test]
fn functional_runtime_daemon_projects_phase6_deferred_runtime_markers_when_shutdown_signals_are_present(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
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

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

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
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains(
        "\"daemon_phase6_runtime_reason_taxonomy_version\":\"kamn.runtime.daemon.phase6.reason-taxonomy.v1\""
    ));
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_applied\""));
}

#[test]
fn regression_runtime_daemon_live_postgres_validation_slice_reports_unset_env_gate_reason() {
    // Regression: #5340
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _test_postgres_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", None);
    let _database_guard = EnvVarGuard::set("DATABASE_URL", None);
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
    assert!(maybe_database_url.is_none());
}

#[test]
fn unit_runtime_daemon_live_postgres_validation_slice_prefers_kamn_test_postgres_url() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let preferred = "postgres://preferred:5432/kamn_test";
    let fallback = "postgres://fallback:5432/kamn_test";
    let _test_postgres_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some(preferred));
    let _database_guard = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );
    assert_eq!(maybe_database_url.as_deref(), Some(preferred));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_deferred_path() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

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
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered
        .contains("\"daemon_phase6_runtime_reason_code\":\"m10_phase6_scheduler_cycle_deferred\""));
    assert!(rendered.contains("\"daemon_phase6_runtime_deferred_cycles\":1"));
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_env_matrix_contract_is_deterministic() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");

    let _unset_primary = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", None);
    let _unset_fallback = EnvVarGuard::set("DATABASE_URL", None);
    let (reason_unset, url_unset) = resolve_live_postgres_gate_decision();
    assert_eq!(reason_unset, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
    assert!(url_unset.is_none());
    drop(_unset_fallback);
    drop(_unset_primary);

    let preferred = "postgres://preferred:5432/kamn_test";
    let fallback = "postgres://fallback:5432/kamn_test";
    let _preferred_guard = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some(preferred));
    let _fallback_guard = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (reason_preferred, url_preferred) = resolve_live_postgres_gate_decision();
    assert_eq!(
        reason_preferred,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );
    assert_eq!(url_preferred.as_deref(), Some(preferred));
    drop(_fallback_guard);
    drop(_preferred_guard);

    let _blank_primary = EnvVarGuard::set("KAMN_TEST_POSTGRES_URL", Some("   "));
    let _fallback_only = EnvVarGuard::set("DATABASE_URL", Some(fallback));
    let (reason_fallback, url_fallback) = resolve_live_postgres_gate_decision();
    assert_eq!(reason_fallback, LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE);
    assert_eq!(url_fallback.as_deref(), Some(fallback));
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical(
) {
    let rows = project_live_postgres_matrix_rows();
    assert_eq!(
        rows,
        vec![
            LivePostgresMatrixRow {
                scenario_id: "env_unset",
                gate_reason_code: LIVE_POSTGRES_ENV_UNSET_REASON_CODE,
                daemon_phase6_reason_code: None,
            },
            LivePostgresMatrixRow {
                scenario_id: "env_set_no_shutdown",
                gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
                daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE),
            },
            LivePostgresMatrixRow {
                scenario_id: "env_set_shutdown",
                gate_reason_code: LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
                daemon_phase6_reason_code: Some(LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE),
            },
        ],
        "matrix projection rows must remain canonical and ordered"
    );
    let scenario_csv = rows
        .iter()
        .map(|row| row.scenario_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(scenario_csv, LIVE_POSTGRES_MATRIX_SCENARIOS_CSV);

    let reason_codes_csv = format!(
        "{},{},{}",
        LIVE_POSTGRES_ENV_UNSET_REASON_CODE,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(reason_codes_csv, LIVE_POSTGRES_MATRIX_REASON_CODES_CSV);
    assert_eq!(
        LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    );
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_matrix_taxonomy_bridge_contract_is_canonical(
) {
    let rows = project_live_postgres_matrix_rows();
    let bridge_reason_codes_csv = rows
        .iter()
        .filter_map(|row| row.daemon_phase6_reason_code)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_MATRIX_REASON_TAXONOMY_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    );
    assert_eq!(
        bridge_reason_codes_csv,
        LIVE_POSTGRES_RUNTIME_TO_MATRIX_BRIDGE_REASON_CODES_CSV
    );
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical(
) {
    let profiles = project_live_postgres_load_profiles();
    let profile_ids_csv = profiles
        .iter()
        .map(|profile| profile.profile_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(profile_ids_csv, LIVE_POSTGRES_MATRIX_LOAD_PROFILE_IDS_CSV);

    assert!(profiles
        .iter()
        .take(3)
        .all(|profile| profile.expected_reason_code
            == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(profiles
        .iter()
        .skip(3)
        .all(|profile| profile.expected_reason_code
            == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_load_profile_matrix_is_deterministic(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

    for profile in project_live_postgres_load_profiles() {
        let first = run_daemon_for_phase6_projection(profile.args.clone());
        let second = run_daemon_for_phase6_projection(profile.args);
        assert_eq!(
            first.reason_code, profile.expected_reason_code,
            "profile {} should project expected phase6 reason code",
            profile.profile_id
        );
        assert_eq!(
            first.reason_code, second.reason_code,
            "profile {} reason code should remain stable across repeated runs",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "profile {} should remain bridged to runtime taxonomy version",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, second.reason_taxonomy_version,
            "profile {} taxonomy version should remain stable across repeated runs",
            profile.profile_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_role_profile_matrix_contract_is_canonical(
) {
    let profiles = project_live_postgres_role_profiles();
    let profile_ids_csv = profiles
        .iter()
        .map(|profile| profile.profile_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(profile_ids_csv, LIVE_POSTGRES_MATRIX_ROLE_PROFILE_IDS_CSV);

    assert_eq!(
        profiles[0].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[1].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(
        profiles[2].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[3].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
    assert_eq!(
        profiles[4].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE
    );
    assert_eq!(
        profiles[5].expected_reason_code,
        LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_role_profile_matrix_is_deterministic(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

    for profile in project_live_postgres_role_profiles() {
        let first = run_daemon_for_phase6_projection(profile.args.clone());
        let second = run_daemon_for_phase6_projection(profile.args);
        assert_eq!(
            first.reason_code, profile.expected_reason_code,
            "role profile {} should project expected phase6 reason code",
            profile.profile_id
        );
        assert_eq!(
            first.reason_code, second.reason_code,
            "role profile {} reason code should remain stable across repeated runs",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "role profile {} should remain bridged to runtime taxonomy version",
            profile.profile_id
        );
        assert_eq!(
            first.reason_taxonomy_version, second.reason_taxonomy_version,
            "role profile {} taxonomy version should remain stable across repeated runs",
            profile.profile_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_role_pair_matrix_contract_is_canonical()
{
    let pairs = project_live_postgres_role_pair_profiles();
    let pair_ids_csv = pairs
        .iter()
        .map(|pair| pair.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(pair_ids_csv, LIVE_POSTGRES_MATRIX_ROLE_PAIR_IDS_CSV);
    assert!(pairs
        .iter()
        .step_by(2)
        .all(|pair| pair.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(pairs
        .iter()
        .skip(1)
        .step_by(2)
        .all(|pair| pair.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_role_pair_matrix_is_deterministic(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

    for pair in project_live_postgres_role_pair_profiles() {
        let leg_a_first = run_daemon_for_phase6_projection(pair.leg_a_args.clone());
        let leg_a_second = run_daemon_for_phase6_projection(pair.leg_a_args);
        let leg_b_first = run_daemon_for_phase6_projection(pair.leg_b_args.clone());
        let leg_b_second = run_daemon_for_phase6_projection(pair.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, pair.expected_reason_code,
            "pair {} leg A ({}) should project expected phase6 reason code",
            pair.pair_id, pair.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, pair.expected_reason_code,
            "pair {} leg B ({}) should project expected phase6 reason code",
            pair.pair_id, pair.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "pair {} leg A reason code should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "pair {} leg B reason code should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "pair {} leg A taxonomy should stay on runtime taxonomy version",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "pair {} leg B taxonomy should stay on runtime taxonomy version",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "pair {} leg A taxonomy should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "pair {} leg B taxonomy should remain stable across repeated runs",
            pair.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "pair {} legs should share the same runtime taxonomy version",
            pair.pair_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_role_pair_lane_contract_is_canonical(
) {
    let lanes = project_live_postgres_parallel_role_pair_lanes();
    let lane_ids_csv = lanes
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        lane_ids_csv,
        LIVE_POSTGRES_MATRIX_PARALLEL_ROLE_PAIR_LANE_IDS_CSV
    );
    assert!(lanes
        .iter()
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(lanes
        .iter()
        .skip(1)
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_role_pair_lane_is_deterministic(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

    for lane in project_live_postgres_parallel_role_pair_lanes() {
        let (leg_a_first, leg_b_first) =
            run_parallel_phase6_projections(lane.leg_a_args.clone(), lane.leg_b_args.clone());
        let (leg_a_second, leg_b_second) =
            run_parallel_phase6_projections(lane.leg_a_args, lane.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, lane.expected_reason_code,
            "parallel lane {} leg A ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, lane.expected_reason_code,
            "parallel lane {} leg B ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "parallel lane {} leg A reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "parallel lane {} leg B reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "parallel lane {} leg A taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "parallel lane {} leg B taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "parallel lane {} leg A taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "parallel lane {} leg B taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "parallel lane {} legs should share the same runtime taxonomy version",
            lane.pair_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_asymmetric_parallel_lane_contract_is_canonical(
) {
    let lanes = project_live_postgres_asymmetric_parallel_lanes();
    let lane_ids_csv = lanes
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        lane_ids_csv,
        LIVE_POSTGRES_MATRIX_ASYMMETRIC_PARALLEL_LANE_IDS_CSV
    );
    assert!(lanes
        .iter()
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE));
    assert!(lanes
        .iter()
        .skip(1)
        .step_by(2)
        .all(|lane| lane.expected_reason_code == LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE));
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_asymmetric_parallel_lane_is_deterministic(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

    for lane in project_live_postgres_asymmetric_parallel_lanes() {
        let (leg_a_first, leg_b_first) =
            run_parallel_phase6_projections(lane.leg_a_args.clone(), lane.leg_b_args.clone());
        let (leg_a_second, leg_b_second) =
            run_parallel_phase6_projections(lane.leg_a_args, lane.leg_b_args);

        assert_eq!(
            leg_a_first.reason_code, lane.expected_reason_code,
            "asymmetric lane {} leg A ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_a_profile_id
        );
        assert_eq!(
            leg_b_first.reason_code, lane.expected_reason_code,
            "asymmetric lane {} leg B ({}) should project expected phase6 reason code",
            lane.pair_id, lane.leg_b_profile_id
        );
        assert_eq!(
            leg_a_first.reason_code, leg_a_second.reason_code,
            "asymmetric lane {} leg A reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_code, leg_b_second.reason_code,
            "asymmetric lane {} leg B reason code should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "asymmetric lane {} leg A taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
            "asymmetric lane {} leg B taxonomy should stay on runtime taxonomy version",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_a_second.reason_taxonomy_version,
            "asymmetric lane {} leg A taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_b_first.reason_taxonomy_version, leg_b_second.reason_taxonomy_version,
            "asymmetric lane {} leg B taxonomy should remain stable across repeated runs",
            lane.pair_id
        );
        assert_eq!(
            leg_a_first.reason_taxonomy_version, leg_b_first.reason_taxonomy_version,
            "asymmetric lane {} legs should share the same runtime taxonomy version",
            lane.pair_id
        );
    }
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_order_invariance_contract_is_canonical(
) {
    let lane_sets_csv = ["symmetric_parallel", "asymmetric_parallel"].join(",");
    assert_eq!(
        lane_sets_csv,
        LIVE_POSTGRES_MATRIX_ORDER_INVARIANCE_LANE_SETS_CSV
    );

    let mut expected_symmetric_fingerprints = project_live_postgres_parallel_role_pair_lanes()
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    expected_symmetric_fingerprints.sort();

    let mut expected_asymmetric_fingerprints = project_live_postgres_asymmetric_parallel_lanes()
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    expected_asymmetric_fingerprints.sort();

    assert_eq!(
        expected_symmetric_fingerprints,
        vec![
            "listener_approver_parallel_applied",
            "listener_approver_parallel_deferred",
            "processor_listener_parallel_applied",
            "processor_listener_parallel_deferred"
        ]
    );
    assert_eq!(
        expected_asymmetric_fingerprints,
        vec![
            "listener_approver_asymmetric_parallel_applied",
            "listener_approver_asymmetric_parallel_deferred",
            "processor_listener_asymmetric_parallel_applied",
            "processor_listener_asymmetric_parallel_deferred"
        ]
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_order_is_invariant(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

    let symmetric_baseline =
        run_parallel_lane_set_fingerprints(project_live_postgres_parallel_role_pair_lanes());
    let mut symmetric_permuted_lanes = project_live_postgres_parallel_role_pair_lanes();
    symmetric_permuted_lanes.reverse();
    let symmetric_permuted = run_parallel_lane_set_fingerprints(symmetric_permuted_lanes);
    assert_eq!(
        symmetric_baseline, symmetric_permuted,
        "symmetric parallel lane fingerprints should remain invariant to lane order"
    );

    let asymmetric_baseline =
        run_parallel_lane_set_fingerprints(project_live_postgres_asymmetric_parallel_lanes());
    let mut asymmetric_permuted_lanes = project_live_postgres_asymmetric_parallel_lanes();
    asymmetric_permuted_lanes.reverse();
    let asymmetric_permuted = run_parallel_lane_set_fingerprints(asymmetric_permuted_lanes);
    assert_eq!(
        asymmetric_baseline, asymmetric_permuted,
        "asymmetric parallel lane fingerprints should remain invariant to lane order"
    );
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_permutation_contract_is_canonical(
) {
    assert_eq!(
        LIVE_POSTGRES_MATRIX_PERMUTATION_IDS_CSV,
        [
            "baseline",
            "reverse",
            "rotate_left_1",
            "interleaved_even_then_odd"
        ]
        .join(",")
    );
    let symmetric_lanes = project_live_postgres_parallel_role_pair_lanes();
    let symmetric_base_ids = symmetric_lanes
        .iter()
        .map(|lane| lane.pair_id)
        .collect::<Vec<_>>();
    assert_eq!(
        symmetric_base_ids,
        vec![
            "processor_listener_parallel_applied",
            "processor_listener_parallel_deferred",
            "listener_approver_parallel_applied",
            "listener_approver_parallel_deferred"
        ]
    );
    let symmetric_reverse_ids =
        permute_role_pair_lanes(project_live_postgres_parallel_role_pair_lanes(), "reverse")
            .iter()
            .map(|lane| lane.pair_id)
            .collect::<Vec<_>>();
    assert_eq!(
        symmetric_reverse_ids,
        vec![
            "listener_approver_parallel_deferred",
            "listener_approver_parallel_applied",
            "processor_listener_parallel_deferred",
            "processor_listener_parallel_applied"
        ]
    );
    let symmetric_rotate_ids = permute_role_pair_lanes(
        project_live_postgres_parallel_role_pair_lanes(),
        "rotate_left_1",
    )
    .iter()
    .map(|lane| lane.pair_id)
    .collect::<Vec<_>>();
    assert_eq!(
        symmetric_rotate_ids,
        vec![
            "processor_listener_parallel_deferred",
            "listener_approver_parallel_applied",
            "listener_approver_parallel_deferred",
            "processor_listener_parallel_applied"
        ]
    );
    let symmetric_interleaved_ids = permute_role_pair_lanes(
        project_live_postgres_parallel_role_pair_lanes(),
        "interleaved_even_then_odd",
    )
    .iter()
    .map(|lane| lane.pair_id)
    .collect::<Vec<_>>();
    assert_eq!(
        symmetric_interleaved_ids,
        vec![
            "processor_listener_parallel_applied",
            "listener_approver_parallel_applied",
            "processor_listener_parallel_deferred",
            "listener_approver_parallel_deferred"
        ]
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant(
) {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    let Some(database_url) = maybe_database_url else {
        assert_eq!(gate_reason_code, LIVE_POSTGRES_ENV_UNSET_REASON_CODE);
        return;
    };
    assert_eq!(
        gate_reason_code,
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible for live postgres validation");
    runtime.block_on(async move {
        let adapter = kamn_core::DataLayerPgExecutionAdapter::connect(
            kamn_core::DataLayerPgExecutionAdapterConfig {
                database_url,
                max_connections: 4,
            },
        )
        .await
        .expect("live postgres connection should succeed when test URL is provided");
        adapter
            .apply_migrations()
            .await
            .expect("live postgres migrations should apply for validation slice");
    });

    let permutation_ids = [
        "baseline",
        "reverse",
        "rotate_left_1",
        "interleaved_even_then_odd",
    ];

    let symmetric_baseline = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
        project_live_postgres_parallel_role_pair_lanes(),
        permutation_ids[0],
    ));
    for permutation in permutation_ids.iter().skip(1) {
        let permuted = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
            project_live_postgres_parallel_role_pair_lanes(),
            permutation,
        ));
        assert_eq!(
            symmetric_baseline, permuted,
            "symmetric parallel lane fingerprints should remain invariant to permutation {permutation}"
        );
    }

    let asymmetric_baseline = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
        project_live_postgres_asymmetric_parallel_lanes(),
        permutation_ids[0],
    ));
    for permutation in permutation_ids.iter().skip(1) {
        let permuted = run_parallel_lane_set_fingerprints(permute_role_pair_lanes(
            project_live_postgres_asymmetric_parallel_lanes(),
            permutation,
        ));
        assert_eq!(
            asymmetric_baseline, permuted,
            "asymmetric parallel lane fingerprints should remain invariant to permutation {permutation}"
        );
    }
}

// daemon_tests structural budget shell phase2; route topology contracts through src/main_tests/daemon_tests/*.rs includes
include!("daemon_tests/live_postgres_topology_contract_tests.rs");

use super::{cli, DiagnosticsMode, OutputMode, RuntimeMode};
use kamn_core::{NodeRole, SyncMode};

fn is_test_cfg_attribute(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("#[cfg(") && trimmed.contains("test") && !trimmed.contains("not(test)")
}

fn skip_cfg_test_item(lines: &[&str], mut index: usize) -> usize {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }

    while index < lines.len() && lines[index].trim_start().starts_with("#[") {
        index += 1;
    }

    if index >= lines.len() {
        return index;
    }

    let mut brace_depth = 0_i64;
    let mut saw_open_brace = false;
    while index < lines.len() {
        let line = lines[index];
        let open_count = line.matches('{').count() as i64;
        let close_count = line.matches('}').count() as i64;
        if open_count > 0 {
            saw_open_brace = true;
        }
        brace_depth += open_count - close_count;
        index += 1;

        if saw_open_brace {
            if brace_depth <= 0 {
                return index;
            }
            continue;
        }

        if line.trim_end().ends_with(';') {
            return index;
        }
    }

    index
}

fn production_source_without_cfg_test_items(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut retained = Vec::with_capacity(lines.len());
    let mut index = 0usize;
    while index < lines.len() {
        if is_test_cfg_attribute(lines[index]) {
            index = skip_cfg_test_item(&lines, index + 1);
            continue;
        }
        retained.push(lines[index]);
        index += 1;
    }
    retained.join("\n")
}

#[test]
fn cli_module_parses_required_role_and_defaults() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ];

    let parsed = cli::parse_args(args).expect("args should parse");
    assert_eq!(parsed.role, NodeRole::Processor);
    assert_eq!(parsed.chain_id, "kamn-devnet");
    assert_eq!(parsed.chain_version, "v0.1.0");
    assert_eq!(parsed.storage_dir, "./data");
    assert!(parsed.enable_gossip);
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
    assert_eq!(parsed.runtime_mode, RuntimeMode::bootstrap());
    assert!(parsed.daemon_shutdown_signal_ticks.is_empty());
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, None);
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, None);
    assert_eq!(parsed.api_bind_addr, None);
    assert_eq!(parsed.api_max_requests, 1);
    assert_eq!(parsed.api_idle_timeout_ms, 5_000);
    assert_eq!(parsed.api_body_limit_bytes, 64 * 1024);
    assert_eq!(parsed.api_concurrency_limit, 32);
    assert_eq!(parsed.api_rate_limit_per_second, 120);
    assert_eq!(parsed.observability_endpoint_bind_addr, None);
    assert_eq!(parsed.observability_endpoint_metrics_path, "/metrics");
    assert_eq!(parsed.observability_endpoint_health_path, "/healthz");
    assert_eq!(parsed.observability_endpoint_max_requests, 1);
    assert_eq!(parsed.observability_endpoint_idle_timeout_ms, 5_000);
    assert_eq!(parsed.output_mode, OutputMode::text());
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::basic());
}

#[test]
fn cli_module_runtime_mode_api_requires_bind_address() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
    ];

    let err = cli::parse_args(args).expect_err("api runtime without bind should fail");
    assert_eq!(
        err,
        kamn_core::ConfigError::MissingArgumentValue("--api-bind")
    );
}

#[test]
fn regression_cli_module_api_limiter_overrides_require_bind_address() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--api-body-limit-bytes".to_owned(),
        "1024".to_owned(),
    ];

    let err = cli::parse_args(args).expect_err("api limiter override without bind should fail");
    assert_eq!(
        err,
        kamn_core::ConfigError::MissingArgumentValue("--api-bind")
    );
}

#[test]
fn cli_module_parses_api_runtime_endpoint_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34051".to_owned(),
        "--api-max-requests".to_owned(),
        "4".to_owned(),
        "--api-idle-timeout-ms".to_owned(),
        "2000".to_owned(),
        "--api-body-limit-bytes".to_owned(),
        "131072".to_owned(),
        "--api-concurrency-limit".to_owned(),
        "16".to_owned(),
        "--api-rate-limit-per-second".to_owned(),
        "240".to_owned(),
    ];

    let parsed = cli::parse_args(args).expect("api args should parse");
    assert_eq!(
        parsed.runtime_mode,
        RuntimeMode::parse("api").expect("mode")
    );
    assert_eq!(parsed.api_bind_addr.as_deref(), Some("127.0.0.1:34051"));
    assert_eq!(parsed.api_max_requests, 4);
    assert_eq!(parsed.api_idle_timeout_ms, 2_000);
    assert_eq!(parsed.api_body_limit_bytes, 131_072);
    assert_eq!(parsed.api_concurrency_limit, 16);
    assert_eq!(parsed.api_rate_limit_per_second, 240);
}

#[test]
fn regression_2745_cli_kolme_live_branch_has_no_expect_panics() {
    let cli_source = include_str!("cli.rs");
    assert!(
        !cli_source.contains("expect(\"provider hint is required for kolme-live mode\")"),
        "kolme-live provider-hint guard must remain fallible and panic-free"
    );
    assert!(
        !cli_source.contains("expect(\"signing profile is required for kolme-live mode\")"),
        "kolme-live signing-profile guard must remain fallible and panic-free"
    );
}

#[test]
fn regression_3940_production_source_extractor_retains_non_test_items() {
    // Regression: #3940
    let source = r#"
#[cfg(test)]
use std::sync::Mutex;

fn production_path() {
    let _value = std::env::var("KAMN_REGRESSION_EXAMPLE").expect("typed path should be visible");
}
"#;
    let production_source = production_source_without_cfg_test_items(source);
    assert!(
        production_source.contains("expect("),
        "production extractor must retain non-test items after top-level cfg(test) imports"
    );
}

#[test]
fn regression_3598_startup_paths_have_no_panic_control_flow() {
    // Regression: #3598
    for (name, source) in [
        ("cli.rs", include_str!("cli.rs")),
        ("main.rs", include_str!("main.rs")),
        (
            "runtime_kolme_live.rs",
            include_str!("runtime_kolme_live.rs"),
        ),
        ("signer.rs", include_str!("signer.rs")),
        ("daemon_shutdown.rs", include_str!("daemon_shutdown.rs")),
        (
            "service_api_endpoint.rs",
            include_str!("service_api_endpoint.rs"),
        ),
        (
            "observability_endpoint.rs",
            include_str!("observability_endpoint.rs"),
        ),
        (
            "daemon_observability.rs",
            include_str!("daemon_observability.rs"),
        ),
        (
            "kolme_live_observability.rs",
            include_str!("kolme_live_observability.rs"),
        ),
    ] {
        let production_source = production_source_without_cfg_test_items(source);
        assert!(
            !production_source.contains("expect("),
            "startup production paths must not use expect(): {name}"
        );
        assert!(
            !production_source.contains("unreachable!("),
            "startup production paths must not use unreachable!(): {name}"
        );
        assert!(
            !production_source.contains("panic!("),
            "startup production paths must not use panic!(): {name}"
        );
    }
}

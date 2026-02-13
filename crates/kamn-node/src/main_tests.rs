use super::{
    build_bootstrap_report, build_kolme_live_direct_signed_wire_payload,
    build_kolme_live_managed_signing_key, build_kolme_live_request,
    build_kolme_live_signer_adapter, capture_test_logs, encode_kolme_hex_lower, execute,
    parse_args, render_bootstrap_report, render_kolme_live_native_direct_message,
    render_log_event_line, resolve_kolme_live_managed_signer_required_marker,
    resolve_kolme_live_nonce, resolve_kolme_live_signer_private_key_env_name,
    resolve_log_config_from_inputs, sign_kolme_live_managed_external_message, DiagnosticsMode,
    LocalProfile, NodeBootstrapReport, NodeLogConfig, NodeLogFormat, NodeLogLevel, OutputMode,
    RuntimeExecutionBundle, RuntimeMode,
};
use kamn_core::{
    bootstrap, ConfigError, KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitRequest, NodeConfig,
    NodeRole, SignerProviderHandshakeMatrix, SyncMode,
};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::{env, sync::OnceLock};

const TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
const TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY: &str =
    "838c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
const TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE: &str =
    "secure:aws-kms:role-operator/key-live-ops-primary";
const TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY: &str =
    "secure:aws-kms:role-operator/key-live-ops-secondary";

fn signer_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn log_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn managed_signer_public_key_hex(key_reference: &str) -> String {
    let signing_key = build_kolme_live_managed_signing_key(key_reference)
        .expect("managed signing key should derive");
    encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    )
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            env::set_var(self.key, previous);
        } else {
            env::remove_var(self.key);
        }
    }
}

#[derive(Clone)]
struct MockHttpReply {
    status_line: &'static str,
    body: String,
}

impl MockHttpReply {
    fn ok(body: &str) -> Self {
        Self {
            status_line: "HTTP/1.1 200 OK",
            body: body.to_owned(),
        }
    }
}

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end = None;
    let mut expected_total = None;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be set");

    loop {
        let read_count = match stream.read(&mut chunk) {
            Ok(read_count) => read_count,
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("request bytes should be readable: {error}"),
        };
        if read_count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read_count]);

        if header_end.is_none() {
            header_end = buffer
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|pos| pos + 4);
            if let Some(end) = header_end {
                let headers = String::from_utf8_lossy(&buffer[..end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        if name.eq_ignore_ascii_case("Content-Length") {
                            return value.trim().parse::<usize>().ok();
                        }
                        None
                    })
                    .unwrap_or(0);
                expected_total = Some(end + content_length);
            }
        }
        if let Some(total) = expected_total {
            if buffer.len() >= total {
                break;
            }
        }
    }

    String::from_utf8(buffer).expect("request should be valid utf-8")
}

fn request_body(raw_request: &str) -> &str {
    raw_request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

fn extract_json_string_field(body: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\":\"");
    let start = body.find(marker.as_str())?;
    let remainder = &body[start + marker.len()..];
    let end = remainder.find('"')?;
    Some(remainder[..end].to_owned())
}

fn spawn_kolme_live_mock_server(replies: Vec<MockHttpReply>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    listener
        .set_nonblocking(true)
        .expect("listener should allow nonblocking accepts");
    let addr = listener.local_addr().expect("local addr should resolve");
    let recorded_requests = Arc::new(Mutex::new(Vec::new()));
    let recorded_requests_ref = Arc::clone(&recorded_requests);
    thread::spawn(move || {
        for reply in replies {
            let accept_deadline = Instant::now() + Duration::from_secs(2);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_http_request(&mut stream);
                        recorded_requests_ref
                            .lock()
                            .expect("request mutex should lock")
                            .push(request);
                        let response = format!(
                            "{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            reply.status_line,
                            reply.body.len(),
                            reply.body
                        );
                        stream
                            .write_all(response.as_bytes())
                            .expect("response should write");
                        break;
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        if Instant::now() >= accept_deadline {
                            return;
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept should succeed: {error}"),
                }
            }
        }
    });
    (format!("http://{addr}"), recorded_requests)
}

#[test]
fn parses_required_role_and_defaults() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.profile, None);
    assert_eq!(parsed.role, NodeRole::Processor);
    assert_eq!(parsed.chain_id, "kamn-devnet");
    assert_eq!(parsed.chain_version, "v0.1.0");
    assert_eq!(parsed.storage_dir, "./data");
    assert!(parsed.enable_gossip);
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
    assert_eq!(parsed.runtime_mode, RuntimeMode::bootstrap());
    assert_eq!(parsed.expected_state_hash, None);
    assert_eq!(parsed.expected_state_version, None);
    assert!(parsed.proposals.is_empty());
    assert!(parsed.rejoin_attempts.is_empty());
    assert_eq!(parsed.daemon_max_ticks, None);
    assert_eq!(parsed.daemon_tick_interval_ms, None);
    assert_eq!(parsed.daemon_peer_id, None);
    assert!(parsed.daemon_lifecycle_events.is_empty());
    assert_eq!(parsed.kolme_live_base_url, None);
    assert_eq!(parsed.kolme_live_provider_hint, None);
    assert_eq!(parsed.kolme_live_signing_profile, None);
    assert_eq!(parsed.output_mode, OutputMode::text());
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::basic());
}

#[test]
fn parses_disable_gossip_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "listener".to_owned(),
        "--disable-gossip".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.role, NodeRole::Listener);
    assert!(!parsed.enable_gossip);
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
}

#[test]
fn parses_sync_mode_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--sync-mode".to_owned(),
        "archive".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.sync_mode, SyncMode::Archive);
}

#[test]
fn parses_output_mode_json_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.output_mode, OutputMode::json());
}

#[test]
fn parses_diagnostics_snapshot_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--diagnostics".to_owned(),
        "snapshot".to_owned(),
    ];

    let parsed = parse_args(args).expect("diagnostics args should parse");
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::snapshot());
}

#[test]
fn unit_log_config_parses_level_and_format_inputs() {
    let config = resolve_log_config_from_inputs(Some("debug"), Some("json"))
        .expect("log config inputs should parse");
    assert_eq!(
        config,
        NodeLogConfig {
            level: NodeLogLevel::Debug,
            format: NodeLogFormat::Json,
        }
    );
}

#[test]
fn unit_log_renderer_renders_json_event_fields() {
    let line = render_log_event_line(
        NodeLogConfig {
            level: NodeLogLevel::Info,
            format: NodeLogFormat::Json,
        },
        NodeLogLevel::Info,
        "kolme.live.submit.start",
        &[
            ("correlation_id", "runtime-commit:abc"),
            ("provider_hint", "local"),
        ],
    );
    assert!(line.contains("\"level\":\"INFO\""));
    assert!(line.contains("\"event\":\"kolme.live.submit.start\""));
    assert!(line.contains("\"correlation_id\":\"runtime-commit:abc\""));
    assert!(line.contains("\"provider_hint\":\"local\""));
}

#[test]
fn integration_bootstrap_runtime_emits_structured_marker() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ])
    .expect("args should parse");
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("bootstrap execution should succeed");
    assert_eq!(report.runtime_mode, "bootstrap");
    assert!(
        captured_logs
            .iter()
            .any(|line| line.contains("\"event\":\"node.runtime.bootstrap.plan.ready\"")),
        "bootstrap runtime should emit structured bootstrap marker"
    );
}

#[test]
fn functional_kolme_live_submit_and_finality_logs_keep_correlation_id() {
    let _signer_lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _log_lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ])
    .expect("kolme-live args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("kolme-live execution should succeed");
    assert_eq!(report.runtime_mode, "kolme-live");

    let required_events = [
        "kolme.live.submit.start",
        "kolme.live.submit.outcome",
        "kolme.live.finality.poll.start",
        "kolme.live.finality.poll.outcome",
        "kolme.live.execution.complete",
    ];

    let mut correlation_id = None;
    for event_name in required_events {
        let matching_line = captured_logs
            .iter()
            .find(|line| line.contains(format!("\"event\":\"{event_name}\"").as_str()))
            .expect("required structured event should be present");
        let observed = extract_json_string_field(matching_line, "correlation_id")
            .expect("structured event should include correlation id");
        if let Some(expected) = correlation_id.as_deref() {
            assert_eq!(observed, expected);
        } else {
            assert!(!observed.is_empty(), "correlation id must not be empty");
            correlation_id = Some(observed);
        }
    }
}

#[test]
fn regression_invalid_log_level_config_fails_closed() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("invalid-level"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ])
    .expect("args should parse");
    let error = execute(parsed).expect_err("invalid log level should fail closed");
    assert!(
        matches!(error, ConfigError::InvalidLogConfig(message) if message.contains("KAMN_NODE_LOG_LEVEL")),
        "invalid log level should produce InvalidLogConfig"
    );
}

#[test]
fn performance_structured_logging_rendering_stays_bounded() {
    let started = Instant::now();
    for _ in 0..5_000 {
        let line = render_log_event_line(
            NodeLogConfig {
                level: NodeLogLevel::Info,
                format: NodeLogFormat::Json,
            },
            NodeLogLevel::Info,
            "kolme.live.submit.outcome",
            &[
                ("correlation_id", "runtime-commit:benchmark"),
                ("commit_id", "kolme-commit:benchmark"),
                ("finality", "pending"),
            ],
        );
        assert!(
            line.contains("\"event\":\"kolme.live.submit.outcome\""),
            "rendered line should contain expected event marker"
        );
    }
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "structured log rendering baseline exceeded 1s bound for 5k iterations"
    );
}

#[test]
fn parses_runtime_mode_planning_with_proposals() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-2|did:kamn:agent:bbb|2|state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("planning args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::planning());
    assert_eq!(parsed.expected_state_hash, Some("state-1".to_owned()));
    assert_eq!(parsed.proposals.len(), 2);
}

#[test]
fn parses_runtime_mode_recovery_check_with_rejoin_attempt() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::recovery_check());
    assert_eq!(parsed.expected_state_version, Some(42));
    assert_eq!(parsed.expected_state_hash, Some("state-42".to_owned()));
    assert_eq!(parsed.rejoin_attempts.len(), 1);
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
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "start-connect".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
    ];

    let parsed = parse_args(args).expect("daemon args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::daemon());
    assert_eq!(parsed.daemon_max_ticks, Some(3));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
    assert_eq!(parsed.daemon_peer_id, Some("peer-alpha".to_owned()));
    assert_eq!(parsed.daemon_lifecycle_events.len(), 2);
}

#[test]
fn parses_runtime_mode_kolme_live_with_required_flags() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    assert_eq!(parsed.runtime_mode.as_str(), "kolme-live");
    assert_eq!(
        parsed.kolme_live_base_url,
        Some("http://127.0.0.1:3000".to_owned())
    );
    assert_eq!(
        parsed.kolme_live_provider_hint,
        Some("kolme-fork-local".to_owned())
    );
    assert_eq!(
        parsed.kolme_live_signing_profile,
        Some("kolme-fork-secp256k1-v1".to_owned())
    );
    assert!(!parsed.kolme_live_strict_signer_contracts);
    assert_eq!(parsed.kolme_live_signer_profile, None);
    assert_eq!(
        parsed.kolme_live_signer_key_source,
        Some("env-local".to_owned())
    );
}

#[test]
fn parses_local_listener_profile_defaults() {
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-listener".to_owned(),
    ];

    let parsed = parse_args(args).expect("profile args should parse");
    assert_eq!(parsed.profile, Some(LocalProfile::Listener));
    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-localnet");
    assert_eq!(parsed.storage_dir, "./data/listener");
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
    assert!(parsed.enable_gossip);
}

#[test]
fn profile_defaults_can_be_overridden_by_explicit_flags() {
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-listener".to_owned(),
        "--chain-id".to_owned(),
        "kamn-custom".to_owned(),
        "--storage-dir".to_owned(),
        "./tmp/custom-listener".to_owned(),
        "--disable-gossip".to_owned(),
        "--sync-mode".to_owned(),
        "archive".to_owned(),
    ];

    let parsed = parse_args(args).expect("profile args with overrides should parse");
    assert_eq!(parsed.profile, Some(LocalProfile::Listener));
    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-custom");
    assert_eq!(parsed.storage_dir, "./tmp/custom-listener");
    assert_eq!(parsed.sync_mode, SyncMode::Archive);
    assert!(!parsed.enable_gossip);
}

#[test]
fn functional_json_render_is_deterministic() {
    let report = NodeBootstrapReport {
        runtime_mode: "bootstrap".to_owned(),
        diagnostics_mode: "basic".to_owned(),
        component_count: 2,
        planning_expected_state_hash: None,
        planning_candidate_count: None,
        planning_scheduled_candidate_ids: None,
        recovery_expected_state_version: None,
        recovery_expected_state_hash: None,
        recovery_attempt_count: None,
        recovery_decisions: None,
        daemon_max_ticks: None,
        daemon_tick_interval_ms: None,
        daemon_executed_ticks: None,
        daemon_completion_reason: None,
        daemon_peer_id: None,
        daemon_peer_lifecycle_final_state: None,
        daemon_peer_lifecycle_applied_events: None,
        kolme_live_provider_client_contract: None,
        kolme_live_base_url: None,
        kolme_live_provider_hint: None,
        kolme_live_signing_profile: None,
        kolme_live_signer_profile_selector_env: None,
        kolme_live_signer_profile: None,
        kolme_live_signer_key_source: None,
        kolme_live_signer_private_key_env: None,
        kolme_live_execution_status: None,
        profile: None,
        role: "processor".to_owned(),
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        storage_dir: "./data".to_owned(),
        gossip_enabled: true,
        sync_mode: "fast".to_owned(),
        sync_startup: "StateSyncToLatest".to_owned(),
        sync_recovery: "ResumeRecentState".to_owned(),
        state_version: 1,
        pending_migrations: 0,
        components: vec!["processor".to_owned(), "listener".to_owned()],
    };

    let first = render_bootstrap_report(&report, OutputMode::json());
    let second = render_bootstrap_report(&report, OutputMode::json());
    assert_eq!(first, second, "json output should be deterministic");
    assert!(first.contains("\"role\":\"processor\""));
    assert!(first.contains("\"components\":[\"processor\",\"listener\"]"));
}

#[test]
fn integration_parse_bootstrap_and_render_json() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("args should parse");
    let config = NodeConfig {
        chain_id: parsed.chain_id,
        chain_version: parsed.chain_version,
        role: parsed.role,
        storage_dir: parsed.storage_dir,
        enable_gossip: parsed.enable_gossip,
        sync_mode: parsed.sync_mode,
    };
    let plan = bootstrap(config).expect("bootstrap should succeed");
    let report = build_bootstrap_report(
        &plan,
        parsed.profile,
        parsed.diagnostics_mode,
        RuntimeMode::bootstrap(),
        RuntimeExecutionBundle::default(),
    );
    let rendered = render_bootstrap_report(&report, parsed.output_mode);

    assert!(rendered.contains("\"diagnostics_mode\":\"basic\""));
    assert!(rendered.contains("\"profile\":null"));
    assert!(rendered.contains("\"role\":\"processor\""));
    assert!(rendered.contains("\"chain_id\":\"kamn-devnet\""));
    assert!(rendered.contains("\"sync_mode\":\"fast\""));
    assert!(rendered.contains("\"components\":["));
}

#[test]
fn integration_profile_bootstrap_and_render_json() {
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-listener".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("profile args should parse");
    let config = NodeConfig {
        chain_id: parsed.chain_id,
        chain_version: parsed.chain_version,
        role: parsed.role,
        storage_dir: parsed.storage_dir,
        enable_gossip: parsed.enable_gossip,
        sync_mode: parsed.sync_mode,
    };
    let plan = bootstrap(config).expect("bootstrap should succeed");
    let report = build_bootstrap_report(
        &plan,
        parsed.profile,
        parsed.diagnostics_mode,
        RuntimeMode::bootstrap(),
        RuntimeExecutionBundle::default(),
    );
    let rendered = render_bootstrap_report(&report, parsed.output_mode);

    assert!(rendered.contains("\"diagnostics_mode\":\"basic\""));
    assert!(rendered.contains("\"profile\":\"local-listener\""));
    assert!(rendered.contains("\"role\":\"listener\""));
    assert!(rendered.contains("\"chain_id\":\"kamn-localnet\""));
    assert!(rendered.contains("\"storage_dir\":\"./data/listener\""));
}

#[test]
fn integration_diagnostics_snapshot_includes_component_count() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
        "--diagnostics".to_owned(),
        "snapshot".to_owned(),
    ];
    let parsed = parse_args(args).expect("diagnostics args should parse");
    let config = NodeConfig {
        chain_id: parsed.chain_id,
        chain_version: parsed.chain_version,
        role: parsed.role,
        storage_dir: parsed.storage_dir,
        enable_gossip: parsed.enable_gossip,
        sync_mode: parsed.sync_mode,
    };
    let plan = bootstrap(config).expect("bootstrap should succeed");
    let report = build_bootstrap_report(
        &plan,
        parsed.profile,
        parsed.diagnostics_mode,
        RuntimeMode::bootstrap(),
        RuntimeExecutionBundle::default(),
    );
    let rendered = render_bootstrap_report(&report, parsed.output_mode);

    assert!(rendered.contains("\"diagnostics_mode\":\"snapshot\""));
    assert!(rendered.contains("\"component_count\":"));
}

#[test]
fn integration_runtime_planning_renders_sorted_candidate_ids() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
        "--proposal".to_owned(),
        "tx-2|did:kamn:agent:bbb|2|state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("planning args should parse");
    let report = execute(parsed).expect("planning execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"planning\""));
    assert!(rendered.contains("\"planning_candidate_count\":2"));
    assert!(rendered.contains("\"planning_scheduled_candidate_ids\":[\"tx-1\",\"tx-2\"]"));
}

#[test]
fn integration_runtime_recovery_check_renders_deterministic_decision_output() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|40|state-40|resume-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("recovery-check args should parse");
    let report = execute(parsed).expect("recovery-check execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"recovery-check\""));
    assert!(rendered.contains("\"recovery_expected_state_version\":42"));
    assert!(rendered.contains("\"recovery_expected_state_hash\":\"state-42\""));
    assert!(rendered.contains("\"recovery_attempt_count\":1"));
    assert!(rendered.contains("\"recovery_decisions\":[\"catch-up-required:40->42\"]"));
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

    let parsed = parse_args(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"daemon\""));
    assert!(rendered.contains("\"daemon_max_ticks\":3"));
    assert!(rendered.contains("\"daemon_tick_interval_ms\":25"));
    assert!(rendered.contains("\"daemon_executed_ticks\":3"));
    assert!(rendered.contains("\"daemon_completion_reason\":\"tick-budget-exhausted\""));
    assert!(rendered.contains("\"daemon_peer_id\":\"peer-alpha\""));
    assert!(rendered.contains("\"daemon_peer_lifecycle_final_state\":\"active\""));
    assert!(
        rendered.contains(
            "\"daemon_peer_lifecycle_applied_events\":[\"start-connect\",\"handshake-succeeded\",\"heartbeat-missed\",\"heartbeat-restored\"]"
        )
    );
}

#[test]
fn integration_runtime_kolme_live_renders_provider_contract_markers() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    let report = execute(parsed).expect("kolme-live execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"kolme-live\""));
    assert!(rendered
        .contains("\"kolme_live_provider_client_contract\":\"KolmeRuntimeCommitLiveProvider\""));
    assert!(rendered.contains("\"kolme_live_signing_profile\":\"kolme-fork-secp256k1-v1\""));
    assert!(rendered
        .contains("\"kolme_live_signer_profile_selector_env\":\"KAMN_KOLME_LIVE_SIGNER_PROFILE\""));
    assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-primary\""));
    assert!(rendered.contains("\"kolme_live_signer_key_source\":\"env-local\""));
    assert!(rendered.contains(
        "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX\""
    ));
    assert!(rendered.contains("\"kolme_live_execution_status\":\"submitted;"));

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        3,
        "live runtime should issue nonce, submit, and finality requests"
    );
    assert!(recorded_requests[0].contains("GET /get-next-nonce?pubkey="));
    assert!(recorded_requests[1].contains("PUT /broadcast HTTP/1.1"));
    assert!(recorded_requests[1].contains("X-Idempotency-Key: "));
    let signature =
        extract_json_string_field(request_body(recorded_requests[1].as_str()), "signature")
            .expect("submit request should contain signature JSON field");
    // Regression: #2197
    assert!(
        signature.len() == 128
            && signature
                .as_bytes()
                .iter()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "live runtime submit must not fall back to synthetic idempotency-key signatures"
    );
    assert!(recorded_requests[2]
        .contains("GET /runtime-commit/status?commit_id=kolme-commit%3Aab12cd34 HTTP/1.1"));
}

#[test]
fn unit_kolme_live_signer_builds_direct_signed_wire_payload() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2197",
        "state:node-live-2197",
        "kamn:did:agent:node-live-2197",
        1,
        "payload:node-live-2197",
    )
    .expect("request should build");

    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":22,"account_id":"acct-2197"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        None,
    )
    .expect("signed payload should be produced");

    assert_eq!(signer_selection.profile, "ops-primary");
    assert_eq!(
        signer_selection.private_key_env,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
    );
    assert_eq!(signer_selection.key_source, "env-local");
    assert!(signed_wire_payload.contains("\"message\":\"{\\\"pubkey\\\":"));
    let signature = extract_json_string_field(signed_wire_payload.as_str(), "signature")
        .expect("direct signed payload must include signature field");
    assert_eq!(
        signature.len(),
        128,
        "secp256k1 signature must be 64 bytes hex"
    );
    assert!(
        signature
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "signature must be lowercase hex"
    );
}

#[test]
fn unit_kolme_live_signer_adapter_signs_and_verifies_runtime_message() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let message = "{\"pubkey\":\"pk-adapter\",\"nonce\":7,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}";
    let (adapter, selection) =
        build_kolme_live_signer_adapter(None, None).expect("adapter should build");
    assert_eq!(selection.profile, "ops-primary");
    let (signature_hex, recovery_id) = adapter
        .sign_message(message)
        .expect("adapter signing should succeed");
    assert_eq!(signature_hex.len(), 128);
    assert!(
        signature_hex
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "adapter signature must be lowercase hex"
    );
    adapter
        .verify_message(message, signature_hex.as_str(), recovery_id)
        .expect("adapter signature verification should succeed");
}

#[test]
fn regression_kolme_live_signer_adapter_rejects_malformed_signature_hex() {
    // Regression: #2297
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (adapter, _selection) =
        build_kolme_live_signer_adapter(None, None).expect("adapter should build");
    assert!(
        matches!(
            adapter.verify_message(
                "{\"pubkey\":\"pk-adapter\",\"nonce\":7,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}",
                "zz",
                0,
            ),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("runtime_commit_signature_hex contains invalid hex character")
        ),
        "malformed signature hex must fail closed in adapter verification"
    );
}

#[test]
fn regression_kolme_live_signer_adapter_rejects_recovered_key_mismatch() {
    // Regression: #2297
    let primary = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    )
    .expect("primary adapter should build");
    let secondary = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
    )
    .expect("secondary adapter should build");
    let message = "{\"pubkey\":\"pk-adapter\",\"nonce\":9,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}";
    let (signature_hex, recovery_id) = primary
        .sign_message(message)
        .expect("primary adapter signature should succeed");
    assert!(
        matches!(
            secondary.verify_message(message, signature_hex.as_str(), recovery_id),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("recovered public key does not match signer selection")
        ),
        "signature verification must fail closed when recovered key mismatches signer adapter key"
    );
}

#[test]
#[ignore]
fn integration_kolme_live_signer_vector_probe_contract() {
    let private_key_hex = env::var("KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX")
        .expect("KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX must be set");
    let message = env::var("KAMN_KOLME_SIGNATURE_VECTOR_MESSAGE")
        .expect("KAMN_KOLME_SIGNATURE_VECTOR_MESSAGE must be set");

    let adapter = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        private_key_hex.as_str(),
        "KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX",
    )
    .expect("signature parity adapter should build");
    let (signature_hex, recovery_id) = adapter
        .sign_message(message.as_str())
        .expect("signature parity adapter signing should succeed");
    let pubkey_hex = adapter.public_key_compressed_hex();

    println!("signature_hex={signature_hex}");
    println!("recovery_id={recovery_id}");
    println!("pubkey_hex={pubkey_hex}");

    if let Ok(expected_signature_hex) =
        env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_SIGNATURE_HEX")
    {
        assert_eq!(
            signature_hex, expected_signature_hex,
            "signature parity probe must match expected signature vector"
        );
    }
    if let Ok(expected_recovery_id) = env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_RECOVERY_ID") {
        let expected_recovery_id = expected_recovery_id
            .parse::<u8>()
            .expect("expected recovery id must parse as u8");
        assert_eq!(
            recovery_id, expected_recovery_id,
            "signature parity probe must match expected recovery id vector"
        );
    }
    if let Ok(expected_pubkey_hex) = env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_PUBKEY_HEX") {
        assert_eq!(
            pubkey_hex, expected_pubkey_hex,
            "signature parity probe must match expected pubkey vector"
        );
    }
}

#[test]
fn unit_kolme_live_signer_profile_defaults_to_primary_key_env() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", None);

    let (profile, env_name) = resolve_kolme_live_signer_private_key_env_name(None)
        .expect("default profile selection should succeed");
    assert_eq!(profile, "ops-primary");
    assert_eq!(env_name, "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX");
}

#[test]
fn regression_kolme_live_signer_profile_rejects_unsupported_value() {
    // Regression: #2222
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("legacy"));
    assert!(
        matches!(
            resolve_kolme_live_signer_private_key_env_name(None),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("KAMN_KOLME_LIVE_SIGNER_PROFILE has unsupported profile")
        ),
        "unsupported signer profile must fail closed"
    );
}

#[test]
fn integration_kolme_live_signer_profile_secondary_uses_secondary_key_env() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _secondary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2222",
        "state:node-live-2222",
        "kamn:did:agent:node-live-2222",
        1,
        "payload:node-live-2222",
    )
    .expect("request should build");

    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":31,"account_id":"acct-2222"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        None,
    )
    .expect("secondary profile signing should succeed");
    assert_eq!(signer_selection.profile, "ops-secondary");
    assert_eq!(
        signer_selection.private_key_env,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
    );
    assert_eq!(signer_selection.key_source, "env-local");
    let signature = extract_json_string_field(signed_wire_payload.as_str(), "signature")
        .expect("direct signed payload must include signature field");
    assert_eq!(signature.len(), 128);
}

#[test]
fn integration_runtime_kolme_live_renders_secondary_signer_selection_markers() {
    // Regression: #2241
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _secondary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":37,"account_id":"acct-live-secondary"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ef56ab78","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    let report = execute(parsed).expect("kolme-live execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-secondary\""));
    assert!(rendered.contains(
        "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY\""
    ));
    assert!(rendered.contains("\"kolme_live_signer_key_source\":\"env-local\""));
}

#[test]
fn integration_runtime_kolme_live_renders_managed_external_signer_selection_markers() {
    // Regression: #2323
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let request = build_kolme_live_request(
        &bootstrap(NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "./data".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        })
        .expect("bootstrap plan should build"),
    )
    .expect("runtime commit request should build");
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let _managed_signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_pubkey.as_str()),
    );
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 43)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        managed_pubkey,
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":43,"account_id":"acct-live-managed"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:aa11bb22","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "managed-external".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    let report = execute(parsed).expect("kolme-live execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-primary\""));
    assert!(rendered.contains("\"kolme_live_signer_key_source\":\"managed-external\""));
    assert!(rendered.contains(
        "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX\""
    ));
}

#[test]
fn unit_kolme_live_native_direct_message_contains_required_fields() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2207",
        "state:node-live-2207",
        "kamn:did:agent:node-live-2207",
        1,
        "payload:node-live-2207",
    )
    .expect("request should build");

    let message = render_kolme_live_native_direct_message(
        &request,
        "02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344",
        19,
    )
    .expect("native direct message should render");

    assert!(message.contains(
        "\"pubkey\":\"02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344\""
    ));
    assert!(message.contains("\"nonce\":19"));
    assert!(message.contains("\"created\":\""));
    assert!(message.contains("\"messages\":["));
}

#[test]
fn integration_kolme_live_nonce_resolver_fetches_next_nonce() {
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":27,"account_id":"acct-2207"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let signer_adapter = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    )
    .expect("deterministic signer adapter should build");
    let pubkey = signer_adapter.public_key_compressed_hex();

    let nonce = resolve_kolme_live_nonce(base_url.as_str(), &mut transport, pubkey.as_str())
        .expect("nonce should resolve");
    assert_eq!(nonce, 27);

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(recorded_requests.len(), 1);
    assert!(recorded_requests[0].contains("GET /get-next-nonce?pubkey="));
}

#[test]
fn regression_kolme_live_nonce_resolver_rejects_malformed_response() {
    // Regression: #2207
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":0,"account_id":"acct-2207"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = resolve_kolme_live_nonce(
        base_url.as_str(),
        &mut transport,
        "02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344",
    )
    .expect_err("invalid nonce payload must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("nonce response malformed")),
        "expected fail-closed nonce parser error"
    );
}

#[test]
fn regression_kolme_live_signer_requires_primary_key_env_value() {
    // Regression: #2222
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    assert!(
        matches!(
            build_kolme_live_signer_adapter(None, None),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX must be set")
        ),
        "missing primary signer private key env must fail closed"
    );
}

#[test]
fn regression_issue_2279_kolme_live_signer_rejects_fallback_private_key_env_path() {
    // Regression: #2279
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _fallback_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("env-local")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("fallback_signer_secret_present_violation")
        ),
        "fallback signer private key env path must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_requires_key_reference_env_marker() {
    // Regression: #2322
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_KEY_REF", None);
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("managed-external")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_key_reference_missing")
        ),
        "managed-external strict signer selection must require key reference env marker"
    );
}

#[test]
fn regression_kolme_live_managed_external_rejects_invalid_key_reference_schema() {
    // Regression: #2322
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_KEY_REF", Some("invalid:key-ref"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("managed-external")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_key_reference_invalid")
        ),
        "invalid managed-external key reference schema must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_rejects_raw_private_key_env_path() {
    // Regression: #2322
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("managed-external")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_raw_private_key_forbidden")
        ),
        "managed-external strict signer selection must reject raw private key env path"
    );
}

#[test]
fn regression_kolme_live_managed_external_strict_contracts_require_backend_command_marker() {
    // Regression: #2432
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE).as_str()),
    );
    let _backend_command_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2432-missing-backend-command",
        "state:node-live-2432-missing-backend-command",
        "kamn:did:agent:node-live-2432-missing-backend-command",
        1,
        "payload:node-live-2432-missing-backend-command",
    )
    .expect("request should build");

    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":42,"account_id":"acct-2432"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        Some("ops-primary"),
        Some("managed-external"),
    )
    .expect_err("strict managed-external signer contracts must require backend command marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_required_missing")),
        "strict managed-external runtime path must fail closed with deterministic missing backend reason code"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "managed-external missing backend command must fail before nonce lookup"
    );
}

#[test]
fn regression_kolme_live_managed_external_required_marker_rejects_invalid_boolean() {
    // Regression: #2432
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _required_marker_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED",
        Some("invalid-bool"),
    );
    assert!(
        matches!(
            resolve_kolme_live_managed_signer_required_marker(),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_backend_required_invalid")
        ),
        "managed signer required marker must reject non-boolean values"
    );
}

#[test]
fn regression_kolme_live_managed_external_required_marker_forces_backend_command() {
    // Regression: #2432
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE).as_str()),
    );
    let _backend_command_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", None);
    let _required_marker_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED", Some("true"));
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2432-required-marker",
        "state:node-live-2432-required-marker",
        "kamn:did:agent:node-live-2432-required-marker",
        1,
        "payload:node-live-2432-required-marker",
    )
    .expect("request should build");

    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":43,"account_id":"acct-2432-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed signer required marker must force backend command contract");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_required_missing")),
        "required marker should fail closed when backend command is absent"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "required-marker managed-external path must fail before nonce lookup"
    );
}

#[test]
fn regression_kolme_live_managed_external_requires_backend_command_without_required_marker() {
    // Regression: #2505
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE).as_str()),
    );
    let _backend_command_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", None);
    let _required_marker_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2505-command-required",
        "state:node-live-2505-command-required",
        "kamn:did:agent:node-live-2505-command-required",
        1,
        "payload:node-live-2505-command-required",
    )
    .expect("request should build");
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":44,"account_id":"acct-2505-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external signer mode must require backend command marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_required_missing")),
        "managed-external signer mode without backend command must fail closed"
    );
}

#[test]
fn integration_kolme_live_managed_external_builds_direct_signed_wire_payload() {
    // Regression: #2323
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2323",
        "state:node-live-2323",
        "kamn:did:agent:node-live-2323",
        1,
        "payload:node-live-2323",
    )
    .expect("request should build");
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let _managed_signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_pubkey.as_str()),
    );
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 41)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        managed_pubkey,
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );

    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":41,"account_id":"acct-2323"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        Some("ops-primary"),
        Some("managed-external"),
    )
    .expect("managed-external signing should succeed through secure backend route");
    assert_eq!(signer_selection.profile, "ops-primary");
    assert_eq!(signer_selection.key_source, "managed-external");
    assert_eq!(
        signer_selection.private_key_env,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
    );
    let signature = extract_json_string_field(signed_wire_payload.as_str(), "signature")
        .expect("direct signed payload must include signature field");
    assert_eq!(signature.len(), 128);
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        1,
        "managed-external signing should issue one nonce lookup before payload emission"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_response_requires_signer_public_key_marker() {
    // Regression: #2509
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2509-provenance-required",
        "state:node-live-2509-provenance-required",
        "kamn:did:agent:node-live-2509-provenance-required",
        1,
        "payload:node-live-2509-provenance-required",
    )
    .expect("request should build");
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let _managed_signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_pubkey.as_str()),
    );
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 45)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte()
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":45,"account_id":"acct-2509-provenance-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external backend response must include signer public key marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_response_provenance_missing")),
        "missing managed-external signer provenance marker must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_requires_runtime_signer_public_key_marker() {
    // Regression: #2512
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _signer_pubkey_marker_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2512-pubkey-marker-missing",
        "state:node-live-2512-pubkey-marker-missing",
        "kamn:did:agent:node-live-2512-pubkey-marker-missing",
        1,
        "payload:node-live-2512-pubkey-marker-missing",
    )
    .expect("request should build");
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 47)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        managed_pubkey,
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":47,"account_id":"acct-2512-pubkey-marker-missing"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external runtime path must require signer public key marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_public_key_marker_missing")),
        "missing managed-external signer public key marker must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_rejects_invalid_runtime_signer_public_key_marker() {
    // Regression: #2512
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _signer_pubkey_marker_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some("invalid-pubkey-marker"),
    );
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2512-pubkey-marker-invalid",
        "state:node-live-2512-pubkey-marker-invalid",
        "kamn:did:agent:node-live-2512-pubkey-marker-invalid",
        1,
        "payload:node-live-2512-pubkey-marker-invalid",
    )
    .expect("request should build");
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 48)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        managed_pubkey,
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":48,"account_id":"acct-2512-pubkey-marker-invalid"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("invalid managed-external signer public key marker must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_public_key_marker_invalid")),
        "invalid managed-external signer public key marker must fail closed with deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch() {
    // Regression: #2509
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2509-provenance-mismatch",
        "state:node-live-2509-provenance-mismatch",
        "kamn:did:agent:node-live-2509-provenance-mismatch",
        1,
        "payload:node-live-2509-provenance-mismatch",
    )
    .expect("request should build");
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let _managed_signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_pubkey.as_str()),
    );
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 46)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let secondary_key =
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY)
            .expect("secondary managed signing key should derive");
    let secondary_pubkey = encode_kolme_hex_lower(
        secondary_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        secondary_pubkey,
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":46,"account_id":"acct-2509-provenance-mismatch"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external backend response must reject signer public key mismatch");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_response_provenance_mismatch")),
        "managed-external signer provenance mismatch must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_maps_provider_unavailable_reason_code() {
    // Regression: #2323
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2323-provider",
        "state:node-live-2323-provider",
        "kamn:did:agent:node-live-2323-provider",
        1,
        "payload:node-live-2323-provider",
    )
    .expect("request should build");
    let expected_signer_public_key_hex = encode_kolme_hex_lower(
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive")
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(false),
        expected_signer_public_key_hex.as_str(),
    )
    .expect_err("managed-external provider unavailability must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_provider_unavailable")),
        "managed-external provider unavailability must map to deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_timeout_maps_reason_code() {
    // Regression: #2423
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _backend_command_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", Some("sleep 2"));
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("1"));
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2423-timeout",
        "state:node-live-2423-timeout",
        "kamn:did:agent:node-live-2423-timeout",
        1,
        "payload:node-live-2423-timeout",
    )
    .expect("request should build");
    let expected_signer_public_key_hex = encode_kolme_hex_lower(
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive")
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        expected_signer_public_key_hex.as_str(),
    )
    .expect_err("managed-external backend timeout must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_timeout")),
        "managed-external backend timeout must map to deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_malformed_response_maps_reason_code() {
    // Regression: #2423
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some("printf 'signature_hex=zzzz\\nrecovery_id=9\\nsigner_public_key_hex=03af446f76cf36092a4e45864210a1dbf03e872756eec21de61910859f8a607dd2\\n'"),
    );
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("5"));
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2423-malformed",
        "state:node-live-2423-malformed",
        "kamn:did:agent:node-live-2423-malformed",
        1,
        "payload:node-live-2423-malformed",
    )
    .expect("request should build");
    let expected_signer_public_key_hex = encode_kolme_hex_lower(
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive")
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        expected_signer_public_key_hex.as_str(),
    )
    .expect_err("managed-external backend malformed response must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_response_malformed")),
        "managed-external backend malformed response must map to deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_unavailable_maps_reason_code() {
    // Regression: #2423
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some("this-command-should-not-exist-2423"),
    );
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("5"));
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2423-unavailable",
        "state:node-live-2423-unavailable",
        "kamn:did:agent:node-live-2423-unavailable",
        1,
        "payload:node-live-2423-unavailable",
    )
    .expect("request should build");
    let expected_signer_public_key_hex = encode_kolme_hex_lower(
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive")
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        expected_signer_public_key_hex.as_str(),
    )
    .expect_err("managed-external backend unavailability must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_unavailable")),
        "managed-external backend unavailability must map to deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_adapter_retired_not_integrated_marker() {
    // Regression: #2423
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let error = build_kolme_live_signer_adapter(Some("ops-primary"), Some("managed-external"))
        .expect_err("managed-external private-key adapter path must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if !message.contains("managed_signer_backend_path_not_integrated")),
        "managed-external signer adapter path must retire not-integrated marker"
    );
}

#[test]
fn regression_runtime_kolme_live_rejects_provider_marker_drift() {
    // Regression: #2176
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":23,"account_id":"acct-2176"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"unexpected-provider","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("kolme-live args should parse");
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("provider marker drift")
        ),
        "runtime must fail closed when provider marker drifts from configured hint"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        2,
        "provider drift should fail after nonce lookup and submit response mapping"
    );
}

#[test]
fn regression_runtime_kolme_live_rejects_missing_signer_private_key_env() {
    // Regression: #2220
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
    )]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("kolme-live args should parse");
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX must be set for signer profile ops-primary")
        ),
        "runtime must fail closed when signer private key env is missing"
    );
}

#[test]
fn rejects_missing_role() {
    let args = vec!["kamn-node".to_owned()];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--role"))
    );
}

#[test]
fn rejects_planning_without_expected_state_hash() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--expected-state-hash"))
    );
}

#[test]
fn rejects_planning_without_proposal() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--proposal"))
    );
}

#[test]
fn rejects_recovery_check_without_expected_state_version() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--expected-state-version"
        ))
    );
}

#[test]
fn rejects_recovery_check_without_expected_state_hash() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--expected-state-hash"))
    );
}

#[test]
fn rejects_recovery_check_without_rejoin_attempt() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--rejoin-attempt"))
    );
}

#[test]
fn rejects_daemon_without_max_ticks() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"))
    );
}

#[test]
fn rejects_daemon_without_tick_interval() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--daemon-tick-interval-ms"
        ))
    );
}

#[test]
fn rejects_kolme_live_without_base_url() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue("--kolme-live-base-url"))
    );
}

#[test]
fn rejects_kolme_live_without_provider_hint() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-provider-hint"
        ))
    );
}

#[test]
fn rejects_kolme_live_without_signing_profile() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-signing-profile"
        ))
    );
}

#[test]
fn rejects_kolme_live_without_signer_key_source() {
    // Regression: #2626
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-signer-key-source"
        ))
    );
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_without_signer_profile_selector() {
    // Regression: #2246
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-signer-profile"
        ))
    );
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_without_key_source() {
    // Regression: #2246
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::MissingArgumentValue(
            "--kolme-live-signer-key-source"
        ))
    );
}

#[test]
fn parses_kolme_live_strict_signer_contracts_with_managed_external_key_source() {
    // Regression: #2322
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "managed-external".to_owned(),
    ];
    parse_args(args)
        .expect("strict signer contract declarations should parse managed-external markers");
}

#[test]
fn parses_kolme_live_strict_signer_contracts_with_explicit_declarations() {
    // Regression: #2246
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];
    parse_args(args).expect("strict signer contract declarations should parse");
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_with_empty_signer_profile_selector() {
    // Regression: #2247
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        " ".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];
    assert!(
        matches!(
            parse_args(args),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("--kolme-live-signer-profile must not be empty")
        ),
        "strict signer contracts must reject empty signer profile selector"
    );
}

#[test]
fn rejects_kolme_live_strict_signer_contracts_with_empty_key_source() {
    // Regression: #2247
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        " ".to_owned(),
    ];
    assert!(
        matches!(
            parse_args(args),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("--kolme-live-signer-key-source must not be empty")
        ),
        "strict signer contracts must reject empty key-source declaration"
    );
}

#[test]
fn regression_kolme_live_strict_signer_contracts_reject_profile_selector_env_mismatch() {
    // Regression: #2247
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );

    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("env-local")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("strict signer profile mismatch")
        ),
        "strict signer contracts must reject selector/env profile mismatch"
    );
}

#[test]
fn integration_runtime_kolme_live_strict_signer_contracts_fail_closed_before_network_on_selector_env_mismatch(
) {
    // Regression: #2247
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];
    let parsed = parse_args(args).expect("strict kolme-live args should parse");
    assert!(
        matches!(
            execute(parsed),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("strict signer profile mismatch")
        ),
        "runtime must fail closed before network submit when strict signer selector conflicts with env marker"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "strict selector/env mismatch should fail before any live network request"
    );
}

#[test]
fn rejects_kolme_live_with_invalid_signing_profile() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "synthetic-signing-profile".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidKolmeLiveSigningProfile(
            "synthetic-signing-profile".to_owned()
        ))
    );
}

#[test]
fn rejects_kolme_live_with_in_memory_provider_hint_marker() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "InMemoryKolmeRuntimeCommitClient".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidKolmeLiveProviderHint(
            "InMemoryKolmeRuntimeCommitClient".to_owned()
        ))
    );
}

#[test]
fn rejects_unknown_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "approver".to_owned(),
        "--unknown".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::UnknownArgument("--unknown".to_owned()))
    );
}

#[test]
fn rejects_invalid_output_mode() {
    // Regression: #307
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "approver".to_owned(),
        "--output".to_owned(),
        "yaml".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidOutputMode("yaml".to_owned()))
    );
}

#[test]
fn rejects_invalid_profile_value() {
    // Regression: #310
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-unknown".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidNodeProfile("local-unknown".to_owned()))
    );
}

#[test]
fn rejects_invalid_runtime_mode() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "service".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidRuntimeMode("service".to_owned()))
    );
}

#[test]
fn rejects_malformed_proposal_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|state-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidProposalArgument(
            "tx-1|did:kamn:agent:aaa|state-1".to_owned()
        ))
    );
}

#[test]
fn rejects_malformed_rejoin_attempt_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidRejoinAttemptArgument(
            "node-a|42|state-42".to_owned()
        ))
    );
}

#[test]
fn rejects_invalid_expected_state_version_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "0".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidExpectedStateVersion("0".to_owned()))
    );
}

#[test]
fn rejects_invalid_daemon_control_argument() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "abc".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidDaemonControlArgument("abc".to_owned()))
    );
}

#[test]
fn rejects_invalid_daemon_lifecycle_event_argument() {
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
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "resume".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidDaemonLifecycleEvent(
            "resume".to_owned()
        ))
    );
}

#[test]
fn rejects_invalid_diagnostics_mode() {
    // Regression: #313
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--diagnostics".to_owned(),
        "extended".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidDiagnosticsMode("extended".to_owned()))
    );
}

#[test]
fn regression_runtime_planning_rejects_duplicate_candidate_ids() {
    // Regression: #335
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:bbb|2|state-1".to_owned(),
    ];
    let parsed = parse_args(args).expect("planning args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimePlanner(
            "duplicate proposal candidate id: tx-1".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_planning_rejects_stale_state_hash() {
    // Regression: #335
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-2".to_owned(),
    ];
    let parsed = parse_args(args).expect("planning args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimePlanner(
            "proposal candidate state hash mismatch: expected state-1, found state-2".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_recovery_rejects_replay_resume_token() {
    // Regression: #336
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];
    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery(
            "rejoin resume token replayed: resume-1".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_recovery_rejects_state_version_mismatch() {
    // Regression: #336
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|43|state-43|resume-1".to_owned(),
    ];
    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery(
            "rejoin state version mismatch: expected 42, found 43".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_recovery_rejects_state_hash_mismatch() {
    // Regression: #336
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-41|resume-1".to_owned(),
    ];
    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeRecovery(
            "rejoin state hash mismatch: expected state-42, found state-41".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_daemon_rejects_zero_tick_budget() {
    // Regression: #348
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "0".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ];
    assert_eq!(
        parse_args(args),
        Err(ConfigError::InvalidDaemonControlArgument("0".to_owned()))
    );
}

#[test]
fn regression_runtime_daemon_rejects_invalid_lifecycle_transition() {
    // Regression: #349
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
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
    ];
    let parsed = parse_args(args).expect("daemon args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeDaemonLifecycle(
            "invalid peer lifecycle transition from Disconnected via HandshakeSucceeded".to_owned()
        ))
    );
}

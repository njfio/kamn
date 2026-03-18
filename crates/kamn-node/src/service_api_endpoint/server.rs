use super::*;
use std::io::BufReader;

const SERVICE_API_TLS_REQUIRED_RUNTIME_MODES: &[&str] = &["daemon", "api", "full", "kolme-live"];

fn resolve_service_api_auth_public_key_hex() -> Result<Option<String>, String> {
    match env::var(SERVICE_API_AUTH_PUBLIC_KEY_HEX_ENV) {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(format!(
                    "service api auth public key env must not be empty: {SERVICE_API_AUTH_PUBLIC_KEY_HEX_ENV}"
                ));
            }
            Ok(Some(normalized.to_owned()))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "service api auth public key env must be utf-8: {SERVICE_API_AUTH_PUBLIC_KEY_HEX_ENV}"
        )),
    }
}

fn resolve_service_api_state_file(
    config: &ServiceApiEndpointConfig,
) -> Result<Option<String>, String> {
    resolve_service_api_state_file_from_env(env::var(SERVICE_API_STATE_FILE_ENV), config)
}

fn resolve_service_api_state_file_from_env(
    env_value: Result<String, env::VarError>,
    config: &ServiceApiEndpointConfig,
) -> Result<Option<String>, String> {
    match env_value {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(format!(
                    "service api state file env must not be empty: {SERVICE_API_STATE_FILE_ENV}"
                ));
            }
            Ok(Some(normalized.to_owned()))
        }
        Err(env::VarError::NotPresent) => Ok(Some(default_service_api_state_file_path(config))),
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "service api state file env must be utf-8: {SERVICE_API_STATE_FILE_ENV}"
        )),
    }
}

fn default_service_api_state_file_path(config: &ServiceApiEndpointConfig) -> String {
    super::default_service_api_state_file_path_for_bind_addr(config.bind_addr.as_str())
}

fn resolve_service_api_relay_spool_file(
    state_file: Option<&str>,
) -> Result<Option<String>, String> {
    resolve_service_api_relay_spool_file_from_env(
        env::var(SERVICE_API_RELAY_SPOOL_FILE_ENV),
        state_file,
    )
}

fn resolve_service_api_relay_spool_file_from_env(
    env_value: Result<String, env::VarError>,
    state_file: Option<&str>,
) -> Result<Option<String>, String> {
    match env_value {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(format!(
                    "service api relay spool env must not be empty: {SERVICE_API_RELAY_SPOOL_FILE_ENV}"
                ));
            }
            Ok(Some(normalized.to_owned()))
        }
        Err(env::VarError::NotPresent) => {
            Ok(state_file.map(super::default_service_api_relay_spool_file_path_from_state_file))
        }
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "service api relay spool env must be utf-8: {SERVICE_API_RELAY_SPOOL_FILE_ENV}"
        )),
    }
}

fn build_service_api_replay_guard(message_store: &ServiceApiMessageStore) -> ServiceApiReplayGuard {
    let mut replay_guard = ServiceApiReplayGuard::new(
        DEFAULT_SERVICE_API_REPLAY_GUARD_MAX_ENTRIES,
        Duration::from_secs(DEFAULT_SERVICE_API_REPLAY_GUARD_TTL_SECS),
    );
    for (sender_did, nonce) in message_store.auth_nonce_high_watermarks() {
        replay_guard.seed_sender_nonce_high_watermark(sender_did.as_str(), nonce);
    }
    replay_guard
}

pub(super) fn resolve_service_api_tls_mode(
    runtime_mode: &str,
    bind_addr: &str,
) -> Result<ServiceApiTlsMode, String> {
    resolve_service_api_tls_mode_from_env(
        env::var(SERVICE_API_TLS_MODE_ENV),
        env::var(SERVICE_API_TLS_CERT_FILE_ENV),
        env::var(SERVICE_API_TLS_KEY_FILE_ENV),
        runtime_mode,
        bind_addr,
        cfg!(test),
    )
}

fn resolve_service_api_tls_mode_from_env(
    mode_env: Result<String, env::VarError>,
    cert_file_env: Result<String, env::VarError>,
    key_file_env: Result<String, env::VarError>,
    runtime_mode: &str,
    bind_addr: &str,
    allow_insecure_default: bool,
) -> Result<ServiceApiTlsMode, String> {
    let tls_mode = match mode_env {
        Ok(value) => {
            let mode = value.trim().to_ascii_lowercase();
            if mode.is_empty() {
                return Err(format!(
                    "service api tls mode env must not be empty: {SERVICE_API_TLS_MODE_ENV}"
                ));
            }
            match mode.as_str() {
                SERVICE_API_TLS_MODE_DISABLED => Ok(ServiceApiTlsMode::Disabled),
                SERVICE_API_TLS_MODE_REQUIRE => {
                    resolve_service_api_required_tls_mode_from_env(cert_file_env, key_file_env)
                }
                other => Err(format!(
                    "service api tls mode is invalid: {other} (supported: {SERVICE_API_TLS_MODE_DISABLED},{SERVICE_API_TLS_MODE_REQUIRE})"
                )),
            }
        }
        Err(env::VarError::NotPresent) => {
            if allow_insecure_default {
                return Ok(ServiceApiTlsMode::Disabled);
            }
            resolve_service_api_required_tls_mode_from_env(cert_file_env, key_file_env).map_err(
                |error| {
                    format!(
                        "service api tls mode defaults to require when {SERVICE_API_TLS_MODE_ENV} is unset: {error}"
                    )
                },
            )
        }
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "service api tls mode env must be utf-8: {SERVICE_API_TLS_MODE_ENV}"
        )),
    }?;
    enforce_service_api_tls_runtime_policy(runtime_mode, bind_addr, &tls_mode)?;
    Ok(tls_mode)
}

fn runtime_mode_requires_service_api_tls(runtime_mode: &str) -> bool {
    let normalized = runtime_mode.trim();
    SERVICE_API_TLS_REQUIRED_RUNTIME_MODES
        .iter()
        .any(|required_mode| normalized.eq_ignore_ascii_case(required_mode))
}

fn service_api_bind_addr_is_loopback(bind_addr: &str) -> bool {
    let normalized = bind_addr.trim();
    if let Ok(addr) = normalized.parse::<SocketAddr>() {
        return addr.ip().is_loopback();
    }
    normalized
        .strip_prefix("localhost:")
        .is_some_and(|port| !port.trim().is_empty())
}

fn enforce_service_api_tls_runtime_policy(
    runtime_mode: &str,
    bind_addr: &str,
    tls_mode: &ServiceApiTlsMode,
) -> Result<(), String> {
    if !runtime_mode_requires_service_api_tls(runtime_mode) {
        return Ok(());
    }
    if *tls_mode != ServiceApiTlsMode::Disabled {
        return Ok(());
    }
    if service_api_bind_addr_is_loopback(bind_addr) {
        return Ok(());
    }
    Err(format!(
        "service api tls disabled is forbidden for runtime mode {runtime_mode} on non-loopback bind address {bind_addr} (set {SERVICE_API_TLS_MODE_ENV}={SERVICE_API_TLS_MODE_REQUIRE} with {SERVICE_API_TLS_CERT_FILE_ENV}/{SERVICE_API_TLS_KEY_FILE_ENV})"
    ))
}

fn resolve_service_api_required_tls_mode_from_env(
    cert_file_env: Result<String, env::VarError>,
    key_file_env: Result<String, env::VarError>,
) -> Result<ServiceApiTlsMode, String> {
    let cert_file = match cert_file_env {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(format!(
                    "service api tls cert env must not be empty: {SERVICE_API_TLS_CERT_FILE_ENV}"
                ));
            }
            normalized.to_owned()
        }
        Err(env::VarError::NotPresent) => {
            return Err(format!(
                "service api tls mode requires env: {SERVICE_API_TLS_CERT_FILE_ENV}"
            ));
        }
        Err(env::VarError::NotUnicode(_)) => {
            return Err(format!(
                "service api tls cert env must be utf-8: {SERVICE_API_TLS_CERT_FILE_ENV}"
            ));
        }
    };
    let key_file = match key_file_env {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(format!(
                    "service api tls key env must not be empty: {SERVICE_API_TLS_KEY_FILE_ENV}"
                ));
            }
            normalized.to_owned()
        }
        Err(env::VarError::NotPresent) => {
            return Err(format!(
                "service api tls mode requires env: {SERVICE_API_TLS_KEY_FILE_ENV}"
            ));
        }
        Err(env::VarError::NotUnicode(_)) => {
            return Err(format!(
                "service api tls key env must be utf-8: {SERVICE_API_TLS_KEY_FILE_ENV}"
            ));
        }
    };
    validate_service_api_tls_materials(cert_file.as_str(), key_file.as_str())?;
    Ok(ServiceApiTlsMode::Require {
        cert_file,
        key_file,
    })
}

pub(super) fn validate_service_api_tls_materials(
    cert_file: &str,
    key_file: &str,
) -> Result<(), String> {
    let cert_bytes = fs::read(cert_file).map_err(|error| {
        format!("service api tls certificate file read failed: {cert_file}: {error}")
    })?;
    let key_bytes = fs::read(key_file)
        .map_err(|error| format!("service api tls key file read failed: {key_file}: {error}"))?;

    let mut cert_reader = BufReader::new(cert_bytes.as_slice());
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!("service api tls certificate file parse failed: {cert_file}: {error}")
        })?;
    if certs.is_empty() {
        return Err(format!(
            "service api tls certificate file parse failed: {cert_file}: no certificates found"
        ));
    }

    let mut key_reader = BufReader::new(key_bytes.as_slice());
    let private_key = rustls_pemfile::private_key(&mut key_reader)
        .map_err(|error| format!("service api tls key file parse failed: {key_file}: {error}"))?;
    if private_key.is_none() {
        return Err(format!(
            "service api tls key file parse failed: {key_file}: no private key found"
        ));
    }
    Ok(())
}

pub(super) async fn serve_service_api_endpoint_async(
    config: ServiceApiEndpointConfig,
    snapshot: ServiceApiSnapshot,
) -> Result<(), String> {
    let tls_mode =
        resolve_service_api_tls_mode(snapshot.runtime_mode.as_str(), config.bind_addr.as_str())?;
    let live_solana_bridge_dispatch =
        super::live_bridge_dispatch::resolve_live_solana_bridge_dispatch_config()?;
    let live_solana_settlement =
        super::live_settlement_dispatch::resolve_live_solana_settlement_config(
            live_solana_bridge_dispatch.as_ref(),
        )?;
    let auth_public_key_hex = resolve_service_api_auth_public_key_hex()?;
    let state_file = resolve_service_api_state_file(&config)?;
    let relay_spool_file = resolve_service_api_relay_spool_file(state_file.as_deref())?;
    let message_store = ServiceApiMessageStore::from_optional_state_file(state_file)?;
    let replay_guard = build_service_api_replay_guard(&message_store);
    let sender_anti_spam = build_service_api_sender_anti_spam_engine()
        .map_err(|error| format!("service api anti-spam init failed: {error}"))?;

    let runtime_state = Arc::new(ServiceApiRuntimeState {
        snapshot,
        replay_guard: Arc::new(Mutex::new(replay_guard)),
        request_budget: Arc::new(ServiceApiRequestBudget::new(config.max_requests)),
        websocket_events: ServiceApiWebsocketEventFanout::new(),
        runtime_observability: Arc::new(Mutex::new(ServiceApiRuntimeObservability::new(
            Instant::now(),
        ))),
        body_limit_bytes: config.body_limit_bytes as usize,
        concurrency_limiter: Arc::new(Semaphore::new(config.concurrency_limit as usize)),
        ingress_rate_window: Arc::new(Mutex::new(ServiceApiIngressRateWindow::new(
            config.rate_limit_per_second,
        ))),
        sender_anti_spam: Arc::new(Mutex::new(sender_anti_spam)),
        auth_public_key_hex,
        message_store: Arc::new(Mutex::new(message_store)),
        relay_spool_file,
        live_solana_bridge_dispatch,
        live_solana_settlement,
    });
    let request_budget_shared = runtime_state.request_budget.clone();
    let timeout_reached = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now() + Duration::from_millis(config.idle_timeout_ms);

    let app = build_service_api_router(runtime_state);

    match tls_mode {
        ServiceApiTlsMode::Disabled => {
            if !cfg!(test) {
                eprintln!(
                    "warning: service api tls disabled; traffic is plaintext (set {SERVICE_API_TLS_MODE_ENV}={SERVICE_API_TLS_MODE_REQUIRE} with {SERVICE_API_TLS_CERT_FILE_ENV}/{SERVICE_API_TLS_KEY_FILE_ENV})"
                );
            }
            let request_budget = request_budget_shared.clone();
            let timeout_flag = timeout_reached.clone();
            let listener = tokio::net::TcpListener::bind(config.bind_addr.as_str())
                .await
                .map_err(|error| format!("service api bind failed: {error}"))?;
            axum::serve(listener, app.clone())
                .with_graceful_shutdown(async move {
                    let wait_for_budget = request_budget.wait_until_complete();
                    tokio::pin!(wait_for_budget);
                    let idle_timeout = tokio::time::sleep_until(deadline.into());
                    tokio::pin!(idle_timeout);
                    tokio::select! {
                        _ = &mut wait_for_budget => {}
                        _ = &mut idle_timeout => {
                            timeout_flag.store(true, Ordering::SeqCst);
                        }
                    }
                })
                .await
                .map_err(|error| format!("service api serve failed: {error}"))?;
        }
        ServiceApiTlsMode::Require {
            cert_file,
            key_file,
        } => {
            let request_budget = request_budget_shared.clone();
            let timeout_flag = timeout_reached.clone();
            let bind_addr = config.bind_addr.parse::<SocketAddr>().map_err(|error| {
                format!(
                    "service api tls bind address parse failed: {}: {error}",
                    config.bind_addr
                )
            })?;
            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
                cert_file.clone(),
                key_file.clone(),
            )
            .await
            .map_err(|error| {
                format!(
                    "service api tls config load failed: cert_file={cert_file}, key_file={key_file}: {error}"
                )
            })?;

            let handle = axum_server::Handle::new();
            let shutdown_handle = handle.clone();
            tokio::spawn(async move {
                let wait_for_budget = request_budget.wait_until_complete();
                tokio::pin!(wait_for_budget);
                let idle_timeout = tokio::time::sleep_until(deadline.into());
                tokio::pin!(idle_timeout);
                tokio::select! {
                    _ = &mut wait_for_budget => {}
                    _ = &mut idle_timeout => {
                        timeout_flag.store(true, Ordering::SeqCst);
                    }
                }
                shutdown_handle.graceful_shutdown(None);
            });

            axum_server::bind_rustls(bind_addr, rustls_config)
                .handle(handle)
                .serve(app.clone().into_make_service())
                .await
                .map_err(|error| format!("service api tls serve failed: {error}"))?;
        }
    }

    if timeout_reached.load(Ordering::SeqCst) {
        return Err(format!(
            "service api timed out after {} ms waiting for requests",
            config.idle_timeout_ms
        ));
    }

    Ok(())
}

pub(super) fn build_service_api_sender_anti_spam_engine() -> Result<AntiSpamEngine, String> {
    let config = AntiSpamConfig {
        minimum_sybil_deposit: 0,
        ..AntiSpamConfig::default()
    };
    AntiSpamEngine::new(config).map_err(|error| error.to_string())
}

pub(super) fn build_service_api_router(state: Arc<ServiceApiRuntimeState>) -> Router {
    Router::new()
        .route(
            ROUTE_EVENTS_WS,
            get(super::handle_service_api_websocket_route),
        )
        .route("/", any(super::handle_service_api_http_route))
        .route("/{*path}", any(super::handle_service_api_http_route))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            super::service_api_auth_middleware,
        ))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture_endpoint_config(bind_addr: &str) -> ServiceApiEndpointConfig {
        ServiceApiEndpointConfig {
            bind_addr: bind_addr.to_owned(),
            max_requests: 1,
            idle_timeout_ms: 1_000,
            body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
            concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
            rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
        }
    }

    #[test]
    fn unit_service_api_state_file_resolution_prefers_explicit_env_override() {
        let config = fixture_endpoint_config("127.0.0.1:34079");
        let resolved = resolve_service_api_state_file_from_env(
            Ok("  /tmp/custom-service-api-state.json  ".to_owned()),
            &config,
        )
        .expect("explicit env state file should resolve");
        assert_eq!(
            resolved,
            Some("/tmp/custom-service-api-state.json".to_owned())
        );
    }

    #[test]
    fn unit_service_api_state_file_resolution_derives_deterministic_default_path_when_env_missing()
    {
        let config = fixture_endpoint_config("127.0.0.1:34079");
        let resolved =
            resolve_service_api_state_file_from_env(Err(env::VarError::NotPresent), &config)
                .expect("missing env should derive default state file");
        let expected = Some(default_service_api_state_file_path(&config));
        assert_eq!(resolved, expected);
        let resolved_path = resolved.expect("default path should exist");
        assert!(
            resolved_path.contains("kamn-node-service-api-state-127-0-0-1-34079.json"),
            "default path should encode sanitized bind address: {resolved_path}"
        );
    }

    #[test]
    fn unit_service_api_relay_spool_resolution_prefers_explicit_env_override() {
        let resolved = resolve_service_api_relay_spool_file_from_env(
            Ok("  /tmp/custom-relay-spool.ndjson  ".to_owned()),
            Some("/tmp/state.json"),
        )
        .expect("relay spool should resolve from explicit env");
        assert_eq!(resolved, Some("/tmp/custom-relay-spool.ndjson".to_owned()));
    }

    #[test]
    fn unit_service_api_relay_spool_resolution_derives_from_state_path() {
        let resolved = resolve_service_api_relay_spool_file_from_env(
            Err(env::VarError::NotPresent),
            Some("/tmp/state-store.json"),
        )
        .expect("state-derived relay spool should resolve");
        assert_eq!(
            resolved,
            Some("/tmp/state-store.json.relay.ndjson".to_owned())
        );
    }

    #[test]
    fn regression_service_api_tls_mode_resolution_fails_closed_when_unset_in_production_mode() {
        // Regression: #6185
        let error = resolve_service_api_tls_mode_from_env(
            Err(env::VarError::NotPresent),
            Err(env::VarError::NotPresent),
            Err(env::VarError::NotPresent),
            "api",
            "0.0.0.0:34080",
            false,
        )
        .expect_err("production default must fail closed when tls mode env is unset");
        assert!(
            error.contains("service api tls mode defaults to require"),
            "production default failure should include deterministic marker: {error}"
        );
    }

    #[test]
    fn unit_service_api_tls_mode_resolution_allows_insecure_default_in_test_mode() {
        let mode = resolve_service_api_tls_mode_from_env(
            Err(env::VarError::NotPresent),
            Err(env::VarError::NotPresent),
            Err(env::VarError::NotPresent),
            "api",
            "127.0.0.1:34081",
            true,
        )
        .expect("test-mode default may resolve disabled");
        assert_eq!(mode, ServiceApiTlsMode::Disabled);
    }

    #[test]
    fn regression_service_api_tls_mode_resolution_rejects_explicit_disabled_mode_for_production_paths(
    ) {
        let error = resolve_service_api_tls_mode_from_env(
            Ok("disabled".to_owned()),
            Err(env::VarError::NotPresent),
            Err(env::VarError::NotPresent),
            "api",
            "0.0.0.0:34082",
            false,
        )
        .expect_err("production runtime paths must reject explicit disabled tls mode");
        assert!(
            error.contains("service api tls disabled is forbidden"),
            "explicit disabled production rejection should include deterministic marker: {error}"
        );
    }

    #[test]
    fn unit_service_api_tls_mode_resolution_allows_explicit_disabled_for_loopback_local_path() {
        let mode = resolve_service_api_tls_mode_from_env(
            Ok("disabled".to_owned()),
            Err(env::VarError::NotPresent),
            Err(env::VarError::NotPresent),
            "api",
            "127.0.0.1:34083",
            false,
        )
        .expect("loopback-bound local path may explicitly opt into disabled tls");
        assert_eq!(mode, ServiceApiTlsMode::Disabled);
    }

    #[test]
    fn integration_service_api_replay_guard_seeds_nonce_high_watermark_from_state() {
        let unique_suffix = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be monotonic")
                .as_nanos()
        );
        let state_file = std::env::temp_dir().join(format!(
            "kamn-node-service-api-replay-seed-state-{unique_suffix}.json"
        ));
        let state_path = state_file.to_string_lossy().to_string();
        let mut store = ServiceApiMessageStore::from_optional_state_file(Some(state_path))
            .expect("state-backed store should initialize");
        store
            .record_auth_nonce_high_watermark("kamn:did:agent:seeded", 9)
            .expect("nonce high-watermark should persist");

        let mut replay_guard = build_service_api_replay_guard(&store);
        let start = Instant::now();
        assert!(!replay_guard.record_nonce_if_fresh("kamn:did:agent:seeded", 9, start));
        assert!(replay_guard.record_nonce_if_fresh(
            "kamn:did:agent:seeded",
            10,
            start + Duration::from_secs(1)
        ));

        let _ = std::fs::remove_file(state_file);
    }
}

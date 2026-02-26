use super::*;
use std::io::BufReader;

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

fn resolve_service_api_auth_public_keys_by_did() -> Result<Option<BTreeMap<String, String>>, String>
{
    resolve_service_api_auth_public_keys_by_did_from_env(env::var(
        SERVICE_API_AUTH_PUBLIC_KEYS_BY_DID_JSON_ENV,
    ))
}

fn resolve_service_api_auth_public_keys_by_did_from_env(
    env_value: Result<String, env::VarError>,
) -> Result<Option<BTreeMap<String, String>>, String> {
    match env_value {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(format!(
                    "service api auth did key map env must not be empty: {SERVICE_API_AUTH_PUBLIC_KEYS_BY_DID_JSON_ENV}"
                ));
            }
            let parsed = serde_json::from_str::<BTreeMap<String, String>>(normalized).map_err(
                |error| {
                    format!(
                        "service api auth did key map env must be valid JSON object: {SERVICE_API_AUTH_PUBLIC_KEYS_BY_DID_JSON_ENV}: {error}"
                    )
                },
            )?;
            if parsed.is_empty() {
                return Err(format!(
                    "service api auth did key map env must contain at least one sender entry: {SERVICE_API_AUTH_PUBLIC_KEYS_BY_DID_JSON_ENV}"
                ));
            }
            let mut normalized_map = BTreeMap::new();
            for (sender_did, public_key_hex) in parsed {
                AgentDid::parse(sender_did.as_str()).map_err(|error| {
                    format!(
                        "service api auth did key map has invalid sender did in {SERVICE_API_AUTH_PUBLIC_KEYS_BY_DID_JSON_ENV}: {sender_did}: {error}"
                    )
                })?;
                let public_key_hex = public_key_hex.trim();
                if public_key_hex.is_empty() {
                    return Err(format!(
                        "service api auth did key map has empty public key for sender {sender_did}: {SERVICE_API_AUTH_PUBLIC_KEYS_BY_DID_JSON_ENV}"
                    ));
                }
                normalized_map.insert(sender_did, public_key_hex.to_owned());
            }
            Ok(Some(normalized_map))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "service api auth did key map env must be utf-8: {SERVICE_API_AUTH_PUBLIC_KEYS_BY_DID_JSON_ENV}"
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

fn resolve_service_api_replay_guard_state_file(state_file: Option<&str>) -> Option<String> {
    state_file.map(super::default_service_api_replay_guard_state_file_path_from_state_file)
}

pub(super) fn resolve_service_api_tls_mode(bind_addr: &str) -> Result<ServiceApiTlsMode, String> {
    resolve_service_api_tls_mode_from_env(env::var(SERVICE_API_TLS_MODE_ENV), bind_addr)
}

fn resolve_service_api_tls_mode_from_env(
    env_value: Result<String, env::VarError>,
    bind_addr: &str,
) -> Result<ServiceApiTlsMode, String> {
    match env_value {
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
                    let cert_file = env::var(SERVICE_API_TLS_CERT_FILE_ENV)
                        .map_err(|_| {
                            format!(
                                "service api tls mode requires env: {SERVICE_API_TLS_CERT_FILE_ENV}"
                            )
                        })?
                        .trim()
                        .to_owned();
                    if cert_file.is_empty() {
                        return Err(format!(
                            "service api tls cert env must not be empty: {SERVICE_API_TLS_CERT_FILE_ENV}"
                        ));
                    }
                    let key_file = env::var(SERVICE_API_TLS_KEY_FILE_ENV)
                        .map_err(|_| {
                            format!(
                                "service api tls mode requires env: {SERVICE_API_TLS_KEY_FILE_ENV}"
                            )
                        })?
                        .trim()
                        .to_owned();
                    if key_file.is_empty() {
                        return Err(format!(
                            "service api tls key env must not be empty: {SERVICE_API_TLS_KEY_FILE_ENV}"
                        ));
                    }
                    validate_service_api_tls_materials(cert_file.as_str(), key_file.as_str())?;
                    Ok(ServiceApiTlsMode::Require {
                        cert_file,
                        key_file,
                    })
                }
                other => Err(format!(
                    "service api tls mode is invalid: {other} (supported: {SERVICE_API_TLS_MODE_DISABLED},{SERVICE_API_TLS_MODE_REQUIRE})"
                )),
            }
        }
        Err(env::VarError::NotPresent) => {
            if service_api_bind_addr_is_loopback(bind_addr) {
                Ok(ServiceApiTlsMode::Disabled)
            } else {
                Err(format!(
                    "service api tls mode must be explicitly configured for non-loopback bind address {bind_addr}: set {SERVICE_API_TLS_MODE_ENV}={SERVICE_API_TLS_MODE_DISABLED}|{SERVICE_API_TLS_MODE_REQUIRE}"
                ))
            }
        }
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "service api tls mode env must be utf-8: {SERVICE_API_TLS_MODE_ENV}"
        )),
    }
}

fn service_api_bind_addr_is_loopback(bind_addr: &str) -> bool {
    if let Ok(socket_addr) = bind_addr.parse::<SocketAddr>() {
        return socket_addr.ip().is_loopback();
    }
    let host = bind_addr
        .rsplit_once(':')
        .map(|(host, _)| host)
        .unwrap_or(bind_addr)
        .trim();
    let normalized = host.trim_start_matches('[').trim_end_matches(']');
    normalized.eq_ignore_ascii_case("localhost") || normalized == "127.0.0.1" || normalized == "::1"
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
    let tls_mode = resolve_service_api_tls_mode(config.bind_addr.as_str())?;
    if matches!(tls_mode, ServiceApiTlsMode::Disabled)
        && !service_api_bind_addr_is_loopback(config.bind_addr.as_str())
    {
        let _ = log_warn(
            "service.api.tls.disabled.non_loopback",
            &[
                ("bind_addr", config.bind_addr.as_str()),
                ("tls_mode_env", SERVICE_API_TLS_MODE_ENV),
            ],
        );
    }
    let auth_public_key_hex = resolve_service_api_auth_public_key_hex()?;
    let auth_public_keys_by_did = resolve_service_api_auth_public_keys_by_did()?;
    let state_file = resolve_service_api_state_file(&config)?;
    let relay_spool_file = resolve_service_api_relay_spool_file(state_file.as_deref())?;
    let replay_guard_state_file =
        resolve_service_api_replay_guard_state_file(state_file.as_deref());
    let message_store = ServiceApiMessageStore::from_optional_state_file(state_file)?;
    let sender_anti_spam = build_service_api_sender_anti_spam_engine()
        .map_err(|error| format!("service api anti-spam init failed: {error}"))?;

    let runtime_state = Arc::new(ServiceApiRuntimeState {
        snapshot,
        replay_guard: Arc::new(Mutex::new(
            ServiceApiReplayGuard::from_state_file(
                DEFAULT_SERVICE_API_REPLAY_GUARD_MAX_ENTRIES,
                Duration::from_secs(DEFAULT_SERVICE_API_REPLAY_GUARD_TTL_SECS),
                replay_guard_state_file.as_deref(),
            )
            .map_err(|error| format!("service api replay guard init failed: {error}"))?,
        )),
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
        auth_public_keys_by_did,
        message_store: Arc::new(Mutex::new(message_store)),
        relay_spool_file,
    });
    let request_budget_shared = runtime_state.request_budget.clone();
    let timeout_reached = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now() + Duration::from_millis(config.idle_timeout_ms);

    let app = build_service_api_router(runtime_state);

    match tls_mode {
        ServiceApiTlsMode::Disabled => {
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
    fn regression_service_api_tls_mode_missing_env_allows_loopback_default_only() {
        // Regression: #6069
        let loopback_mode = resolve_service_api_tls_mode_from_env(
            Err(env::VarError::NotPresent),
            "127.0.0.1:34079",
        )
        .expect("loopback bind should keep dev default");
        assert!(matches!(loopback_mode, ServiceApiTlsMode::Disabled));
    }

    #[test]
    fn regression_service_api_tls_mode_missing_env_rejects_non_loopback_bind() {
        // Regression: #6069
        let error =
            resolve_service_api_tls_mode_from_env(Err(env::VarError::NotPresent), "0.0.0.0:34079")
                .expect_err("non-loopback bind must require explicit TLS mode");
        assert!(error.contains(SERVICE_API_TLS_MODE_ENV));
        assert!(error.contains("non-loopback"));
    }

    #[test]
    fn unit_service_api_auth_did_key_map_resolution_parses_valid_json_map() {
        let resolved = resolve_service_api_auth_public_keys_by_did_from_env(Ok(
            r#"{"kamn:did:agent:alice":"02aa","kamn:did:agent:bob":"03bb"}"#.to_owned(),
        ))
        .expect("valid did key map should resolve");
        let map = resolved.expect("did key map should be present");
        assert_eq!(map.get("kamn:did:agent:alice"), Some(&"02aa".to_owned()));
        assert_eq!(map.get("kamn:did:agent:bob"), Some(&"03bb".to_owned()));
    }

    #[test]
    fn unit_service_api_auth_did_key_map_resolution_rejects_invalid_sender_did() {
        let error = resolve_service_api_auth_public_keys_by_did_from_env(Ok(
            r#"{"not-a-did":"02aa"}"#.to_owned(),
        ))
        .expect_err("invalid sender did must fail did key map resolution");
        assert!(error.contains(SERVICE_API_AUTH_PUBLIC_KEYS_BY_DID_JSON_ENV));
        assert!(error.contains("invalid sender did"));
    }
}

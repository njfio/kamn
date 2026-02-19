use super::*;

pub(super) fn resolve_service_api_tls_mode() -> Result<ServiceApiTlsMode, String> {
    match env::var(SERVICE_API_TLS_MODE_ENV) {
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
        Err(env::VarError::NotPresent) => Ok(ServiceApiTlsMode::Disabled),
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "service api tls mode env must be utf-8: {SERVICE_API_TLS_MODE_ENV}"
        )),
    }
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
    let tls_mode = resolve_service_api_tls_mode()?;
    let sender_anti_spam = build_service_api_sender_anti_spam_engine()
        .map_err(|error| format!("service api anti-spam init failed: {error}"))?;

    let runtime_state = Arc::new(ServiceApiRuntimeState {
        snapshot,
        replay_guard: Arc::new(Mutex::new(BTreeSet::new())),
        request_budget: Arc::new(ServiceApiRequestBudget::new(config.max_requests)),
        body_limit_bytes: config.body_limit_bytes as usize,
        concurrency_limiter: Arc::new(Semaphore::new(config.concurrency_limit as usize)),
        ingress_rate_window: Arc::new(Mutex::new(ServiceApiIngressRateWindow::new(
            config.rate_limit_per_second,
        ))),
        sender_anti_spam: Arc::new(Mutex::new(sender_anti_spam)),
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

use super::{
    payload_contract::enforce_observability_endpoint_payload_contract,
    payload_render::{
        render_health_body, render_metrics_body, render_readiness_body, render_stream_body,
    },
    render_observability_http_response,
    tls_mode::{resolve_observability_endpoint_tls_mode, ObservabilityEndpointTlsMode},
    ObservabilityEndpointConfig, ObservabilityEndpointPayloadSurface,
    ObservabilityEndpointResponse, RuntimeObservabilitySnapshot,
    DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH, DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH,
};
use axum::{
    extract::State,
    http::{
        header::{CONTENT_LENGTH, TRANSFER_ENCODING},
        HeaderMap, Method, Uri,
    },
    response::Response,
    routing::any,
    Router,
};
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::Instant;

#[derive(Debug)]
struct ObservabilityRequestBudget {
    max_requests: u64,
    served_requests: AtomicU64,
    completion: Notify,
}

impl ObservabilityRequestBudget {
    fn new(max_requests: u64) -> Self {
        Self {
            max_requests,
            served_requests: AtomicU64::new(0),
            completion: Notify::new(),
        }
    }

    fn record_request(&self) {
        let served = self.served_requests.fetch_add(1, Ordering::SeqCst) + 1;
        if served >= self.max_requests {
            self.completion.notify_waiters();
        }
    }

    async fn wait_until_complete(&self) {
        loop {
            if self.served_requests.load(Ordering::SeqCst) >= self.max_requests {
                return;
            }
            self.completion.notified().await;
        }
    }
}

#[derive(Debug)]
struct ObservabilityEndpointRuntimeState {
    snapshot: RuntimeObservabilitySnapshot,
    metrics_path: String,
    health_path: String,
    request_budget: Arc<ObservabilityRequestBudget>,
}

pub(super) async fn serve_observability_endpoint_async(
    config: ObservabilityEndpointConfig,
    snapshot: RuntimeObservabilitySnapshot,
) -> Result<(), String> {
    let tls_mode = resolve_observability_endpoint_tls_mode()?;
    let request_budget = Arc::new(ObservabilityRequestBudget::new(config.max_requests));
    let runtime_state = Arc::new(ObservabilityEndpointRuntimeState {
        snapshot,
        metrics_path: config.metrics_path,
        health_path: config.health_path,
        request_budget: request_budget.clone(),
    });
    let timeout_reached = Arc::new(AtomicBool::new(false));
    let timeout_flag = timeout_reached.clone();
    let deadline = Instant::now() + Duration::from_millis(config.idle_timeout_ms);
    let app = build_observability_endpoint_router(runtime_state);

    match tls_mode {
        ObservabilityEndpointTlsMode::Disabled => {
            let listener = tokio::net::TcpListener::bind(config.bind_addr.as_str())
                .await
                .map_err(|error| format!("observability endpoint bind failed: {error}"))?;
            axum::serve(listener, app.clone())
                .with_graceful_shutdown(async move {
                    let wait_for_budget = request_budget.wait_until_complete();
                    tokio::pin!(wait_for_budget);
                    let idle_timeout = tokio::time::sleep_until(deadline);
                    tokio::pin!(idle_timeout);
                    tokio::select! {
                        _ = &mut wait_for_budget => {}
                        _ = &mut idle_timeout => {
                            timeout_flag.store(true, Ordering::SeqCst);
                        }
                    }
                })
                .await
                .map_err(|error| format!("observability endpoint serve failed: {error}"))?;
        }
        ObservabilityEndpointTlsMode::Require {
            cert_file,
            key_file,
        } => {
            let bind_addr = config.bind_addr.parse::<SocketAddr>().map_err(|error| {
                format!(
                    "observability endpoint tls bind address parse failed: {}: {error}",
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
                    "observability endpoint tls config load failed: cert_file={cert_file}, key_file={key_file}: {error}"
                )
            })?;

            let handle = axum_server::Handle::new();
            let shutdown_handle = handle.clone();
            tokio::spawn(async move {
                let wait_for_budget = request_budget.wait_until_complete();
                tokio::pin!(wait_for_budget);
                let idle_timeout = tokio::time::sleep_until(deadline);
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
                .map_err(|error| format!("observability endpoint tls serve failed: {error}"))?;
        }
    }

    if timeout_reached.load(Ordering::SeqCst) {
        return Err(format!(
            "observability endpoint timed out after {} ms waiting for requests",
            config.idle_timeout_ms
        ));
    }

    Ok(())
}

fn build_observability_endpoint_router(state: Arc<ObservabilityEndpointRuntimeState>) -> Router {
    Router::new()
        .route("/", any(handle_observability_http_route))
        .route("/{*path}", any(handle_observability_http_route))
        .with_state(state)
}

async fn handle_observability_http_route(
    State(state): State<Arc<ObservabilityEndpointRuntimeState>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let response = if method != Method::GET || request_has_non_empty_body(&headers) {
        handle_observability_not_found_path().await
    } else {
        dispatch_observability_endpoint_request(
            &state.snapshot,
            uri.path(),
            state.metrics_path.as_str(),
            state.health_path.as_str(),
            DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH,
            DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH,
        )
        .await
    };
    state.request_budget.record_request();
    render_observability_http_response(response)
}

fn request_has_non_empty_body(headers: &HeaderMap) -> bool {
    if headers.contains_key(TRANSFER_ENCODING) {
        return true;
    }

    match headers.get(CONTENT_LENGTH) {
        Some(content_length) => match content_length.to_str() {
            Ok(value) => value.parse::<u64>().map_or(true, |parsed| parsed > 0),
            Err(_) => true,
        },
        None => false,
    }
}

async fn dispatch_observability_endpoint_request(
    snapshot: &RuntimeObservabilitySnapshot,
    request_path: &str,
    metrics_path: &str,
    health_path: &str,
    readiness_path: &str,
    stream_path: &str,
) -> ObservabilityEndpointResponse {
    if request_path == metrics_path {
        return handle_observability_metrics_path(snapshot).await;
    }
    if request_path == health_path {
        return handle_observability_health_path(snapshot).await;
    }
    if request_path == readiness_path {
        return handle_observability_readiness_path(snapshot).await;
    }
    if request_path == stream_path {
        return handle_observability_stream_path(snapshot).await;
    }
    handle_observability_not_found_path().await
}

async fn handle_observability_metrics_path(
    snapshot: &RuntimeObservabilitySnapshot,
) -> ObservabilityEndpointResponse {
    enforce_observability_endpoint_payload_contract(
        ObservabilityEndpointPayloadSurface::Metrics,
        "text/plain; version=0.0.4",
        render_metrics_body(snapshot),
    )
}

async fn handle_observability_health_path(
    snapshot: &RuntimeObservabilitySnapshot,
) -> ObservabilityEndpointResponse {
    enforce_observability_endpoint_payload_contract(
        ObservabilityEndpointPayloadSurface::Health,
        "application/json",
        render_health_body(snapshot),
    )
}

async fn handle_observability_readiness_path(
    snapshot: &RuntimeObservabilitySnapshot,
) -> ObservabilityEndpointResponse {
    enforce_observability_endpoint_payload_contract(
        ObservabilityEndpointPayloadSurface::Readiness,
        "application/json",
        render_readiness_body(snapshot),
    )
}

async fn handle_observability_stream_path(
    snapshot: &RuntimeObservabilitySnapshot,
) -> ObservabilityEndpointResponse {
    enforce_observability_endpoint_payload_contract(
        ObservabilityEndpointPayloadSurface::Stream,
        "application/x-ndjson",
        render_stream_body(snapshot),
    )
}

async fn handle_observability_not_found_path() -> ObservabilityEndpointResponse {
    ObservabilityEndpointResponse {
        status_code: 404,
        content_type: "text/plain; charset=utf-8",
        body: "not found\n".to_owned(),
    }
}

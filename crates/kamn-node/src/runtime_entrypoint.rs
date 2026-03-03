use kamn_core::ConfigError;

use crate::{
    build_runtime_observability_snapshot, build_service_api_snapshot, log_info,
    serve_observability_endpoint, serve_service_api_endpoint, NodeBootstrapReport,
    ObservabilityEndpointConfig, ServiceApiEndpointConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ServiceApiEndpointRuntimePath {
    SkipForFullSupervisor,
    ServeInProcess,
}

pub(crate) fn classify_service_api_endpoint_runtime_path(
    runtime_mode: &str,
) -> Result<ServiceApiEndpointRuntimePath, ConfigError> {
    if runtime_mode == "full" {
        Ok(ServiceApiEndpointRuntimePath::SkipForFullSupervisor)
    } else if runtime_mode == "api" {
        Ok(ServiceApiEndpointRuntimePath::ServeInProcess)
    } else {
        Err(ConfigError::RuntimeDaemonLifecycle(
            "service api endpoint requires runtime-mode api or full".to_owned(),
        ))
    }
}

pub(crate) fn should_skip_observability_endpoint_for_full_supervisor(runtime_mode: &str) -> bool {
    runtime_mode == "full"
}

pub(crate) fn serve_runtime_endpoints(
    report: &NodeBootstrapReport,
    service_api_endpoint_config: Option<&ServiceApiEndpointConfig>,
    observability_endpoint_config: Option<&ObservabilityEndpointConfig>,
    execution_id: &str,
) -> Result<(), ConfigError> {
    if let Some(endpoint_config) = service_api_endpoint_config {
        serve_service_api_runtime_endpoint(report, endpoint_config, execution_id)?;
    }
    if let Some(endpoint_config) = observability_endpoint_config {
        serve_observability_runtime_endpoint(report, endpoint_config, execution_id)?;
    }
    Ok(())
}

fn serve_service_api_runtime_endpoint(
    report: &NodeBootstrapReport,
    endpoint_config: &ServiceApiEndpointConfig,
    execution_id: &str,
) -> Result<(), ConfigError> {
    match classify_service_api_endpoint_runtime_path(report.runtime_mode.as_str())? {
        ServiceApiEndpointRuntimePath::SkipForFullSupervisor => {
            // Full-mode endpoint lanes are supervised inside runtime orchestration.
            Ok(())
        }
        ServiceApiEndpointRuntimePath::ServeInProcess => {
            let snapshot = build_service_api_snapshot(report);
            let max_requests_label = endpoint_config.max_requests.to_string();
            let idle_timeout_ms_label = endpoint_config.idle_timeout_ms.to_string();
            let body_limit_bytes_label = endpoint_config.body_limit_bytes.to_string();
            let concurrency_limit_label = endpoint_config.concurrency_limit.to_string();
            let rate_limit_per_second_label = endpoint_config.rate_limit_per_second.to_string();
            log_info(
                "node.runtime.service_api.endpoint.start",
                &[
                    ("bind_addr", endpoint_config.bind_addr.as_str()),
                    ("max_requests", max_requests_label.as_str()),
                    ("idle_timeout_ms", idle_timeout_ms_label.as_str()),
                    ("body_limit_bytes", body_limit_bytes_label.as_str()),
                    ("concurrency_limit", concurrency_limit_label.as_str()),
                    (
                        "rate_limit_per_second",
                        rate_limit_per_second_label.as_str(),
                    ),
                    ("execution_id", execution_id),
                ],
            )?;
            serve_service_api_endpoint(endpoint_config, &snapshot)
                .map_err(ConfigError::RuntimeDaemonLifecycle)?;
            log_info(
                "node.runtime.service_api.endpoint.complete",
                &[
                    ("bind_addr", endpoint_config.bind_addr.as_str()),
                    ("execution_id", execution_id),
                ],
            )?;
            Ok(())
        }
    }
}

fn serve_observability_runtime_endpoint(
    report: &NodeBootstrapReport,
    endpoint_config: &ObservabilityEndpointConfig,
    execution_id: &str,
) -> Result<(), ConfigError> {
    if should_skip_observability_endpoint_for_full_supervisor(report.runtime_mode.as_str()) {
        // Full-mode endpoint lanes are supervised inside runtime orchestration.
        return Ok(());
    }

    let snapshot = build_runtime_observability_snapshot(report).ok_or_else(|| {
        ConfigError::RuntimeDaemonLifecycle(
            "observability endpoint export requires daemon or kolme-live telemetry".to_owned(),
        )
    })?;
    let max_requests_label = endpoint_config.max_requests.to_string();
    let idle_timeout_ms_label = endpoint_config.idle_timeout_ms.to_string();
    log_info(
        "node.runtime.observability.endpoint.start",
        &[
            ("bind_addr", endpoint_config.bind_addr.as_str()),
            ("metrics_path", endpoint_config.metrics_path.as_str()),
            ("health_path", endpoint_config.health_path.as_str()),
            ("max_requests", max_requests_label.as_str()),
            ("idle_timeout_ms", idle_timeout_ms_label.as_str()),
            ("execution_id", execution_id),
        ],
    )?;
    serve_observability_endpoint(endpoint_config, &snapshot)
        .map_err(ConfigError::RuntimeDaemonLifecycle)?;
    log_info(
        "node.runtime.observability.endpoint.complete",
        &[
            ("bind_addr", endpoint_config.bind_addr.as_str()),
            ("execution_id", execution_id),
        ],
    )?;
    Ok(())
}

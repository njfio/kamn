use super::{
    payload_render::escape_json_string, ObservabilityEndpointPayloadSurface,
    ObservabilityEndpointResponse, OBSERVABILITY_ENDPOINT_FAIL_CLOSED_SCHEMA_VERSION,
    OBSERVABILITY_ENDPOINT_REASON_TAXONOMY_VERSION,
    OBSERVABILITY_ENDPOINT_REQUIRED_FIELD_MISSING_REASON_PREFIX,
    OBSERVABILITY_ENDPOINT_SCHEMA_DRIFT_REASON_PREFIX, OBSERVABILITY_HEALTH_SCHEMA_VERSION,
    OBSERVABILITY_READINESS_SCHEMA_VERSION, OBSERVABILITY_STREAM_SCHEMA_VERSION,
};

pub(crate) fn enforce_observability_endpoint_payload_contract(
    surface: ObservabilityEndpointPayloadSurface,
    content_type: &'static str,
    payload: String,
) -> ObservabilityEndpointResponse {
    match validate_observability_endpoint_payload_contract(surface, payload.as_str()) {
        Ok(()) => ObservabilityEndpointResponse {
            status_code: 200,
            content_type,
            body: payload,
        },
        Err(reason_code) => ObservabilityEndpointResponse {
            status_code: 503,
            content_type: "application/json",
            body: render_observability_endpoint_fail_closed_body(surface, reason_code.as_str()),
        },
    }
}

pub(crate) fn validate_observability_endpoint_payload_contract(
    surface: ObservabilityEndpointPayloadSurface,
    payload: &str,
) -> Result<(), String> {
    match surface {
        ObservabilityEndpointPayloadSurface::Metrics => {
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_latency_p50_ms ",
                "kamn_observability_latency_p50_ms",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_latency_p99_ms ",
                "kamn_observability_latency_p99_ms",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_throughput_tps ",
                "kamn_observability_throughput_tps",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_error_rate_bps ",
                "kamn_observability_error_rate_bps",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_availability_bps ",
                "kamn_observability_availability_bps",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_alert_count ",
                "kamn_observability_alert_count",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_transport_checkpoint_failures ",
                "kamn_observability_transport_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_signer_checkpoint_failures ",
                "kamn_observability_signer_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_commit_checkpoint_failures ",
                "kamn_observability_commit_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_ready ",
                "kamn_observability_ready",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_source{source=\"",
                "kamn_observability_source",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_runtime_mode{runtime_mode=\"",
                "kamn_observability_runtime_mode",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_reason_code{reason_code=\"",
                "kamn_observability_reason_code",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_readiness_reason_code{readiness_reason_code=\"",
                "kamn_observability_readiness_reason_code",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_transport_dependency_status{status=\"",
                "kamn_observability_transport_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_signer_dependency_status{status=\"",
                "kamn_observability_signer_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_commit_dependency_status{status=\"",
                "kamn_observability_commit_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "kamn_observability_health{health=\"",
                "kamn_observability_health",
            )?;
            Ok(())
        }
        ObservabilityEndpointPayloadSurface::Health => {
            validate_json_schema_version(
                surface,
                payload,
                OBSERVABILITY_HEALTH_SCHEMA_VERSION,
                "schema_version",
            )?;
            validate_required_field_marker(surface, payload, "\"source\":", "source")?;
            validate_required_field_marker(surface, payload, "\"runtime_mode\":", "runtime_mode")?;
            validate_required_field_marker(surface, payload, "\"health\":", "health")?;
            validate_required_field_marker(surface, payload, "\"alert_count\":", "alert_count")?;
            validate_required_field_marker(surface, payload, "\"reason_code\":", "reason_code")?;
            validate_required_field_marker(surface, payload, "\"ready\":", "ready")?;
            validate_required_field_marker(
                surface,
                payload,
                "\"readiness_reason_code\":",
                "readiness_reason_code",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"readiness_reason_taxonomy_version\":",
                "readiness_reason_taxonomy_version",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"transport_dependency_status\":",
                "transport_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"signer_dependency_status\":",
                "signer_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"commit_dependency_status\":",
                "commit_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"transport_checkpoint_failures\":",
                "transport_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"signer_checkpoint_failures\":",
                "signer_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"commit_checkpoint_failures\":",
                "commit_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"latency_p50_ms\":",
                "latency_p50_ms",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"latency_p99_ms\":",
                "latency_p99_ms",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"throughput_tps\":",
                "throughput_tps",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"error_rate_bps\":",
                "error_rate_bps",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"availability_bps\":",
                "availability_bps",
            )?;
            Ok(())
        }
        ObservabilityEndpointPayloadSurface::Readiness => {
            validate_json_schema_version(
                surface,
                payload,
                OBSERVABILITY_READINESS_SCHEMA_VERSION,
                "schema_version",
            )?;
            validate_required_field_marker(surface, payload, "\"source\":", "source")?;
            validate_required_field_marker(surface, payload, "\"runtime_mode\":", "runtime_mode")?;
            validate_required_field_marker(surface, payload, "\"ready\":", "ready")?;
            validate_required_field_marker(surface, payload, "\"health\":", "health")?;
            validate_required_field_marker(surface, payload, "\"reason_code\":", "reason_code")?;
            validate_required_field_marker(
                surface,
                payload,
                "\"readiness_reason_code\":",
                "readiness_reason_code",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"readiness_reason_taxonomy_version\":",
                "readiness_reason_taxonomy_version",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"transport_dependency_status\":",
                "transport_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"signer_dependency_status\":",
                "signer_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"commit_dependency_status\":",
                "commit_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"transport_checkpoint_failures\":",
                "transport_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"signer_checkpoint_failures\":",
                "signer_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"commit_checkpoint_failures\":",
                "commit_checkpoint_failures",
            )?;
            Ok(())
        }
        ObservabilityEndpointPayloadSurface::Stream => {
            validate_json_schema_version(
                surface,
                payload,
                OBSERVABILITY_STREAM_SCHEMA_VERSION,
                "schema_version",
            )?;
            validate_required_field_marker(surface, payload, "\"source\":", "source")?;
            validate_required_field_marker(surface, payload, "\"runtime_mode\":", "runtime_mode")?;
            validate_required_field_marker(surface, payload, "\"health\":", "health")?;
            validate_required_field_marker(surface, payload, "\"alert_count\":", "alert_count")?;
            validate_required_field_marker(surface, payload, "\"reason_code\":", "reason_code")?;
            validate_required_field_marker(surface, payload, "\"ready\":", "ready")?;
            validate_required_field_marker(
                surface,
                payload,
                "\"readiness_reason_code\":",
                "readiness_reason_code",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"transport_dependency_status\":",
                "transport_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"signer_dependency_status\":",
                "signer_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"commit_dependency_status\":",
                "commit_dependency_status",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"transport_checkpoint_failures\":",
                "transport_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"signer_checkpoint_failures\":",
                "signer_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"commit_checkpoint_failures\":",
                "commit_checkpoint_failures",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"latency_p50_ms\":",
                "latency_p50_ms",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"latency_p99_ms\":",
                "latency_p99_ms",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"throughput_tps\":",
                "throughput_tps",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"error_rate_bps\":",
                "error_rate_bps",
            )?;
            validate_required_field_marker(
                surface,
                payload,
                "\"availability_bps\":",
                "availability_bps",
            )?;
            Ok(())
        }
    }
}

fn validate_json_schema_version(
    surface: ObservabilityEndpointPayloadSurface,
    payload: &str,
    expected_schema_version: &str,
    field_name: &str,
) -> Result<(), String> {
    validate_required_field_marker(surface, payload, "\"schema_version\":", "schema_version")?;
    let schema_marker = format!("\"schema_version\":\"{expected_schema_version}\"");
    if payload.contains(schema_marker.as_str()) {
        return Ok(());
    }
    Err(schema_drift_reason_code(surface, field_name))
}

fn validate_required_field_marker(
    surface: ObservabilityEndpointPayloadSurface,
    payload: &str,
    marker: &str,
    field_name: &str,
) -> Result<(), String> {
    if payload.contains(marker) {
        return Ok(());
    }
    Err(required_field_missing_reason_code(surface, field_name))
}

fn required_field_missing_reason_code(
    surface: ObservabilityEndpointPayloadSurface,
    field_name: &str,
) -> String {
    format!(
        "{}:{}.{}",
        OBSERVABILITY_ENDPOINT_REQUIRED_FIELD_MISSING_REASON_PREFIX,
        surface.reason_surface(),
        field_name
    )
}

fn schema_drift_reason_code(
    surface: ObservabilityEndpointPayloadSurface,
    field_name: &str,
) -> String {
    format!(
        "{}:{}.{}",
        OBSERVABILITY_ENDPOINT_SCHEMA_DRIFT_REASON_PREFIX,
        surface.reason_surface(),
        field_name
    )
}

fn render_observability_endpoint_fail_closed_body(
    surface: ObservabilityEndpointPayloadSurface,
    reason_code: &str,
) -> String {
    format!(
        "{{\"schema_version\":\"{}\",\"status\":\"fail_closed\",\"final_decision\":\"NO-GO\",\"surface\":\"{}\",\"reason_taxonomy_version\":\"{}\",\"reason_code\":\"{}\"}}",
        OBSERVABILITY_ENDPOINT_FAIL_CLOSED_SCHEMA_VERSION,
        surface.reason_surface(),
        OBSERVABILITY_ENDPOINT_REASON_TAXONOMY_VERSION,
        escape_json_string(reason_code),
    )
}

use super::*;

pub(super) fn render_service_api_endpoint_response(
    snapshot: &ServiceApiSnapshot,
    method: &str,
    path: &str,
    body: &str,
) -> ServiceApiEndpointResponse {
    if method == "GET" && path == ROUTE_HEALTHZ {
        let payload = ServiceApiHealthBody {
            status: "ok".to_owned(),
            runtime_mode: snapshot.runtime_mode.clone(),
            role: snapshot.role.clone(),
            observability_source: snapshot.observability_source.clone(),
            observability_health: snapshot.observability_health.clone(),
        };
        return ServiceApiEndpointResponse {
            status_code: 200,
            content_type: "application/json",
            body: serialize_service_api_json(&payload),
        };
    }
    if method == "GET" && path == ROUTE_METRICS {
        let health_value = if snapshot.observability_health == "healthy" {
            1
        } else {
            0
        };
        let metrics = format!(
            "kamn_service_api_health{{runtime_mode=\"{}\"}} 1\nkamn_service_api_role{{role=\"{}\"}} 1\nkamn_service_api_chain_info{{chain_id=\"{}\",chain_version=\"{}\"}} 1\nkamn_service_api_cross_store_replay_reason_taxonomy_info{{version=\"{}\"}} 1\nkamn_service_api_cross_store_replay_reason_code_count {}\nkamn_service_api_auth_reason_taxonomy_info{{version=\"{}\"}} 1\nkamn_service_api_auth_reason_code_count {}\nkamn_service_api_scope_policy_reason_taxonomy_info{{version=\"{}\"}} 1\nkamn_service_api_scope_policy_reason_code_count {}\nkamn_service_api_scope_policy_fixture_schema_info{{version=\"{}\"}} 1\nkamn_service_api_scope_policy_fixture_reason_taxonomy_info{{version=\"{}\"}} 1\nkamn_service_api_scope_policy_fixture_reason_code_count {}\nkamn_service_api_scope_policy_fixture_row_count {}\nkamn_service_api_scope_policy_fixture_allow_row_count {}\nkamn_service_api_scope_policy_fixture_deny_row_count {}\nkamn_service_api_scope_policy_fixture_unique_route_count {}\nkamn_service_api_scope_policy_fixture_unique_scope_count {}\nkamn_service_api_scope_policy_fixture_unique_method_count {}\nkamn_service_api_scope_policy_fixture_unique_expected_outcome_count {}\nkamn_service_api_route_authz_matrix_schema_info{{version=\"{}\"}} 1\nkamn_service_api_route_authz_matrix_total_route_count {}\nkamn_service_api_route_authz_matrix_public_route_count {}\nkamn_service_api_route_authz_matrix_protected_route_count {}\nkamn_service_api_lifecycle_rejection_reason_taxonomy_info{{version=\"{}\"}} 1\nkamn_service_api_lifecycle_rejection_reason_code_count {}\nkamn_service_api_websocket_reason_taxonomy_info{{version=\"{}\"}} 1\nkamn_service_api_websocket_reason_code_count {}\nkamn_service_api_observability_latency_p50_ms {}\nkamn_service_api_observability_latency_p99_ms {}\nkamn_service_api_observability_throughput_tps {}\nkamn_service_api_observability_error_rate_bps {}\nkamn_service_api_observability_availability_bps {}\nkamn_service_api_observability_alert_count {}\nkamn_service_api_observability_source{{source=\"{}\"}} 1\nkamn_service_api_observability_health{{health=\"{}\"}} {}\n",
            escape_metrics_label(snapshot.runtime_mode.as_str()),
            escape_metrics_label(snapshot.role.as_str()),
            escape_metrics_label(snapshot.chain_id.as_str()),
            escape_metrics_label(snapshot.chain_version.as_str()),
            escape_metrics_label(snapshot.cross_store_replay_reason_taxonomy_version.as_str()),
            snapshot.cross_store_replay_reason_code_count,
            escape_metrics_label(snapshot.auth_reason_taxonomy_version.as_str()),
            snapshot.auth_reason_code_count,
            escape_metrics_label(snapshot.scope_policy_reason_taxonomy_version.as_str()),
            snapshot.scope_policy_reason_code_count,
            escape_metrics_label(SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION),
            escape_metrics_label(snapshot.scope_policy_fixture_reason_taxonomy_version.as_str()),
            snapshot.scope_policy_fixture_reason_code_count,
            snapshot.scope_policy_fixture_row_count,
            snapshot.scope_policy_fixture_allow_row_count,
            snapshot.scope_policy_fixture_deny_row_count,
            snapshot.scope_policy_fixture_unique_route_count,
            snapshot.scope_policy_fixture_unique_scope_count,
            snapshot.scope_policy_fixture_unique_method_count,
            snapshot.scope_policy_fixture_unique_expected_outcome_count,
            escape_metrics_label(snapshot.route_authz_matrix_schema_version.as_str()),
            snapshot.route_authz_matrix_total_route_count,
            snapshot.route_authz_matrix_public_route_count,
            snapshot.route_authz_matrix_protected_route_count,
            escape_metrics_label(
                snapshot
                    .lifecycle_rejection_reason_taxonomy_version
                    .as_str(),
            ),
            snapshot.lifecycle_rejection_reason_code_count,
            escape_metrics_label(snapshot.websocket_reason_taxonomy_version.as_str()),
            snapshot.websocket_reason_code_count,
            snapshot.observability_latency_p50_ms,
            snapshot.observability_latency_p99_ms,
            snapshot.observability_throughput_tps,
            snapshot.observability_error_rate_bps,
            snapshot.observability_availability_bps,
            snapshot.observability_alert_count,
            escape_metrics_label(snapshot.observability_source.as_str()),
            escape_metrics_label(snapshot.observability_health.as_str()),
            health_value,
        );
        return ServiceApiEndpointResponse {
            status_code: 200,
            content_type: "text/plain; version=0.0.4",
            body: metrics,
        };
    }
    if method == "GET" && path == ROUTE_EVENTS_WS {
        return json_error_endpoint_response(
            StatusCode::BAD_REQUEST,
            "bad-request",
            REASON_CODE_WEBSOCKET_UPGRADE_REQUIRED,
            "websocket upgrade required",
        );
    }
    if method == "POST" && path == ROUTE_MESSAGES_SEND {
        let message_id = format!("msg-local-{}", deterministic_body_tag(body.as_bytes()));
        let payload = ServiceApiMessageCreateBody {
            message_id,
            status: "created".to_owned(),
            runtime_mode: snapshot.runtime_mode.clone(),
        };
        return ServiceApiEndpointResponse {
            status_code: 202,
            content_type: "application/json",
            body: serialize_service_api_json(&payload),
        };
    }
    if method == "POST" && path == ROUTE_CHANNELS_CREATE {
        let channel_id = format!("channel-local-{}", deterministic_body_tag(body.as_bytes()));
        let payload = ServiceApiChannelCreateBody {
            channel_id,
            status: "created".to_owned(),
        };
        return ServiceApiEndpointResponse {
            status_code: 201,
            content_type: "application/json",
            body: serialize_service_api_json(&payload),
        };
    }
    if method == "POST" && path == ROUTE_TASKS_CREATE {
        let task_id = format!("task-local-{}", deterministic_body_tag(body.as_bytes()));
        let payload = ServiceApiTaskCreateBody {
            task_id,
            state: "submitted".to_owned(),
        };
        return ServiceApiEndpointResponse {
            status_code: 201,
            content_type: "application/json",
            body: serialize_service_api_json(&payload),
        };
    }
    if method == "GET" {
        if let Some(message_id) = message_path_id(path) {
            let payload = ServiceApiMessageGetBody {
                message_id: message_id.to_owned(),
                status: "created".to_owned(),
            };
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: serialize_service_api_json(&payload),
            };
        }
        if let Some(channel_id) = channel_messages_path_id(path) {
            let payload = ServiceApiChannelMessagesBody {
                channel_id: channel_id.to_owned(),
                messages: Vec::new(),
            };
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: serialize_service_api_json(&payload),
            };
        }
        if let Some(task_id) = task_path_id(path) {
            let payload = ServiceApiTaskGetBody {
                task_id: task_id.to_owned(),
                state: "submitted".to_owned(),
            };
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: serialize_service_api_json(&payload),
            };
        }
        if let Some(agent_did) = agent_path_id(path) {
            let payload = ServiceApiAgentGetBody {
                did: agent_did.to_owned(),
                reputation_score: 500,
            };
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: serialize_service_api_json(&payload),
            };
        }
    }

    if route_exists_for_other_method(path) {
        return json_error_endpoint_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "method-not-allowed",
            REASON_CODE_METHOD_NOT_ALLOWED,
            "method not allowed",
        );
    }
    json_error_endpoint_response(
        StatusCode::NOT_FOUND,
        "not-found",
        REASON_CODE_ROUTE_NOT_FOUND,
        "not found",
    )
}

pub(super) fn route_exists_for_other_method(path: &str) -> bool {
    path == ROUTE_MESSAGES_SEND
        || path == ROUTE_CHANNELS_CREATE
        || path == ROUTE_TASKS_CREATE
        || path == ROUTE_EVENTS_WS
        || path == ROUTE_HEALTHZ
        || path == ROUTE_METRICS
        || message_path_id(path).is_some()
        || channel_messages_path_id(path).is_some()
        || task_path_id(path).is_some()
        || agent_path_id(path).is_some()
}

pub(super) fn message_path_id(path: &str) -> Option<&str> {
    path.strip_prefix(ROUTE_MESSAGES_PREFIX).and_then(|id| {
        if id.is_empty() || id == "send" || id.contains('/') {
            return None;
        }
        Some(id)
    })
}

pub(super) fn channel_messages_path_id(path: &str) -> Option<&str> {
    let channel_path = path.strip_prefix(ROUTE_CHANNELS_PREFIX)?;
    let channel_id = channel_path.strip_suffix(ROUTE_CHANNELS_MESSAGES_SUFFIX)?;
    if channel_id.is_empty() || channel_id.contains('/') {
        return None;
    }
    Some(channel_id)
}

pub(super) fn task_path_id(path: &str) -> Option<&str> {
    path.strip_prefix(ROUTE_TASKS_PREFIX).and_then(|id| {
        if id.is_empty() || id == "create" || id.contains('/') {
            return None;
        }
        Some(id)
    })
}

pub(super) fn agent_path_id(path: &str) -> Option<&str> {
    path.strip_prefix(ROUTE_AGENTS_PREFIX).and_then(|did| {
        if did.is_empty() || did.contains('/') {
            return None;
        }
        Some(did)
    })
}

pub(super) fn contract_response(response: ServiceApiEndpointResponse) -> Response {
    let status =
        StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        [("Content-Type", response.content_type)],
        response.body,
    )
        .into_response()
}

pub(super) fn json_error_endpoint_response(
    status_code: StatusCode,
    error: &str,
    reason_code: &str,
    message: &str,
) -> ServiceApiEndpointResponse {
    ServiceApiEndpointResponse {
        status_code: status_code.as_u16(),
        content_type: "application/json",
        body: serialize_service_api_json(&ServiceApiErrorBody {
            error: error.to_owned(),
            reason_code: reason_code.to_owned(),
            message: message.to_owned(),
        }),
    }
}

pub(super) fn json_error_response(
    status_code: StatusCode,
    error: &str,
    reason_code: &str,
    message: &str,
) -> Response {
    let payload = ServiceApiErrorBody {
        error: error.to_owned(),
        reason_code: reason_code.to_owned(),
        message: message.to_owned(),
    };
    (
        status_code,
        [("Content-Type", "application/json")],
        serialize_service_api_json(&payload),
    )
        .into_response()
}

#[cfg(test)]
pub(crate) fn parse_service_api_payload<T: DeserializeOwned>(payload: &str) -> Result<T, String> {
    serde_json::from_str(payload).map_err(|error| {
        format!(
            "{}:{}",
            service_api_payload_decode_reason_code(&error),
            error
        )
    })
}

#[cfg(test)]
pub(crate) fn service_api_payload_decode_reason_code(error: &serde_json::Error) -> &'static str {
    use serde_json::error::Category;
    match error.classify() {
        Category::Io => "service_api_payload_io_error",
        Category::Syntax | Category::Eof => "service_api_payload_json_syntax_invalid",
        Category::Data => "service_api_payload_structure_invalid",
    }
}

pub(super) fn serialize_service_api_json<T: Serialize>(payload: &T) -> String {
    serde_json::to_string(payload).unwrap_or_else(|error| {
        format!(
            "{{\"error\":\"internal\",\"reason_code\":\"service_api_payload_serialization_failed\",\"message\":\"service api payload serialization failed: {}\"}}",
            escape_json_string(error.to_string().as_str())
        )
    })
}

pub(super) fn deterministic_body_tag(payload: &[u8]) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in payload {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(*byte);
    }
    acc
}

pub(super) fn escape_json_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(super) fn escape_metrics_label(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

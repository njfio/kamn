use super::{
    RuntimeObservabilitySnapshot, OBSERVABILITY_HEALTH_SCHEMA_VERSION,
    OBSERVABILITY_READINESS_REASON_TAXONOMY_VERSION, OBSERVABILITY_READINESS_SCHEMA_VERSION,
    OBSERVABILITY_STREAM_SCHEMA_VERSION,
};

pub(super) fn render_metrics_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let health_value = if snapshot.health == "healthy" { 1 } else { 0 };
    let ready_value = if is_runtime_ready(snapshot) { 1 } else { 0 };
    let readiness_reason_code = readiness_reason_code(snapshot);
    let transport_status = transport_dependency_status(snapshot);
    let signer_status = signer_dependency_status(snapshot);
    let commit_status = commit_dependency_status(snapshot);
    format!(
        "kamn_observability_latency_p50_ms {}\nkamn_observability_latency_p99_ms {}\nkamn_observability_throughput_tps {}\nkamn_observability_error_rate_bps {}\nkamn_observability_availability_bps {}\nkamn_observability_alert_count {}\nkamn_observability_transport_checkpoint_failures {}\nkamn_observability_signer_checkpoint_failures {}\nkamn_observability_commit_checkpoint_failures {}\nkamn_observability_ready {}\nkamn_observability_source{{source=\"{}\"}} 1\nkamn_observability_runtime_mode{{runtime_mode=\"{}\"}} 1\nkamn_observability_reason_code{{reason_code=\"{}\"}} 1\nkamn_observability_readiness_reason_code{{readiness_reason_code=\"{}\"}} 1\nkamn_observability_transport_dependency_status{{status=\"{}\"}} 1\nkamn_observability_signer_dependency_status{{status=\"{}\"}} 1\nkamn_observability_commit_dependency_status{{status=\"{}\"}} 1\nkamn_observability_health{{health=\"{}\"}} {}\n",
        snapshot.latency_p50_ms,
        snapshot.latency_p99_ms,
        snapshot.throughput_tps,
        snapshot.error_rate_bps,
        snapshot.availability_bps,
        snapshot.alert_count,
        snapshot.transport_checkpoint_failures,
        snapshot.signer_checkpoint_failures,
        snapshot.commit_checkpoint_failures,
        ready_value,
        escape_metrics_label(snapshot.source.as_str()),
        escape_metrics_label(snapshot.runtime_mode.as_str()),
        escape_metrics_label(snapshot.reason_code.as_str()),
        escape_metrics_label(readiness_reason_code),
        escape_metrics_label(transport_status),
        escape_metrics_label(signer_status),
        escape_metrics_label(commit_status),
        escape_metrics_label(snapshot.health.as_str()),
        health_value
    )
}

pub(super) fn render_health_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let readiness_reason_code = readiness_reason_code(snapshot);
    format!(
        "{{\"schema_version\":\"{}\",\"source\":\"{}\",\"runtime_mode\":\"{}\",\"health\":\"{}\",\"alert_count\":{},\"reason_code\":\"{}\",\"ready\":{},\"readiness_reason_code\":\"{}\",\"readiness_reason_taxonomy_version\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{},\"latency_p50_ms\":{},\"latency_p99_ms\":{},\"throughput_tps\":{},\"error_rate_bps\":{},\"availability_bps\":{}}}",
        OBSERVABILITY_HEALTH_SCHEMA_VERSION,
        escape_json_string(snapshot.source.as_str()),
        escape_json_string(snapshot.runtime_mode.as_str()),
        escape_json_string(snapshot.health.as_str()),
        snapshot.alert_count,
        escape_json_string(snapshot.reason_code.as_str()),
        is_runtime_ready(snapshot),
        escape_json_string(readiness_reason_code),
        OBSERVABILITY_READINESS_REASON_TAXONOMY_VERSION,
        transport_dependency_status(snapshot),
        signer_dependency_status(snapshot),
        commit_dependency_status(snapshot),
        snapshot.transport_checkpoint_failures,
        snapshot.signer_checkpoint_failures,
        snapshot.commit_checkpoint_failures,
        snapshot.latency_p50_ms,
        snapshot.latency_p99_ms,
        snapshot.throughput_tps,
        snapshot.error_rate_bps,
        snapshot.availability_bps
    )
}

pub(super) fn render_stream_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let readiness_reason_code = readiness_reason_code(snapshot);
    format!(
        "{{\"schema_version\":\"{}\",\"source\":\"{}\",\"runtime_mode\":\"{}\",\"health\":\"{}\",\"alert_count\":{},\"reason_code\":\"{}\",\"ready\":{},\"readiness_reason_code\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{},\"latency_p50_ms\":{},\"latency_p99_ms\":{},\"throughput_tps\":{},\"error_rate_bps\":{},\"availability_bps\":{}}}\n",
        OBSERVABILITY_STREAM_SCHEMA_VERSION,
        escape_json_string(snapshot.source.as_str()),
        escape_json_string(snapshot.runtime_mode.as_str()),
        escape_json_string(snapshot.health.as_str()),
        snapshot.alert_count,
        escape_json_string(snapshot.reason_code.as_str()),
        is_runtime_ready(snapshot),
        escape_json_string(readiness_reason_code),
        transport_dependency_status(snapshot),
        signer_dependency_status(snapshot),
        commit_dependency_status(snapshot),
        snapshot.transport_checkpoint_failures,
        snapshot.signer_checkpoint_failures,
        snapshot.commit_checkpoint_failures,
        snapshot.latency_p50_ms,
        snapshot.latency_p99_ms,
        snapshot.throughput_tps,
        snapshot.error_rate_bps,
        snapshot.availability_bps
    )
}

pub(super) fn render_readiness_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let readiness_reason_code = readiness_reason_code(snapshot);
    format!(
        "{{\"schema_version\":\"{}\",\"source\":\"{}\",\"runtime_mode\":\"{}\",\"ready\":{},\"health\":\"{}\",\"reason_code\":\"{}\",\"readiness_reason_code\":\"{}\",\"readiness_reason_taxonomy_version\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{}}}",
        OBSERVABILITY_READINESS_SCHEMA_VERSION,
        escape_json_string(snapshot.source.as_str()),
        escape_json_string(snapshot.runtime_mode.as_str()),
        is_runtime_ready(snapshot),
        escape_json_string(snapshot.health.as_str()),
        escape_json_string(snapshot.reason_code.as_str()),
        escape_json_string(readiness_reason_code),
        OBSERVABILITY_READINESS_REASON_TAXONOMY_VERSION,
        transport_dependency_status(snapshot),
        signer_dependency_status(snapshot),
        commit_dependency_status(snapshot),
        snapshot.transport_checkpoint_failures,
        snapshot.signer_checkpoint_failures,
        snapshot.commit_checkpoint_failures
    )
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

fn is_runtime_ready(snapshot: &RuntimeObservabilitySnapshot) -> bool {
    snapshot.transport_checkpoint_failures == 0
        && snapshot.signer_checkpoint_failures == 0
        && snapshot.commit_checkpoint_failures == 0
        && snapshot.health == "healthy"
}

fn readiness_reason_code(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {
    if snapshot.transport_checkpoint_failures > 0 {
        "readiness_transport_dependency_unhealthy"
    } else if snapshot.signer_checkpoint_failures > 0 {
        "readiness_signer_dependency_unhealthy"
    } else if snapshot.commit_checkpoint_failures > 0 {
        "readiness_commit_dependency_unhealthy"
    } else if snapshot.health != "healthy" {
        "readiness_runtime_health_degraded"
    } else {
        "none"
    }
}

fn transport_dependency_status(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {
    if snapshot.transport_checkpoint_failures > 0 {
        "degraded"
    } else {
        "ready"
    }
}

fn signer_dependency_status(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {
    if snapshot.signer_checkpoint_failures > 0 {
        "degraded"
    } else {
        "ready"
    }
}

fn commit_dependency_status(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {
    if snapshot.commit_checkpoint_failures > 0 {
        "degraded"
    } else {
        "ready"
    }
}

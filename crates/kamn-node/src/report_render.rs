use crate::{NodeBootstrapReport, OutputMode, OutputModeKind};

pub(crate) fn render_bootstrap_report(report: &NodeBootstrapReport, mode: OutputMode) -> String {
    match mode.kind {
        OutputModeKind::Text => render_text_report(report),
        OutputModeKind::Json => render_json_report(report),
    }
}

fn render_text_report(report: &NodeBootstrapReport) -> String {
    let profile = report.profile.as_deref().unwrap_or("none");
    let planning_expected_state_hash = report
        .planning_expected_state_hash
        .as_deref()
        .unwrap_or("none");
    let planning_candidate_count = report
        .planning_candidate_count
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let planning_scheduled_candidate_ids = report
        .planning_scheduled_candidate_ids
        .as_ref()
        .map(|value| value.join(", "))
        .unwrap_or_else(|| "none".to_owned());
    let recovery_expected_state_version = report
        .recovery_expected_state_version
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let recovery_expected_state_hash = report
        .recovery_expected_state_hash
        .as_deref()
        .unwrap_or("none");
    let recovery_attempt_count = report
        .recovery_attempt_count
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let recovery_decisions = report
        .recovery_decisions
        .as_ref()
        .map(|value| value.join(", "))
        .unwrap_or_else(|| "none".to_owned());
    let daemon_max_ticks = report
        .daemon_max_ticks
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_tick_interval_ms = report
        .daemon_tick_interval_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_executed_ticks = report
        .daemon_executed_ticks
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_completion_reason = report.daemon_completion_reason.as_deref().unwrap_or("none");
    let daemon_observability_latency_p50_ms = report
        .daemon_observability_latency_p50_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_observability_latency_p99_ms = report
        .daemon_observability_latency_p99_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_observability_throughput_tps = report
        .daemon_observability_throughput_tps
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_observability_error_rate_bps = report
        .daemon_observability_error_rate_bps
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_observability_availability_bps = report
        .daemon_observability_availability_bps
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_observability_health = report
        .daemon_observability_health
        .as_deref()
        .unwrap_or("none");
    let daemon_observability_alert_count = report
        .daemon_observability_alert_count
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let daemon_peer_id = report.daemon_peer_id.as_deref().unwrap_or("none");
    let daemon_peer_lifecycle_final_state = report
        .daemon_peer_lifecycle_final_state
        .as_deref()
        .unwrap_or("none");
    let daemon_peer_lifecycle_applied_events = report
        .daemon_peer_lifecycle_applied_events
        .as_ref()
        .map(|value| value.join(", "))
        .unwrap_or_else(|| "none".to_owned());
    let kolme_live_provider_client_contract = report
        .kolme_live_provider_client_contract
        .as_deref()
        .unwrap_or("none");
    let kolme_live_base_url = report.kolme_live_base_url.as_deref().unwrap_or("none");
    let kolme_live_provider_hint = report.kolme_live_provider_hint.as_deref().unwrap_or("none");
    let kolme_live_signing_profile = report
        .kolme_live_signing_profile
        .as_deref()
        .unwrap_or("none");
    let kolme_live_signer_profile_selector_env = report
        .kolme_live_signer_profile_selector_env
        .as_deref()
        .unwrap_or("none");
    let kolme_live_signer_profile = report
        .kolme_live_signer_profile
        .as_deref()
        .unwrap_or("none");
    let kolme_live_signer_key_source = report
        .kolme_live_signer_key_source
        .as_deref()
        .unwrap_or("none");
    let kolme_live_signer_private_key_env = report
        .kolme_live_signer_private_key_env
        .as_deref()
        .unwrap_or("none");
    let kolme_live_execution_status = report
        .kolme_live_execution_status
        .as_deref()
        .unwrap_or("none");
    format!(
        "KAMN node bootstrap\n  runtime_mode: {}\n  diagnostics_mode: {}\n  profile: {}\n  role: {}\n  chain: {} ({})\n  storage: {}\n  gossip: {}\n  sync_mode: {}\n  sync_startup: {}\n  sync_recovery: {}\n  state_version: {}\n  pending_migrations: {}\n  component_count: {}\n  planning_expected_state_hash: {}\n  planning_candidate_count: {}\n  planning_scheduled_candidate_ids: {}\n  recovery_expected_state_version: {}\n  recovery_expected_state_hash: {}\n  recovery_attempt_count: {}\n  recovery_decisions: {}\n  daemon_max_ticks: {}\n  daemon_tick_interval_ms: {}\n  daemon_executed_ticks: {}\n  daemon_completion_reason: {}\n  daemon_observability_latency_p50_ms: {}\n  daemon_observability_latency_p99_ms: {}\n  daemon_observability_throughput_tps: {}\n  daemon_observability_error_rate_bps: {}\n  daemon_observability_availability_bps: {}\n  daemon_observability_health: {}\n  daemon_observability_alert_count: {}\n  daemon_peer_id: {}\n  daemon_peer_lifecycle_final_state: {}\n  daemon_peer_lifecycle_applied_events: {}\n  kolme_live_provider_client_contract: {}\n  kolme_live_base_url: {}\n  kolme_live_provider_hint: {}\n  kolme_live_signing_profile: {}\n  kolme_live_signer_profile_selector_env: {}\n  kolme_live_signer_profile: {}\n  kolme_live_signer_key_source: {}\n  kolme_live_signer_private_key_env: {}\n  kolme_live_execution_status: {}\n  components: {}",
        report.runtime_mode,
        report.diagnostics_mode,
        profile,
        report.role,
        report.chain_id,
        report.chain_version,
        report.storage_dir,
        if report.gossip_enabled {
            "enabled"
        } else {
            "disabled"
        },
        report.sync_mode,
        report.sync_startup,
        report.sync_recovery,
        report.state_version,
        report.pending_migrations,
        report.component_count,
        planning_expected_state_hash,
        planning_candidate_count,
        planning_scheduled_candidate_ids,
        recovery_expected_state_version,
        recovery_expected_state_hash,
        recovery_attempt_count,
        recovery_decisions,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_executed_ticks,
        daemon_completion_reason,
        daemon_observability_latency_p50_ms,
        daemon_observability_latency_p99_ms,
        daemon_observability_throughput_tps,
        daemon_observability_error_rate_bps,
        daemon_observability_availability_bps,
        daemon_observability_health,
        daemon_observability_alert_count,
        daemon_peer_id,
        daemon_peer_lifecycle_final_state,
        daemon_peer_lifecycle_applied_events,
        kolme_live_provider_client_contract,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_signer_profile_selector_env,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        kolme_live_signer_private_key_env,
        kolme_live_execution_status,
        report.components.join(", "),
    )
}

fn render_json_report(report: &NodeBootstrapReport) -> String {
    let profile = match &report.profile {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let planning_expected_state_hash = match &report.planning_expected_state_hash {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let planning_candidate_count = match report.planning_candidate_count {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let planning_scheduled_candidate_ids = match &report.planning_scheduled_candidate_ids {
        Some(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| format!("\"{}\"", json_escape(value)))
                .collect::<Vec<String>>()
                .join(",")
        ),
        None => "null".to_owned(),
    };
    let recovery_expected_state_version = match report.recovery_expected_state_version {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let recovery_expected_state_hash = match &report.recovery_expected_state_hash {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let recovery_attempt_count = match report.recovery_attempt_count {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let recovery_decisions = match &report.recovery_decisions {
        Some(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| format!("\"{}\"", json_escape(value)))
                .collect::<Vec<String>>()
                .join(",")
        ),
        None => "null".to_owned(),
    };
    let daemon_max_ticks = match report.daemon_max_ticks {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_tick_interval_ms = match report.daemon_tick_interval_ms {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_executed_ticks = match report.daemon_executed_ticks {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_completion_reason = match &report.daemon_completion_reason {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let daemon_observability_latency_p50_ms = match report.daemon_observability_latency_p50_ms {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_observability_latency_p99_ms = match report.daemon_observability_latency_p99_ms {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_observability_throughput_tps = match report.daemon_observability_throughput_tps {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_observability_error_rate_bps = match report.daemon_observability_error_rate_bps {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_observability_availability_bps = match report.daemon_observability_availability_bps {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_observability_health = match &report.daemon_observability_health {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let daemon_observability_alert_count = match report.daemon_observability_alert_count {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    };
    let daemon_peer_id = match &report.daemon_peer_id {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let daemon_peer_lifecycle_final_state = match &report.daemon_peer_lifecycle_final_state {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let daemon_peer_lifecycle_applied_events = match &report.daemon_peer_lifecycle_applied_events {
        Some(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| format!("\"{}\"", json_escape(value)))
                .collect::<Vec<String>>()
                .join(",")
        ),
        None => "null".to_owned(),
    };
    let kolme_live_provider_client_contract = match &report.kolme_live_provider_client_contract {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_base_url = match &report.kolme_live_base_url {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_provider_hint = match &report.kolme_live_provider_hint {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_signing_profile = match &report.kolme_live_signing_profile {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_signer_profile_selector_env =
        match &report.kolme_live_signer_profile_selector_env {
            Some(value) => format!("\"{}\"", json_escape(value)),
            None => "null".to_owned(),
        };
    let kolme_live_signer_profile = match &report.kolme_live_signer_profile {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_signer_key_source = match &report.kolme_live_signer_key_source {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_signer_private_key_env = match &report.kolme_live_signer_private_key_env {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let kolme_live_execution_status = match &report.kolme_live_execution_status {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    };
    let components = report
        .components
        .iter()
        .map(|component| format!("\"{}\"", json_escape(component)))
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{{\"runtime_mode\":\"{}\",\"diagnostics_mode\":\"{}\",\"profile\":{},\"role\":\"{}\",\"chain_id\":\"{}\",\"chain_version\":\"{}\",\"storage_dir\":\"{}\",\"gossip_enabled\":{},\"sync_mode\":\"{}\",\"sync_startup\":\"{}\",\"sync_recovery\":\"{}\",\"state_version\":{},\"pending_migrations\":{},\"component_count\":{},\"planning_expected_state_hash\":{},\"planning_candidate_count\":{},\"planning_scheduled_candidate_ids\":{},\"recovery_expected_state_version\":{},\"recovery_expected_state_hash\":{},\"recovery_attempt_count\":{},\"recovery_decisions\":{},\"daemon_max_ticks\":{},\"daemon_tick_interval_ms\":{},\"daemon_executed_ticks\":{},\"daemon_completion_reason\":{},\"daemon_observability_latency_p50_ms\":{},\"daemon_observability_latency_p99_ms\":{},\"daemon_observability_throughput_tps\":{},\"daemon_observability_error_rate_bps\":{},\"daemon_observability_availability_bps\":{},\"daemon_observability_health\":{},\"daemon_observability_alert_count\":{},\"daemon_peer_id\":{},\"daemon_peer_lifecycle_final_state\":{},\"daemon_peer_lifecycle_applied_events\":{},\"kolme_live_provider_client_contract\":{},\"kolme_live_base_url\":{},\"kolme_live_provider_hint\":{},\"kolme_live_signing_profile\":{},\"kolme_live_signer_profile_selector_env\":{},\"kolme_live_signer_profile\":{},\"kolme_live_signer_key_source\":{},\"kolme_live_signer_private_key_env\":{},\"kolme_live_execution_status\":{},\"components\":[{}]}}",
        json_escape(&report.runtime_mode),
        json_escape(&report.diagnostics_mode),
        profile,
        json_escape(&report.role),
        json_escape(&report.chain_id),
        json_escape(&report.chain_version),
        json_escape(&report.storage_dir),
        report.gossip_enabled,
        json_escape(&report.sync_mode),
        json_escape(&report.sync_startup),
        json_escape(&report.sync_recovery),
        report.state_version,
        report.pending_migrations,
        report.component_count,
        planning_expected_state_hash,
        planning_candidate_count,
        planning_scheduled_candidate_ids,
        recovery_expected_state_version,
        recovery_expected_state_hash,
        recovery_attempt_count,
        recovery_decisions,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_executed_ticks,
        daemon_completion_reason,
        daemon_observability_latency_p50_ms,
        daemon_observability_latency_p99_ms,
        daemon_observability_throughput_tps,
        daemon_observability_error_rate_bps,
        daemon_observability_availability_bps,
        daemon_observability_health,
        daemon_observability_alert_count,
        daemon_peer_id,
        daemon_peer_lifecycle_final_state,
        daemon_peer_lifecycle_applied_events,
        kolme_live_provider_client_contract,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_signer_profile_selector_env,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        kolme_live_signer_private_key_env,
        kolme_live_execution_status,
        components,
    )
}

fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

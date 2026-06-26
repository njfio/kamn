use crate::NodeBootstrapReport;

use super::support::{gossip_status, text_list, text_opt_list, text_opt_num, text_opt_str};

pub(super) fn render_text_report(report: &NodeBootstrapReport) -> String {
    let lines = collect_text_lines(report);
    format!(
        "KAMN node bootstrap\n{}",
        lines
            .into_iter()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn collect_text_lines(report: &NodeBootstrapReport) -> Vec<String> {
    let mut lines = identity_lines(report);
    lines.extend(sync_state_lines(report));
    lines.extend(planning_recovery_lines(report));
    lines.extend(daemon_lines(report));
    lines.extend(kolme_lines(report));
    lines.push(format!("components: {}", text_list(&report.components)));
    lines
}

fn identity_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!("runtime_mode: {}", report.runtime_mode),
        format!("diagnostics_mode: {}", report.diagnostics_mode),
        format!("profile: {}", text_opt_str(&report.profile)),
        format!("role: {}", report.role),
        format!("chain: {} ({})", report.chain_id, report.chain_version),
        format!("storage: {}", report.storage_dir),
        format!("gossip: {}", gossip_status(report.gossip_enabled)),
    ]
}

fn sync_state_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!("sync_mode: {}", report.sync_mode),
        format!("sync_startup: {}", report.sync_startup),
        format!("sync_recovery: {}", report.sync_recovery),
        format!("state_version: {}", report.state_version),
        format!("pending_migrations: {}", report.pending_migrations),
        format!("component_count: {}", report.component_count),
    ]
}

fn planning_recovery_lines(report: &NodeBootstrapReport) -> Vec<String> {
    let mut lines = planning_lines(report);
    lines.extend(recovery_lines(report));
    lines
}

fn planning_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "planning_expected_state_hash: {}",
            text_opt_str(&report.planning_expected_state_hash)
        ),
        format!(
            "planning_candidate_count: {}",
            text_opt_num(report.planning_candidate_count)
        ),
        format!(
            "planning_scheduled_candidate_ids: {}",
            text_opt_list(&report.planning_scheduled_candidate_ids)
        ),
    ]
}

fn recovery_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "recovery_expected_state_version: {}",
            text_opt_num(report.recovery_expected_state_version)
        ),
        format!(
            "recovery_expected_state_hash: {}",
            text_opt_str(&report.recovery_expected_state_hash)
        ),
        format!(
            "recovery_attempt_count: {}",
            text_opt_num(report.recovery_attempt_count)
        ),
        format!(
            "recovery_decisions: {}",
            text_opt_list(&report.recovery_decisions)
        ),
    ]
}

fn daemon_lines(report: &NodeBootstrapReport) -> Vec<String> {
    let mut lines = daemon_status_lines(report);
    lines.extend(daemon_observability_lines(report));
    lines.extend(daemon_phase6_lines(report));
    lines.extend(daemon_convergence_lines(report));
    lines.extend(daemon_postgres_lines(report));
    lines
}

fn daemon_status_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "daemon_max_ticks: {}",
            text_opt_num(report.daemon_max_ticks)
        ),
        format!(
            "daemon_tick_interval_ms: {}",
            text_opt_num(report.daemon_tick_interval_ms)
        ),
        format!(
            "daemon_executed_ticks: {}",
            text_opt_num(report.daemon_executed_ticks)
        ),
        format!(
            "daemon_completion_reason: {}",
            text_opt_str(&report.daemon_completion_reason)
        ),
        format!("daemon_peer_id: {}", text_opt_str(&report.daemon_peer_id)),
        format!(
            "daemon_peer_lifecycle_final_state: {}",
            text_opt_str(&report.daemon_peer_lifecycle_final_state)
        ),
        format!(
            "daemon_peer_lifecycle_applied_events: {}",
            text_opt_list(&report.daemon_peer_lifecycle_applied_events)
        ),
    ]
}

fn daemon_observability_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "daemon_observability_latency_p50_ms: {}",
            text_opt_num(report.daemon_observability_latency_p50_ms)
        ),
        format!(
            "daemon_observability_latency_p99_ms: {}",
            text_opt_num(report.daemon_observability_latency_p99_ms)
        ),
        format!(
            "daemon_observability_throughput_tps: {}",
            text_opt_num(report.daemon_observability_throughput_tps)
        ),
        format!(
            "daemon_observability_error_rate_bps: {}",
            text_opt_num(report.daemon_observability_error_rate_bps)
        ),
        format!(
            "daemon_observability_availability_bps: {}",
            text_opt_num(report.daemon_observability_availability_bps)
        ),
        format!(
            "daemon_observability_health: {}",
            text_opt_str(&report.daemon_observability_health)
        ),
        format!(
            "daemon_observability_alert_count: {}",
            text_opt_num(report.daemon_observability_alert_count)
        ),
        format!(
            "daemon_observability_reason_code: {}",
            text_opt_str(&report.daemon_observability_reason_code)
        ),
        format!(
            "daemon_observability_transport_checkpoint_failures: {}",
            text_opt_num(report.daemon_observability_transport_checkpoint_failures)
        ),
        format!(
            "daemon_observability_signer_checkpoint_failures: {}",
            text_opt_num(report.daemon_observability_signer_checkpoint_failures)
        ),
        format!(
            "daemon_observability_commit_checkpoint_failures: {}",
            text_opt_num(report.daemon_observability_commit_checkpoint_failures)
        ),
    ]
}

fn daemon_phase6_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "daemon_phase6_runtime_reason_taxonomy_version: {}",
            text_opt_str(&report.daemon_phase6_runtime_reason_taxonomy_version)
        ),
        format!(
            "daemon_phase6_runtime_reason_codes_csv: {}",
            text_opt_str(&report.daemon_phase6_runtime_reason_codes_csv)
        ),
        format!(
            "daemon_phase6_runtime_reason_code: {}",
            text_opt_str(&report.daemon_phase6_runtime_reason_code)
        ),
        format!(
            "daemon_phase6_runtime_total_cycles: {}",
            text_opt_num(report.daemon_phase6_runtime_total_cycles)
        ),
        format!(
            "daemon_phase6_runtime_executed_cycles: {}",
            text_opt_num(report.daemon_phase6_runtime_executed_cycles)
        ),
        format!(
            "daemon_phase6_runtime_deferred_cycles: {}",
            text_opt_num(report.daemon_phase6_runtime_deferred_cycles)
        ),
        format!(
            "daemon_phase6_runtime_fail_closed_cycles: {}",
            text_opt_num(report.daemon_phase6_runtime_fail_closed_cycles)
        ),
    ]
}

fn daemon_convergence_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "daemon_convergence_reason_taxonomy_version: {}",
            text_opt_str(&report.daemon_convergence_reason_taxonomy_version)
        ),
        format!(
            "daemon_convergence_reason_codes_csv: {}",
            text_opt_str(&report.daemon_convergence_reason_codes_csv)
        ),
        format!(
            "daemon_convergence_decision: {}",
            text_opt_str(&report.daemon_convergence_decision)
        ),
        format!(
            "daemon_convergence_reason_code: {}",
            text_opt_str(&report.daemon_convergence_reason_code)
        ),
        format!(
            "daemon_convergence_schema_gate_passed: {}",
            text_opt_num(report.daemon_convergence_schema_gate_passed)
        ),
        format!(
            "daemon_convergence_error_path_gate_passed: {}",
            text_opt_num(report.daemon_convergence_error_path_gate_passed)
        ),
        format!(
            "daemon_convergence_concurrency_gate_passed: {}",
            text_opt_num(report.daemon_convergence_concurrency_gate_passed)
        ),
        format!(
            "daemon_convergence_performance_budget_gate_passed: {}",
            text_opt_num(report.daemon_convergence_performance_budget_gate_passed)
        ),
        format!(
            "daemon_convergence_cost_budget_gate_passed: {}",
            text_opt_num(report.daemon_convergence_cost_budget_gate_passed)
        ),
    ]
}

fn daemon_postgres_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "daemon_live_postgres_multi_host_execution_bundle_schema_version: {}",
            text_opt_str(&report.daemon_live_postgres_multi_host_execution_bundle_schema_version)
        ),
        format!(
            "daemon_live_postgres_multi_host_execution_bundle_selector_prefix: {}",
            text_opt_str(&report.daemon_live_postgres_multi_host_execution_bundle_selector_prefix)
        ),
        format!(
            "daemon_live_postgres_multi_host_execution_bundle_row_count: {}",
            text_opt_num(report.daemon_live_postgres_multi_host_execution_bundle_row_count)
        ),
        format!(
            "daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint: {}",
            text_opt_str(
                &report.daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint
            )
        ),
    ]
}

fn kolme_lines(report: &NodeBootstrapReport) -> Vec<String> {
    let mut lines = kolme_identity_lines(report);
    lines.extend(kolme_observability_lines(report));
    lines
}

fn kolme_identity_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "kolme_live_provider_client_contract: {}",
            text_opt_str(&report.kolme_live_provider_client_contract)
        ),
        format!(
            "kolme_live_base_url: {}",
            text_opt_str(&report.kolme_live_base_url)
        ),
        format!(
            "kolme_live_provider_hint: {}",
            text_opt_str(&report.kolme_live_provider_hint)
        ),
        format!(
            "kolme_live_signing_profile: {}",
            text_opt_str(&report.kolme_live_signing_profile)
        ),
        format!(
            "kolme_live_signer_profile_selector_env: {}",
            text_opt_str(&report.kolme_live_signer_profile_selector_env)
        ),
        format!(
            "kolme_live_signer_profile: {}",
            text_opt_str(&report.kolme_live_signer_profile)
        ),
        format!(
            "kolme_live_signer_key_source: {}",
            text_opt_str(&report.kolme_live_signer_key_source)
        ),
        format!(
            "kolme_live_signer_private_key_env: {}",
            text_opt_str(&report.kolme_live_signer_private_key_env)
        ),
        format!(
            "kolme_live_execution_status: {}",
            text_opt_str(&report.kolme_live_execution_status)
        ),
    ]
}

fn kolme_observability_lines(report: &NodeBootstrapReport) -> Vec<String> {
    vec![
        format!(
            "kolme_live_observability_latency_p50_ms: {}",
            text_opt_num(report.kolme_live_observability_latency_p50_ms)
        ),
        format!(
            "kolme_live_observability_latency_p99_ms: {}",
            text_opt_num(report.kolme_live_observability_latency_p99_ms)
        ),
        format!(
            "kolme_live_observability_throughput_tps: {}",
            text_opt_num(report.kolme_live_observability_throughput_tps)
        ),
        format!(
            "kolme_live_observability_error_rate_bps: {}",
            text_opt_num(report.kolme_live_observability_error_rate_bps)
        ),
        format!(
            "kolme_live_observability_availability_bps: {}",
            text_opt_num(report.kolme_live_observability_availability_bps)
        ),
        format!(
            "kolme_live_observability_health: {}",
            text_opt_str(&report.kolme_live_observability_health)
        ),
        format!(
            "kolme_live_observability_alert_count: {}",
            text_opt_num(report.kolme_live_observability_alert_count)
        ),
        format!(
            "kolme_live_observability_reason_code: {}",
            text_opt_str(&report.kolme_live_observability_reason_code)
        ),
        format!(
            "kolme_live_observability_transport_checkpoint_failures: {}",
            text_opt_num(report.kolme_live_observability_transport_checkpoint_failures)
        ),
        format!(
            "kolme_live_observability_signer_checkpoint_failures: {}",
            text_opt_num(report.kolme_live_observability_signer_checkpoint_failures)
        ),
        format!(
            "kolme_live_observability_commit_checkpoint_failures: {}",
            text_opt_num(report.kolme_live_observability_commit_checkpoint_failures)
        ),
    ]
}

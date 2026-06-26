use crate::NodeBootstrapReport;

use super::support::{json_list, json_opt_list, json_opt_num, json_opt_str, json_str, JsonField};

pub(super) fn render_json_report(report: &NodeBootstrapReport) -> String {
    let fields = collect_json_fields(report);
    format!(
        "{{{}}}",
        fields
            .into_iter()
            .map(render_json_field)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn collect_json_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    let mut fields = identity_fields(report);
    fields.extend(sync_state_fields(report));
    fields.extend(planning_recovery_fields(report));
    fields.extend(daemon_fields(report));
    fields.extend(kolme_fields(report));
    fields.push(("components", json_list(&report.components)));
    fields
}

fn render_json_field((key, value): JsonField) -> String {
    format!("\"{key}\":{value}")
}

fn identity_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        ("runtime_mode", json_str(&report.runtime_mode)),
        ("diagnostics_mode", json_str(&report.diagnostics_mode)),
        ("profile", json_opt_str(&report.profile)),
        ("role", json_str(&report.role)),
        ("chain_id", json_str(&report.chain_id)),
        ("chain_version", json_str(&report.chain_version)),
        ("storage_dir", json_str(&report.storage_dir)),
        ("gossip_enabled", report.gossip_enabled.to_string()),
    ]
}

fn sync_state_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        ("sync_mode", json_str(&report.sync_mode)),
        ("sync_startup", json_str(&report.sync_startup)),
        ("sync_recovery", json_str(&report.sync_recovery)),
        ("state_version", report.state_version.to_string()),
        ("pending_migrations", report.pending_migrations.to_string()),
        ("component_count", report.component_count.to_string()),
    ]
}

fn planning_recovery_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    let mut fields = planning_fields(report);
    fields.extend(recovery_fields(report));
    fields
}

fn planning_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        (
            "planning_expected_state_hash",
            json_opt_str(&report.planning_expected_state_hash),
        ),
        (
            "planning_candidate_count",
            json_opt_num(report.planning_candidate_count),
        ),
        (
            "planning_scheduled_candidate_ids",
            json_opt_list(&report.planning_scheduled_candidate_ids),
        ),
    ]
}

fn recovery_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        (
            "recovery_expected_state_version",
            json_opt_num(report.recovery_expected_state_version),
        ),
        (
            "recovery_expected_state_hash",
            json_opt_str(&report.recovery_expected_state_hash),
        ),
        (
            "recovery_attempt_count",
            json_opt_num(report.recovery_attempt_count),
        ),
        (
            "recovery_decisions",
            json_opt_list(&report.recovery_decisions),
        ),
    ]
}

fn daemon_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    let mut fields = daemon_status_fields(report);
    fields.extend(daemon_observability_fields(report));
    fields.extend(daemon_phase6_fields(report));
    fields.extend(daemon_convergence_fields(report));
    fields.extend(daemon_postgres_fields(report));
    fields
}

fn daemon_status_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        ("daemon_max_ticks", json_opt_num(report.daemon_max_ticks)),
        (
            "daemon_tick_interval_ms",
            json_opt_num(report.daemon_tick_interval_ms),
        ),
        (
            "daemon_executed_ticks",
            json_opt_num(report.daemon_executed_ticks),
        ),
        (
            "daemon_completion_reason",
            json_opt_str(&report.daemon_completion_reason),
        ),
        ("daemon_peer_id", json_opt_str(&report.daemon_peer_id)),
        (
            "daemon_peer_lifecycle_final_state",
            json_opt_str(&report.daemon_peer_lifecycle_final_state),
        ),
        (
            "daemon_peer_lifecycle_applied_events",
            json_opt_list(&report.daemon_peer_lifecycle_applied_events),
        ),
    ]
}

fn daemon_observability_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        (
            "daemon_observability_latency_p50_ms",
            json_opt_num(report.daemon_observability_latency_p50_ms),
        ),
        (
            "daemon_observability_latency_p99_ms",
            json_opt_num(report.daemon_observability_latency_p99_ms),
        ),
        (
            "daemon_observability_throughput_tps",
            json_opt_num(report.daemon_observability_throughput_tps),
        ),
        (
            "daemon_observability_error_rate_bps",
            json_opt_num(report.daemon_observability_error_rate_bps),
        ),
        (
            "daemon_observability_availability_bps",
            json_opt_num(report.daemon_observability_availability_bps),
        ),
        (
            "daemon_observability_health",
            json_opt_str(&report.daemon_observability_health),
        ),
        (
            "daemon_observability_alert_count",
            json_opt_num(report.daemon_observability_alert_count),
        ),
        (
            "daemon_observability_reason_code",
            json_opt_str(&report.daemon_observability_reason_code),
        ),
        (
            "daemon_observability_transport_checkpoint_failures",
            json_opt_num(report.daemon_observability_transport_checkpoint_failures),
        ),
        (
            "daemon_observability_signer_checkpoint_failures",
            json_opt_num(report.daemon_observability_signer_checkpoint_failures),
        ),
        (
            "daemon_observability_commit_checkpoint_failures",
            json_opt_num(report.daemon_observability_commit_checkpoint_failures),
        ),
    ]
}

fn daemon_phase6_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        (
            "daemon_phase6_runtime_reason_taxonomy_version",
            json_opt_str(&report.daemon_phase6_runtime_reason_taxonomy_version),
        ),
        (
            "daemon_phase6_runtime_reason_codes_csv",
            json_opt_str(&report.daemon_phase6_runtime_reason_codes_csv),
        ),
        (
            "daemon_phase6_runtime_reason_code",
            json_opt_str(&report.daemon_phase6_runtime_reason_code),
        ),
        (
            "daemon_phase6_runtime_total_cycles",
            json_opt_num(report.daemon_phase6_runtime_total_cycles),
        ),
        (
            "daemon_phase6_runtime_executed_cycles",
            json_opt_num(report.daemon_phase6_runtime_executed_cycles),
        ),
        (
            "daemon_phase6_runtime_deferred_cycles",
            json_opt_num(report.daemon_phase6_runtime_deferred_cycles),
        ),
        (
            "daemon_phase6_runtime_fail_closed_cycles",
            json_opt_num(report.daemon_phase6_runtime_fail_closed_cycles),
        ),
    ]
}

fn daemon_convergence_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        (
            "daemon_convergence_reason_taxonomy_version",
            json_opt_str(&report.daemon_convergence_reason_taxonomy_version),
        ),
        (
            "daemon_convergence_reason_codes_csv",
            json_opt_str(&report.daemon_convergence_reason_codes_csv),
        ),
        (
            "daemon_convergence_decision",
            json_opt_str(&report.daemon_convergence_decision),
        ),
        (
            "daemon_convergence_reason_code",
            json_opt_str(&report.daemon_convergence_reason_code),
        ),
        (
            "daemon_convergence_schema_gate_passed",
            json_opt_num(report.daemon_convergence_schema_gate_passed),
        ),
        (
            "daemon_convergence_error_path_gate_passed",
            json_opt_num(report.daemon_convergence_error_path_gate_passed),
        ),
        (
            "daemon_convergence_concurrency_gate_passed",
            json_opt_num(report.daemon_convergence_concurrency_gate_passed),
        ),
        (
            "daemon_convergence_performance_budget_gate_passed",
            json_opt_num(report.daemon_convergence_performance_budget_gate_passed),
        ),
        (
            "daemon_convergence_cost_budget_gate_passed",
            json_opt_num(report.daemon_convergence_cost_budget_gate_passed),
        ),
    ]
}

fn daemon_postgres_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        (
            "daemon_live_postgres_multi_host_execution_bundle_schema_version",
            json_opt_str(&report.daemon_live_postgres_multi_host_execution_bundle_schema_version),
        ),
        (
            "daemon_live_postgres_multi_host_execution_bundle_selector_prefix",
            json_opt_str(&report.daemon_live_postgres_multi_host_execution_bundle_selector_prefix),
        ),
        (
            "daemon_live_postgres_multi_host_execution_bundle_row_count",
            json_opt_num(report.daemon_live_postgres_multi_host_execution_bundle_row_count),
        ),
        (
            "daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint",
            json_opt_str(
                &report.daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint,
            ),
        ),
    ]
}

fn kolme_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    let mut fields = kolme_identity_fields(report);
    fields.extend(kolme_observability_fields(report));
    fields
}

fn kolme_identity_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        (
            "kolme_live_provider_client_contract",
            json_opt_str(&report.kolme_live_provider_client_contract),
        ),
        (
            "kolme_live_base_url",
            json_opt_str(&report.kolme_live_base_url),
        ),
        (
            "kolme_live_provider_hint",
            json_opt_str(&report.kolme_live_provider_hint),
        ),
        (
            "kolme_live_signing_profile",
            json_opt_str(&report.kolme_live_signing_profile),
        ),
        (
            "kolme_live_signer_profile_selector_env",
            json_opt_str(&report.kolme_live_signer_profile_selector_env),
        ),
        (
            "kolme_live_signer_profile",
            json_opt_str(&report.kolme_live_signer_profile),
        ),
        (
            "kolme_live_signer_key_source",
            json_opt_str(&report.kolme_live_signer_key_source),
        ),
        (
            "kolme_live_signer_private_key_env",
            json_opt_str(&report.kolme_live_signer_private_key_env),
        ),
        (
            "kolme_live_execution_status",
            json_opt_str(&report.kolme_live_execution_status),
        ),
    ]
}

fn kolme_observability_fields(report: &NodeBootstrapReport) -> Vec<JsonField> {
    vec![
        (
            "kolme_live_observability_latency_p50_ms",
            json_opt_num(report.kolme_live_observability_latency_p50_ms),
        ),
        (
            "kolme_live_observability_latency_p99_ms",
            json_opt_num(report.kolme_live_observability_latency_p99_ms),
        ),
        (
            "kolme_live_observability_throughput_tps",
            json_opt_num(report.kolme_live_observability_throughput_tps),
        ),
        (
            "kolme_live_observability_error_rate_bps",
            json_opt_num(report.kolme_live_observability_error_rate_bps),
        ),
        (
            "kolme_live_observability_availability_bps",
            json_opt_num(report.kolme_live_observability_availability_bps),
        ),
        (
            "kolme_live_observability_health",
            json_opt_str(&report.kolme_live_observability_health),
        ),
        (
            "kolme_live_observability_alert_count",
            json_opt_num(report.kolme_live_observability_alert_count),
        ),
        (
            "kolme_live_observability_reason_code",
            json_opt_str(&report.kolme_live_observability_reason_code),
        ),
        (
            "kolme_live_observability_transport_checkpoint_failures",
            json_opt_num(report.kolme_live_observability_transport_checkpoint_failures),
        ),
        (
            "kolme_live_observability_signer_checkpoint_failures",
            json_opt_num(report.kolme_live_observability_signer_checkpoint_failures),
        ),
        (
            "kolme_live_observability_commit_checkpoint_failures",
            json_opt_num(report.kolme_live_observability_commit_checkpoint_failures),
        ),
    ]
}

use crate::{
    DiagnosticsMode, LocalProfile, NodeBootstrapReport, RuntimeExecutionBundle, RuntimeMode,
};
use kamn_core::BootstrapPlan;

pub(crate) fn build_bootstrap_report(
    plan: &BootstrapPlan,
    profile: Option<LocalProfile>,
    diagnostics_mode: DiagnosticsMode,
    runtime_mode: RuntimeMode,
    runtime_execution: RuntimeExecutionBundle,
) -> NodeBootstrapReport {
    let RuntimeExecutionBundle {
        planning,
        recovery,
        daemon,
        kolme_live,
    } = runtime_execution;
    let operational_profile = plan.config.operational_profile();
    let components = plan
        .wiring
        .all_components()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<String>>();
    let planning_expected_state_hash = planning
        .as_ref()
        .map(|planning| planning.expected_state_hash.clone());
    let planning_candidate_count = planning.as_ref().map(|planning| planning.candidate_count);
    let planning_scheduled_candidate_ids = planning
        .as_ref()
        .map(|planning| planning.scheduled_candidate_ids.clone());
    let recovery_expected_state_version = recovery
        .as_ref()
        .map(|recovery| recovery.expected_state_version);
    let recovery_expected_state_hash = recovery
        .as_ref()
        .map(|recovery| recovery.expected_state_hash.clone());
    let recovery_attempt_count = recovery.as_ref().map(|recovery| recovery.attempt_count);
    let recovery_decisions = recovery.as_ref().map(|recovery| recovery.decisions.clone());
    let daemon_max_ticks = daemon.as_ref().map(|daemon| daemon.max_ticks);
    let daemon_tick_interval_ms = daemon.as_ref().map(|daemon| daemon.tick_interval_ms);
    let daemon_executed_ticks = daemon.as_ref().map(|daemon| daemon.executed_ticks);
    let daemon_completion_reason = daemon
        .as_ref()
        .map(|daemon| daemon.completion_reason.clone());
    let daemon_peer_id = daemon.as_ref().and_then(|daemon| daemon.peer_id.clone());
    let daemon_peer_lifecycle_final_state = daemon
        .as_ref()
        .and_then(|daemon| daemon.peer_lifecycle_final_state.clone());
    let daemon_peer_lifecycle_applied_events = daemon
        .as_ref()
        .and_then(|daemon| daemon.peer_lifecycle_applied_events.clone());
    let kolme_live_provider_client_contract = kolme_live
        .as_ref()
        .map(|execution| execution.provider_client_contract.clone());
    let kolme_live_base_url = kolme_live
        .as_ref()
        .map(|execution| execution.base_url.clone());
    let kolme_live_provider_hint = kolme_live
        .as_ref()
        .map(|execution| execution.provider_hint.clone());
    let kolme_live_signing_profile = kolme_live
        .as_ref()
        .map(|execution| execution.signing_profile.clone());
    let kolme_live_signer_profile_selector_env = kolme_live
        .as_ref()
        .map(|execution| execution.signer_profile_selector_env.clone());
    let kolme_live_signer_profile = kolme_live
        .as_ref()
        .map(|execution| execution.signer_profile.clone());
    let kolme_live_signer_key_source = kolme_live
        .as_ref()
        .map(|execution| execution.signer_key_source.clone());
    let kolme_live_signer_private_key_env = kolme_live
        .as_ref()
        .map(|execution| execution.signer_private_key_env.clone());
    let kolme_live_execution_status = kolme_live
        .as_ref()
        .map(|execution| execution.execution_status.clone());
    NodeBootstrapReport {
        runtime_mode: runtime_mode.as_str().to_owned(),
        diagnostics_mode: diagnostics_mode.as_str().to_owned(),
        component_count: components.len(),
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
        profile: profile.map(LocalProfile::as_str).map(str::to_owned),
        role: plan.config.role.as_str().to_owned(),
        chain_id: plan.config.chain_id.clone(),
        chain_version: plan.config.chain_version.clone(),
        storage_dir: plan.config.storage_dir.clone(),
        gossip_enabled: plan.config.enable_gossip,
        sync_mode: plan.config.sync_mode.as_str().to_owned(),
        sync_startup: format!("{:?}", operational_profile.startup_strategy),
        sync_recovery: format!("{:?}", operational_profile.recovery_strategy),
        state_version: plan.state_schema.version.0,
        pending_migrations: plan.migration_plan.steps.len(),
        components,
    }
}

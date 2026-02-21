use super::*;

mod daemon_phase;

#[cfg(test)]
pub(crate) use daemon_phase::execute_daemon_convergence_projection_for_test;
#[cfg(test)]
pub(crate) use daemon_phase::execute_daemon_phase6_runtime_projection_for_test;
use daemon_phase::execute_daemon_runtime;
#[cfg(test)]
pub(crate) use daemon_phase::live_postgres_multi_host_execution_bundle_row_count_for_test;
#[cfg(test)]
pub(crate) use daemon_phase::live_postgres_multi_host_execution_bundle_selector_rows_for_test;
#[cfg(test)]
pub(crate) use daemon_phase::validate_live_postgres_selector_bundle_for_test;
use daemon_phase::{
    daemon_shutdown_drain_status, daemon_shutdown_reason_field, daemon_shutdown_signal_tick,
    daemon_shutdown_snapshot_flush_status,
};

pub(crate) fn build_runtime_execution_id(
    runtime_mode: RuntimeMode,
    chain_id: &str,
    role: &str,
) -> String {
    format!("node-runtime:{}:{chain_id}:{role}", runtime_mode.as_str())
}

const RUNTIME_TRANSPORT_PROFILE_POLICY_STORE: &str = "runtime-transport-profile";
const RUNTIME_TRANSPORT_PROFILE_GOSSIP_DISABLED_FOR_PRODUCTION_REASON: &str =
    "runtime_transport_profile_gossip_disabled_for_production";
const RUNTIME_TRANSPORT_PROFILE_IN_MEMORY_FALLBACK_FORBIDDEN_REASON: &str =
    "runtime_transport_profile_in_memory_fallback_forbidden";
const RUNTIME_TRANSPORT_PROFILE_PAIR_DISALLOWED_REASON: &str =
    "runtime_transport_profile_pair_disallowed";
const RUNTIME_TRANSPORT_PROFILE_FALLBACK_MARKER_WITHOUT_IN_MEMORY_PROFILE_REASON: &str =
    "runtime_transport_profile_fallback_marker_without_in_memory_profile";
const RUNTIME_TRANSPORT_PROFILE_LIVE_MARKER_MISSING_REASON: &str =
    "runtime_transport_profile_live_marker_missing";
const RUNTIME_TRANSPORT_PROFILE_LIVE_PROVIDER_MISSING_REASON: &str =
    "runtime_transport_profile_live_provider_missing";
const RUNTIME_TRANSPORT_PROFILE_COMPILE_MODE_NOT_NATIVE_REASON: &str =
    "runtime_transport_profile_compile_mode_not_native";
const SHUTDOWN_CHECKPOINT_RECONCILIATION_TIMEOUT_REASON_CODE_MISMATCH: &str =
    "shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch";
const SHUTDOWN_CHECKPOINT_RECONCILIATION_TIMEOUT_CHECKPOINT_MISMATCH: &str =
    "shutdown_checkpoint_reconciliation_timeout_checkpoint_mismatch";
const SHUTDOWN_CHECKPOINT_RECONCILIATION_GRACEFUL_REASON_CODE_MISMATCH: &str =
    "shutdown_checkpoint_reconciliation_graceful_reason_code_mismatch";
const SHUTDOWN_CHECKPOINT_RECONCILIATION_GRACEFUL_CHECKPOINT_MISMATCH: &str =
    "shutdown_checkpoint_reconciliation_graceful_checkpoint_mismatch";
const SHUTDOWN_CHECKPOINT_RECONCILIATION_NOT_SIGNALED_REASON_CODE_MISMATCH: &str =
    "shutdown_checkpoint_reconciliation_not_signaled_reason_code_mismatch";
const SHUTDOWN_CHECKPOINT_RECONCILIATION_NOT_SIGNALED_CHECKPOINT_MISMATCH: &str =
    "shutdown_checkpoint_reconciliation_not_signaled_checkpoint_mismatch";
const SHUTDOWN_CHECKPOINT_RECONCILIATION_UNKNOWN_COMPLETION_REASON: &str =
    "shutdown_checkpoint_reconciliation_unknown_completion_reason";
const KOLME_LIVE_FALLBACK_SIGNER_SECRET_PRESENT_REASON_CODE: &str =
    "fallback_signer_secret_present_violation";

fn production_transport_profile_remediation(reason_code: &'static str) -> &'static str {
    match reason_code {
        RUNTIME_TRANSPORT_PROFILE_GOSSIP_DISABLED_FOR_PRODUCTION_REASON => {
            "remove --disable-gossip (or set enable_gossip=true in config), or use non-production runtime modes (planning/recovery-check)"
        }
        RUNTIME_TRANSPORT_PROFILE_IN_MEMORY_FALLBACK_FORBIDDEN_REASON => {
            "ensure runtime wiring emits p2p-transport-profile:libp2p-live and remove in-memory fallback markers"
        }
        RUNTIME_TRANSPORT_PROFILE_PAIR_DISALLOWED_REASON => {
            "ensure runtime wiring emits exactly one transport profile family (libp2p-live OR in-memory-deterministic), not both"
        }
        RUNTIME_TRANSPORT_PROFILE_FALLBACK_MARKER_WITHOUT_IN_MEMORY_PROFILE_REASON => {
            "ensure p2p-in-memory-transport-fallback is only emitted with p2p-transport-profile:in-memory-deterministic"
        }
        RUNTIME_TRANSPORT_PROFILE_LIVE_MARKER_MISSING_REASON => {
            "ensure bootstrap uses RuntimeTransportProfile::Libp2pLive for production runtime modes"
        }
        RUNTIME_TRANSPORT_PROFILE_LIVE_PROVIDER_MISSING_REASON => {
            "ensure p2p-live-libp2p-provider marker is present and provider wiring is initialized"
        }
        RUNTIME_TRANSPORT_PROFILE_COMPILE_MODE_NOT_NATIVE_REASON => {
            "ensure kamn-node enables kamn-core/libp2p-live-transport so runtime wiring emits p2p-live-libp2p-provider:native"
        }
        _ => "verify runtime transport profile wiring against production policy contract",
    }
}

fn build_production_transport_profile_violation_detail(
    runtime_mode: RuntimeMode,
    reason_code: &'static str,
) -> String {
    format!(
        "runtime mode {} requires live libp2p transport profile wiring; remediation: {}",
        runtime_mode.as_str(),
        production_transport_profile_remediation(reason_code),
    )
}

fn runtime_mode_requires_live_transport_profile(runtime_mode: RuntimeMode) -> bool {
    matches!(
        runtime_mode.kind,
        RuntimeModeKind::Daemon
            | RuntimeModeKind::Api
            | RuntimeModeKind::Full
            | RuntimeModeKind::KolmeLive
    )
}

pub(crate) fn should_use_os_signal_shutdown(
    runtime_mode: RuntimeMode,
    daemon_shutdown_os_signals: bool,
    daemon_shutdown_signal_ticks: &[u64],
) -> bool {
    if !matches!(
        runtime_mode.kind,
        RuntimeModeKind::Daemon | RuntimeModeKind::Full
    ) {
        return false;
    }
    if !daemon_shutdown_signal_ticks.is_empty() {
        return false;
    }
    if daemon_shutdown_os_signals {
        return true;
    }
    cfg!(unix)
}

pub(crate) fn select_runtime_transport_profile_for_runtime_mode(
    runtime_mode: RuntimeMode,
    enable_gossip: bool,
) -> Option<RuntimeTransportProfile> {
    if !enable_gossip {
        return None;
    }
    if runtime_mode_requires_live_transport_profile(runtime_mode) {
        return Some(RuntimeTransportProfile::Libp2pLive);
    }
    None
}

pub(crate) fn classify_production_transport_profile_violation(
    runtime_mode: RuntimeMode,
    enable_gossip: bool,
    components: &[String],
) -> Option<&'static str> {
    let has_component = |expected: &str| components.iter().any(|component| component == expected);
    let has_live_profile = has_component("p2p-transport-profile:libp2p-live");
    let has_in_memory_profile = has_component("p2p-transport-profile:in-memory-deterministic");
    let has_in_memory_fallback = has_component("p2p-in-memory-transport-fallback");
    let has_live_provider = has_component("p2p-live-libp2p-provider");

    if has_live_profile && (has_in_memory_profile || has_in_memory_fallback) {
        return Some(RUNTIME_TRANSPORT_PROFILE_PAIR_DISALLOWED_REASON);
    }
    if has_in_memory_profile && has_live_provider {
        return Some(RUNTIME_TRANSPORT_PROFILE_PAIR_DISALLOWED_REASON);
    }
    if has_in_memory_fallback && !has_in_memory_profile {
        return Some(RUNTIME_TRANSPORT_PROFILE_FALLBACK_MARKER_WITHOUT_IN_MEMORY_PROFILE_REASON);
    }

    if !runtime_mode_requires_live_transport_profile(runtime_mode) {
        return None;
    }
    if !enable_gossip {
        return Some(RUNTIME_TRANSPORT_PROFILE_GOSSIP_DISABLED_FOR_PRODUCTION_REASON);
    }

    if has_in_memory_profile || has_in_memory_fallback {
        return Some(RUNTIME_TRANSPORT_PROFILE_IN_MEMORY_FALLBACK_FORBIDDEN_REASON);
    }
    if !has_live_profile {
        return Some(RUNTIME_TRANSPORT_PROFILE_LIVE_MARKER_MISSING_REASON);
    }
    if !has_live_provider {
        return Some(RUNTIME_TRANSPORT_PROFILE_LIVE_PROVIDER_MISSING_REASON);
    }
    if has_component("p2p-live-libp2p-provider:contract-only")
        || !has_component("p2p-live-libp2p-provider:native")
    {
        return Some(RUNTIME_TRANSPORT_PROFILE_COMPILE_MODE_NOT_NATIVE_REASON);
    }
    None
}

fn enforce_production_transport_profile_policy(
    runtime_mode: RuntimeMode,
    plan: &kamn_core::BootstrapPlan,
) -> Result<(), ConfigError> {
    let components = plan
        .wiring
        .all_components()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<String>>();
    if let Some(reason_code) = classify_production_transport_profile_violation(
        runtime_mode,
        plan.config.enable_gossip,
        components.as_slice(),
    ) {
        return Err(ConfigError::RuntimeStoreCompatibility {
            store: RUNTIME_TRANSPORT_PROFILE_POLICY_STORE,
            reason_code,
            detail: build_production_transport_profile_violation_detail(runtime_mode, reason_code),
        });
    }
    Ok(())
}

pub(crate) fn classify_full_bootstrap_component_contract_violation(
    components: &[&str],
) -> Option<&'static str> {
    if components.len() != FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.len() {
        return Some("full_supervisor_bootstrap_component_count_mismatch");
    }
    for (component, expected_component) in components
        .iter()
        .zip(FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.iter())
    {
        if component != expected_component {
            return Some("full_supervisor_bootstrap_component_order_mismatch");
        }
    }
    None
}

pub(crate) fn validate_full_bootstrap_component_contract(
    components: &[&str],
) -> Result<(), ConfigError> {
    if let Some(reason) = classify_full_bootstrap_component_contract_violation(components) {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "full_supervisor_invariant_violation:{reason}"
        )));
    }
    Ok(())
}

fn parse_nonempty_u64(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<u64>().ok()
}

fn parse_positive_u64(value: &str) -> Option<u64> {
    parse_nonempty_u64(value).filter(|value| *value > 0)
}

pub(crate) fn classify_full_supervisor_stop_contract_violation(
    completion_reason: &str,
    shutdown_drain_status: &str,
    shutdown_snapshot_flush_status: &str,
) -> Option<&'static str> {
    if !matches!(
        shutdown_drain_status,
        "completed" | "timeout" | "not-signaled"
    ) {
        return Some("full_supervisor_stop_invalid_shutdown_drain_status");
    }
    if !matches!(
        shutdown_snapshot_flush_status,
        "snapshot-flushed" | "snapshot-flush-timeout" | "snapshot-not-requested"
    ) {
        return Some("full_supervisor_stop_invalid_shutdown_snapshot_flush_status");
    }
    if completion_reason == "tick-budget-exhausted"
        || completion_reason.starts_with("tick-budget-exhausted;ignored_signals=")
    {
        if let Some(raw_ignored_signals) =
            daemon_shutdown_reason_field(completion_reason, "ignored_signals")
        {
            if parse_nonempty_u64(raw_ignored_signals).is_none() {
                return Some("full_supervisor_stop_invalid_ignored_signals");
            }
        }
        if shutdown_drain_status != "not-signaled" {
            return Some("full_supervisor_stop_not_signaled_status_mismatch");
        }
        if shutdown_snapshot_flush_status != "snapshot-not-requested" {
            return Some("full_supervisor_stop_not_signaled_snapshot_flush_mismatch");
        }
        return None;
    }
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        if daemon_shutdown_signal_tick(completion_reason).is_none() {
            return Some("full_supervisor_stop_missing_signal_tick");
        }
        let Some(raw_drain_ticks) = daemon_shutdown_reason_field(completion_reason, "drain_ticks")
        else {
            return Some("full_supervisor_stop_missing_drain_ticks");
        };
        let Some(drain_ticks) = parse_positive_u64(raw_drain_ticks) else {
            return Some("full_supervisor_stop_invalid_drain_ticks");
        };
        let Some(raw_timeout_ticks) =
            daemon_shutdown_reason_field(completion_reason, "timeout_ticks")
        else {
            return Some("full_supervisor_stop_missing_timeout_ticks");
        };
        let Some(timeout_ticks) = parse_positive_u64(raw_timeout_ticks) else {
            return Some("full_supervisor_stop_invalid_timeout_ticks");
        };
        let Some(raw_ignored_signals) =
            daemon_shutdown_reason_field(completion_reason, "ignored_signals")
        else {
            return Some("full_supervisor_stop_missing_ignored_signals");
        };
        if parse_nonempty_u64(raw_ignored_signals).is_none() {
            return Some("full_supervisor_stop_invalid_ignored_signals");
        }
        if drain_ticks > timeout_ticks {
            return Some("full_supervisor_stop_graceful_drain_timeout_contract_mismatch");
        }
        if shutdown_drain_status != "completed" {
            return Some("full_supervisor_stop_graceful_status_mismatch");
        }
        if shutdown_snapshot_flush_status != "snapshot-flushed" {
            return Some("full_supervisor_stop_graceful_snapshot_flush_status_mismatch");
        }
        return None;
    }
    if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        if daemon_shutdown_signal_tick(completion_reason).is_none() {
            return Some("full_supervisor_stop_missing_signal_tick");
        }
        let Some(raw_drain_ticks) = daemon_shutdown_reason_field(completion_reason, "drain_ticks")
        else {
            return Some("full_supervisor_stop_missing_drain_ticks");
        };
        if parse_positive_u64(raw_drain_ticks).is_none() {
            return Some("full_supervisor_stop_invalid_drain_ticks");
        }
        let Some(raw_timeout_ticks) =
            daemon_shutdown_reason_field(completion_reason, "timeout_ticks")
        else {
            return Some("full_supervisor_stop_missing_timeout_ticks");
        };
        if parse_positive_u64(raw_timeout_ticks).is_none() {
            return Some("full_supervisor_stop_invalid_timeout_ticks");
        }
        let Some(raw_ignored_signals) =
            daemon_shutdown_reason_field(completion_reason, "ignored_signals")
        else {
            return Some("full_supervisor_stop_missing_ignored_signals");
        };
        if parse_nonempty_u64(raw_ignored_signals).is_none() {
            return Some("full_supervisor_stop_invalid_ignored_signals");
        }
        if shutdown_drain_status != "timeout" {
            return Some("full_supervisor_stop_graceful_timeout_status_mismatch");
        }
        if shutdown_snapshot_flush_status != "snapshot-flush-timeout" {
            return Some("full_supervisor_stop_graceful_timeout_snapshot_flush_status_mismatch");
        }
        return None;
    }
    Some("full_supervisor_stop_unknown_completion_reason")
}

pub(crate) fn validate_full_supervisor_stop_contract(
    completion_reason: &str,
    shutdown_drain_status: &str,
    shutdown_snapshot_flush_status: &str,
) -> Result<(), ConfigError> {
    if let Some(reason) = classify_full_supervisor_stop_contract_violation(
        completion_reason,
        shutdown_drain_status,
        shutdown_snapshot_flush_status,
    ) {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "full_supervisor_invariant_violation:{reason}"
        )));
    }
    Ok(())
}

pub(crate) fn classify_shutdown_checkpoint_reconciliation_violation(
    completion_reason: &str,
    observability_reason_code: &str,
    transport_checkpoint_failures: u64,
    signer_checkpoint_failures: u64,
    commit_checkpoint_failures: u64,
) -> Option<&'static str> {
    if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        if observability_reason_code != "daemon_shutdown_timeout" {
            return Some(SHUTDOWN_CHECKPOINT_RECONCILIATION_TIMEOUT_REASON_CODE_MISMATCH);
        }
        if transport_checkpoint_failures != 0
            || signer_checkpoint_failures != 0
            || commit_checkpoint_failures != 1
        {
            return Some(SHUTDOWN_CHECKPOINT_RECONCILIATION_TIMEOUT_CHECKPOINT_MISMATCH);
        }
        return None;
    }
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        if observability_reason_code != "daemon_shutdown_signal" {
            return Some(SHUTDOWN_CHECKPOINT_RECONCILIATION_GRACEFUL_REASON_CODE_MISMATCH);
        }
        if transport_checkpoint_failures != 0
            || signer_checkpoint_failures != 0
            || commit_checkpoint_failures != 0
        {
            return Some(SHUTDOWN_CHECKPOINT_RECONCILIATION_GRACEFUL_CHECKPOINT_MISMATCH);
        }
        return None;
    }
    if completion_reason == "tick-budget-exhausted"
        || completion_reason.starts_with("tick-budget-exhausted;ignored_signals=")
    {
        if observability_reason_code != "none" {
            return Some(SHUTDOWN_CHECKPOINT_RECONCILIATION_NOT_SIGNALED_REASON_CODE_MISMATCH);
        }
        if transport_checkpoint_failures != 0
            || signer_checkpoint_failures != 0
            || commit_checkpoint_failures != 0
        {
            return Some(SHUTDOWN_CHECKPOINT_RECONCILIATION_NOT_SIGNALED_CHECKPOINT_MISMATCH);
        }
        return None;
    }
    Some(SHUTDOWN_CHECKPOINT_RECONCILIATION_UNKNOWN_COMPLETION_REASON)
}

pub(crate) fn validate_shutdown_checkpoint_reconciliation(
    completion_reason: &str,
    observability_reason_code: &str,
    transport_checkpoint_failures: u64,
    signer_checkpoint_failures: u64,
    commit_checkpoint_failures: u64,
) -> Result<(), ConfigError> {
    if let Some(reason) = classify_shutdown_checkpoint_reconciliation_violation(
        completion_reason,
        observability_reason_code,
        transport_checkpoint_failures,
        signer_checkpoint_failures,
        commit_checkpoint_failures,
    ) {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "runtime_shutdown_invariant_violation:{reason}"
        )));
    }
    Ok(())
}

pub(crate) fn resolve_kolme_live_allow_local_signer_testing_override() -> Result<bool, ConfigError>
{
    match env::var(KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must not be empty when present (legacy_local_signer_path_override_invalid)"
                )));
            }
            match trimmed {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must be 'true' or 'false', found '{trimmed}' (legacy_local_signer_path_override_invalid)"
                ))),
            }
        }
        Err(env::VarError::NotPresent) => Ok(false),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must be valid utf-8 when present (legacy_local_signer_path_override_invalid)"
        ))),
    }
}

pub(crate) fn enforce_kolme_live_signer_contract_policy(
    strict_signer_contracts_enabled: bool,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Result<(), ConfigError> {
    if strict_signer_contracts_enabled || allow_local_signer_testing_override || is_test_build {
        return Ok(());
    }

    Err(ConfigError::RuntimeKolmeLive(format!(
        "--kolme-live-strict-signer-contracts is required for runtime-mode kolme-live; set {KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV}=true only for explicit local testing override (legacy_local_signer_path_forbidden)"
    )))
}

pub(crate) fn classify_kolme_live_signer_key_source_policy_violation(
    strict_signer_contracts_enabled: bool,
    strict_signer_key_source: Option<&str>,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Option<&'static str> {
    if !strict_signer_contracts_enabled || allow_local_signer_testing_override || is_test_build {
        return None;
    }
    if strict_signer_key_source == Some(KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL) {
        return Some(KOLME_LIVE_ENV_LOCAL_SIGNER_KEY_SOURCE_FORBIDDEN_REASON_CODE);
    }
    None
}

fn classify_kolme_live_signer_fallback_secret_policy_violation(
) -> Result<Option<&'static str>, ConfigError> {
    match env::var(KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV) {
        Ok(_) => Ok(Some(KOLME_LIVE_FALLBACK_SIGNER_SECRET_PRESENT_REASON_CODE)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must be valid utf-8 when present ({KOLME_LIVE_FALLBACK_SIGNER_SECRET_PRESENT_REASON_CODE})"
        ))),
    }
}

pub(crate) fn enforce_kolme_live_signer_key_source_policy(
    strict_signer_contracts_enabled: bool,
    strict_signer_key_source: Option<&str>,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Result<(), ConfigError> {
    if let Some(reason_code) = classify_kolme_live_signer_fallback_secret_policy_violation()? {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must remain unset for runtime signer policy enforcement; remediation: unset {KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} ({reason_code})"
        )));
    }
    let reason_code = classify_kolme_live_signer_key_source_policy_violation(
        strict_signer_contracts_enabled,
        strict_signer_key_source,
        allow_local_signer_testing_override,
        is_test_build,
    );
    if let Some(reason_code) = reason_code {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-key-source={KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL} is not allowed when --kolme-live-strict-signer-contracts is enabled for production-targeted runs; use --kolme-live-signer-key-source={KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL} or set {KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV}=true for explicit local testing override ({reason_code})"
        )));
    }
    Ok(())
}

pub(crate) fn execute(cli: NodeCli) -> Result<NodeBootstrapReport, ConfigError> {
    let NodeCli {
        profile,
        role,
        chain_id,
        chain_version,
        storage_dir,
        enable_gossip,
        sync_mode,
        runtime_mode,
        expected_state_version,
        expected_state_hash,
        proposals,
        rejoin_attempts,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks,
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id,
        daemon_lifecycle_events,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_strict_signer_contracts,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        api_bind_addr: _,
        api_max_requests: _,
        api_idle_timeout_ms: _,
        api_body_limit_bytes: _,
        api_concurrency_limit: _,
        api_rate_limit_per_second: _,
        observability_endpoint_bind_addr: _,
        observability_endpoint_metrics_path: _,
        observability_endpoint_health_path: _,
        observability_endpoint_max_requests: _,
        observability_endpoint_idle_timeout_ms: _,
        output_mode: _,
        diagnostics_mode,
    } = cli;
    let execution_id = build_runtime_execution_id(runtime_mode, chain_id.as_str(), role.as_str());

    let config = NodeConfig {
        chain_id: chain_id.clone(),
        chain_version: chain_version.clone(),
        role,
        storage_dir: storage_dir.clone(),
        enable_gossip,
        sync_mode,
    };

    let selected_transport_profile =
        select_runtime_transport_profile_for_runtime_mode(runtime_mode, config.enable_gossip);
    let plan = match selected_transport_profile {
        Some(transport_profile) => bootstrap_with_transport_profile(config, transport_profile)?,
        None => bootstrap(config)?,
    };
    enforce_production_transport_profile_policy(runtime_mode, &plan)?;
    log_info(
        "node.runtime.mode.dispatch",
        &[
            ("runtime_mode", runtime_mode.as_str()),
            ("execution_id", execution_id.as_str()),
        ],
    )?;
    let runtime_execution = match runtime_mode.kind {
        RuntimeModeKind::Bootstrap => {
            log_info(
                "node.runtime.bootstrap.plan.ready",
                &[
                    ("chain_id", plan.config.chain_id.as_str()),
                    ("role", plan.config.role.as_str()),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            RuntimeExecutionBundle::default()
        }
        RuntimeModeKind::Planning => {
            let expected_state_hash = expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let planner = DeterministicProposalPlanner::new(&expected_state_hash);
            let proposal_plan = planner
                .plan(proposals)
                .map_err(|error| ConfigError::RuntimePlanner(error.to_string()))?;
            RuntimeExecutionBundle {
                planning: Some(PlanningExecution {
                    expected_state_hash,
                    candidate_count: proposal_plan.ordered_candidates().len(),
                    scheduled_candidate_ids: proposal_plan.ordered_candidate_ids(),
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::RecoveryCheck => {
            let expected_state_version = expected_state_version.ok_or(
                ConfigError::MissingArgumentValue("--expected-state-version"),
            )?;
            let expected_state_hash = expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let mut guard = RecoveryRejoinGuard::new(expected_state_version, &expected_state_hash)
                .map_err(|error| ConfigError::RuntimeRecovery(error.to_string()))?;
            let mut decisions = Vec::with_capacity(rejoin_attempts.len());
            for attempt in rejoin_attempts {
                let decision = match guard
                    .evaluate(attempt)
                    .map_err(|error| ConfigError::RuntimeRecovery(error.to_string()))?
                {
                    RecoveryStatus::RejoinAccepted => "rejoin-accepted".to_owned(),
                    RecoveryStatus::CatchUpRequired {
                        from_version,
                        to_version,
                    } => {
                        format!("catch-up-required:{from_version}->{to_version}")
                    }
                };
                decisions.push(decision);
            }
            RuntimeExecutionBundle {
                recovery: Some(RecoveryExecution {
                    expected_state_version,
                    expected_state_hash,
                    attempt_count: decisions.len(),
                    decisions,
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::Daemon => {
            let daemon_execution = execute_daemon_runtime(
                runtime_mode,
                execution_id.as_str(),
                DaemonRuntimeOptions {
                    daemon_max_ticks,
                    daemon_tick_interval_ms,
                    daemon_shutdown_signal_ticks,
                    daemon_shutdown_os_signals,
                    daemon_shutdown_drain_ticks,
                    daemon_shutdown_timeout_ticks,
                    daemon_peer_id,
                    daemon_lifecycle_events,
                },
            )?;
            RuntimeExecutionBundle {
                daemon: Some(daemon_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::Api => {
            log_info(
                "node.runtime.service_api.mode.ready",
                &[
                    ("runtime_mode", runtime_mode.as_str()),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            RuntimeExecutionBundle::default()
        }
        RuntimeModeKind::Full => {
            validate_full_bootstrap_component_contract(
                FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.as_slice(),
            )?;
            log_info(
                "node.runtime.full.bootstrap.start",
                &[("execution_id", execution_id.as_str())],
            )?;
            for (stage_index, component) in FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE
                .into_iter()
                .enumerate()
            {
                let stage_index_label = (stage_index + 1).to_string();
                log_info(
                    "node.runtime.full.bootstrap.component.ready",
                    &[
                        ("component", component),
                        ("stage_index", stage_index_label.as_str()),
                        ("execution_id", execution_id.as_str()),
                    ],
                )?;
            }
            let daemon_execution = execute_daemon_runtime(
                runtime_mode,
                execution_id.as_str(),
                DaemonRuntimeOptions {
                    daemon_max_ticks,
                    daemon_tick_interval_ms,
                    daemon_shutdown_signal_ticks,
                    daemon_shutdown_os_signals,
                    daemon_shutdown_drain_ticks,
                    daemon_shutdown_timeout_ticks,
                    daemon_peer_id,
                    daemon_lifecycle_events,
                },
            )?;
            log_info(
                "node.runtime.full.bootstrap.ready",
                &[("execution_id", execution_id.as_str())],
            )?;
            let stop_reason = "daemon-execution-complete";
            let completion_reason = daemon_execution.completion_reason.as_str();
            let shutdown_drain_status = daemon_shutdown_drain_status(completion_reason);
            let shutdown_snapshot_flush_status =
                daemon_shutdown_snapshot_flush_status(completion_reason);
            let shutdown_signal_tick =
                daemon_shutdown_signal_tick(completion_reason).unwrap_or("none");
            let shutdown_drain_ticks =
                daemon_shutdown_reason_field(completion_reason, "drain_ticks").unwrap_or("0");
            let shutdown_timeout_ticks =
                daemon_shutdown_reason_field(completion_reason, "timeout_ticks").unwrap_or("0");
            let shutdown_ignored_signals =
                daemon_shutdown_reason_field(completion_reason, "ignored_signals").unwrap_or("0");
            validate_full_supervisor_stop_contract(
                completion_reason,
                shutdown_drain_status,
                shutdown_snapshot_flush_status,
            )?;
            log_info(
                "node.runtime.full.supervisor.stop.requested",
                &[
                    ("stop_reason", stop_reason),
                    ("daemon_completion_reason", completion_reason),
                    (
                        "shutdown_snapshot_flush_status",
                        shutdown_snapshot_flush_status,
                    ),
                    ("shutdown_signal_tick", shutdown_signal_tick),
                    ("shutdown_drain_ticks", shutdown_drain_ticks),
                    ("shutdown_timeout_ticks", shutdown_timeout_ticks),
                    ("shutdown_ignored_signals", shutdown_ignored_signals),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            log_info(
                "node.runtime.full.supervisor.stop.complete",
                &[
                    ("stop_reason", stop_reason),
                    ("daemon_completion_reason", completion_reason),
                    ("shutdown_drain_status", shutdown_drain_status),
                    (
                        "shutdown_snapshot_flush_status",
                        shutdown_snapshot_flush_status,
                    ),
                    ("shutdown_signal_tick", shutdown_signal_tick),
                    ("shutdown_drain_ticks", shutdown_drain_ticks),
                    ("shutdown_timeout_ticks", shutdown_timeout_ticks),
                    ("shutdown_ignored_signals", shutdown_ignored_signals),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            RuntimeExecutionBundle {
                daemon: Some(daemon_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::KolmeLive => {
            let base_url = kolme_live_base_url
                .ok_or(ConfigError::MissingArgumentValue("--kolme-live-base-url"))?;
            let provider_hint = kolme_live_provider_hint.ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-provider-hint"),
            )?;
            let signing_profile = kolme_live_signing_profile.ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-signing-profile"),
            )?;
            let allow_local_signer_testing_override =
                resolve_kolme_live_allow_local_signer_testing_override()?;
            enforce_kolme_live_signer_contract_policy(
                kolme_live_strict_signer_contracts,
                allow_local_signer_testing_override,
                cfg!(test),
            )?;
            let declared_signer_profile = kolme_live_signer_profile
                .as_deref()
                .map(normalize_kolme_live_signer_profile_selector)
                .transpose()?;
            let declared_signer_key_source = kolme_live_signer_key_source
                .as_deref()
                .map(normalize_kolme_live_signer_key_source)
                .transpose()?;
            let strict_signer_profile = if kolme_live_strict_signer_contracts {
                Some(
                    declared_signer_profile.ok_or(ConfigError::MissingArgumentValue(
                        "--kolme-live-signer-profile",
                    ))?,
                )
            } else {
                declared_signer_profile
            };
            let strict_signer_key_source = if kolme_live_strict_signer_contracts {
                Some(
                    declared_signer_key_source.ok_or(ConfigError::MissingArgumentValue(
                        "--kolme-live-signer-key-source",
                    ))?,
                )
            } else {
                declared_signer_key_source
            };
            enforce_kolme_live_signer_key_source_policy(
                kolme_live_strict_signer_contracts,
                strict_signer_key_source,
                allow_local_signer_testing_override,
                cfg!(test),
            )?;
            let _signer_preflight = enforce_kolme_live_signer_preflight(
                strict_signer_profile,
                strict_signer_key_source,
            )?;
            let kolme_live_execution =
                if daemon_max_ticks.is_some() || daemon_tick_interval_ms.is_some() {
                    let max_cycles = daemon_max_ticks
                        .ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
                    let cycle_interval_ms = daemon_tick_interval_ms.ok_or(
                        ConfigError::MissingArgumentValue("--daemon-tick-interval-ms"),
                    )?;
                    execute_kolme_live_runtime_continuous(
                        &plan,
                        base_url,
                        provider_hint,
                        signing_profile,
                        strict_signer_profile,
                        strict_signer_key_source,
                        KolmeLiveContinuousMode {
                            max_cycles,
                            cycle_interval_ms,
                        },
                    )?
                } else {
                    execute_kolme_live_runtime(
                        &plan,
                        base_url,
                        provider_hint,
                        signing_profile,
                        strict_signer_profile,
                        strict_signer_key_source,
                    )?
                };
            RuntimeExecutionBundle {
                kolme_live: Some(kolme_live_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
    };
    let report = build_bootstrap_report(
        &plan,
        profile,
        diagnostics_mode,
        runtime_mode,
        runtime_execution,
    );

    Ok(report)
}

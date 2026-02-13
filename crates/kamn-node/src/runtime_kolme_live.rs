use super::kolme_live_observability::build_kolme_live_observability_telemetry;
use super::signer::build_kolme_live_direct_signed_wire_payload;
use super::{
    KolmeLiveExecution, KOLME_IN_MEMORY_PROVIDER_MARKER, KOLME_LIVE_FINALITY_MAX_ATTEMPTS,
    KOLME_LIVE_FINALITY_STATUS_PATH, KOLME_LIVE_PROVIDER_CONTRACT, KOLME_LIVE_SIGNER_PROFILE_ENV,
    KOLME_LIVE_SIGNING_PROFILE, KOLME_LIVE_TRANSPORT_TIMEOUT_SECONDS,
};
use kamn_core::{
    BootstrapPlan, ConfigError, KolmeCommitReceiptFinality, KolmeRuntimeCommitFinalityChecker,
    KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitLiveProvider, KolmeRuntimeCommitProvider,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderOutcome,
    KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitRequest,
};

pub(crate) fn build_kolme_live_request(
    plan: &BootstrapPlan,
) -> Result<KolmeRuntimeCommitRequest, ConfigError> {
    let role_label = plan.config.role.as_str();
    let operation_id = format!("runtime-commit:{}:{role_label}", plan.config.chain_id);
    let state_root = format!(
        "state:{}:{}",
        plan.config.chain_version, plan.state_schema.version.0
    );
    let actor_did = format!("kamn:did:agent:node-runtime-{role_label}");
    let payload_hash = format!("payload:{}:{role_label}", plan.config.chain_version);
    KolmeRuntimeCommitRequest::deterministic(
        operation_id.as_str(),
        state_root.as_str(),
        actor_did.as_str(),
        1,
        payload_hash.as_str(),
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))
}

pub(crate) fn ensure_kolme_live_provider_marker(
    expected: &str,
    observed: &str,
) -> Result<(), ConfigError> {
    if expected == observed {
        return Ok(());
    }
    Err(ConfigError::RuntimeKolmeLive(format!(
        "provider marker drift: expected '{expected}', observed '{observed}'"
    )))
}

pub(crate) fn kolme_live_finality_label(finality: KolmeCommitReceiptFinality) -> &'static str {
    match finality {
        KolmeCommitReceiptFinality::Pending => "pending",
        KolmeCommitReceiptFinality::Final => "final",
        KolmeCommitReceiptFinality::Failed => "failed",
    }
}

pub(crate) fn map_kolme_live_submit_outcome(
    outcome: KolmeRuntimeCommitProviderOutcome,
) -> Result<(&'static str, KolmeRuntimeCommitProviderReceipt), ConfigError> {
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => Ok(("submitted", receipt)),
        KolmeRuntimeCommitProviderOutcome::Duplicate(receipt) => Ok(("duplicate", receipt)),
        KolmeRuntimeCommitProviderOutcome::Rejected { reason } => {
            Err(ConfigError::RuntimeKolmeLive(format!(
                "provider rejected runtime commit submission: {reason}"
            )))
        }
    }
}

pub(crate) fn execute_kolme_live_runtime(
    plan: &BootstrapPlan,
    base_url: String,
    provider_hint: String,
    signing_profile: String,
    strict_signer_profile: Option<&'static str>,
    strict_signer_key_source: Option<&'static str>,
) -> Result<KolmeLiveExecution, ConfigError> {
    if provider_hint.contains(KOLME_IN_MEMORY_PROVIDER_MARKER) {
        return Err(ConfigError::InvalidKolmeLiveProviderHint(provider_hint));
    }
    if signing_profile != KOLME_LIVE_SIGNING_PROFILE {
        return Err(ConfigError::InvalidKolmeLiveSigningProfile(signing_profile));
    }

    let mut transport = KolmeRuntimeCommitHttpTransport::new(KOLME_LIVE_TRANSPORT_TIMEOUT_SECONDS)
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let request = build_kolme_live_request(plan)?;
    let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        strict_signer_profile,
        strict_signer_key_source,
    )?;
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        provider_hint.as_str(),
        transport.clone(),
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let submit_outcome = provider
        .submit_runtime_commit(signed_wire_payload.as_str(), request.idempotency_key())
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let (submit_status, mut receipt) = map_kolme_live_submit_outcome(submit_outcome)?;
    ensure_kolme_live_provider_marker(provider_hint.as_str(), receipt.provider.as_str())?;
    let mut resolution = "submit-receipt".to_owned();

    if matches!(receipt.finality, KolmeCommitReceiptFinality::Pending) {
        let mut checker = KolmeRuntimeCommitFinalityChecker::new(
            base_url.as_str(),
            KOLME_LIVE_FINALITY_STATUS_PATH,
            transport,
        )
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
        match checker.poll_finality(receipt.commit_id.as_str(), KOLME_LIVE_FINALITY_MAX_ATTEMPTS) {
            Ok(polled_receipt) => {
                ensure_kolme_live_provider_marker(
                    provider_hint.as_str(),
                    polled_receipt.provider.as_str(),
                )?;
                receipt = polled_receipt;
                resolution = "finality-polled".to_owned();
            }
            Err(KolmeRuntimeCommitProviderError::Timeout) => {
                resolution = "finality-timeout".to_owned();
            }
            Err(KolmeRuntimeCommitProviderError::Unavailable { .. }) => {
                resolution = "finality-unavailable".to_owned();
            }
            Err(KolmeRuntimeCommitProviderError::MalformedResponse { reason }) => {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "finality response malformed: {reason}"
                )));
            }
        }
    }

    let finality = kolme_live_finality_label(receipt.finality);
    let execution_status = format!(
        "{submit_status};commit_id={};finality={finality};resolution={resolution}",
        receipt.commit_id
    );
    let observability = build_kolme_live_observability_telemetry(execution_status.as_str())
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    Ok(KolmeLiveExecution {
        provider_client_contract: KOLME_LIVE_PROVIDER_CONTRACT.to_owned(),
        base_url,
        provider_hint,
        signing_profile,
        signer_profile_selector_env: KOLME_LIVE_SIGNER_PROFILE_ENV.to_owned(),
        signer_profile: signer_selection.profile.to_owned(),
        signer_key_source: signer_selection.key_source.to_owned(),
        signer_private_key_env: signer_selection.private_key_env.to_owned(),
        execution_status,
        observability_latency_p50_ms: observability.latency_p50_ms,
        observability_latency_p99_ms: observability.latency_p99_ms,
        observability_throughput_tps: observability.throughput_tps,
        observability_error_rate_bps: observability.error_rate_bps,
        observability_availability_bps: observability.availability_bps,
        observability_health: observability.health,
        observability_alert_count: observability.alert_count,
    })
}

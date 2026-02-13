use kamn_core::{
    BootstrapPlan, ConfigError, KolmeCommitReceiptFinality, KolmeRuntimeCommitProviderOutcome,
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

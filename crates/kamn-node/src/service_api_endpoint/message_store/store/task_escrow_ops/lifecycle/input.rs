use super::*;

const DIGEST_LEN: usize = 64;

#[derive(Debug, Deserialize)]
pub(super) struct CreateInput {
    pub(super) provider_did: String,
    pub(super) transaction_id: String,
    pub(super) terms_digest: String,
    pub(super) idempotency_key: String,
    #[serde(default)]
    pub(super) creator: Option<String>,
    #[serde(default)]
    pub(super) task_type: Option<String>,
    #[serde(default)]
    pub(super) description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct TransitionInput {
    pub(super) idempotency_key: String,
    #[serde(default)]
    pub(super) completion_evidence_digest: Option<String>,
}

pub(super) fn parse_create(payload: &str, actor: &str) -> Result<CreateInput, TaskLifecycleError> {
    let input: CreateInput =
        serde_json::from_str(payload).map_err(|error| agreement(error.to_string()))?;
    validate_provider(input.provider_did.as_str())?;
    validate_creator(input.creator.as_deref(), actor)?;
    validate_agreement(&input)?;
    Ok(input)
}

pub(super) fn parse_transition(
    payload: &str,
    action: &str,
) -> Result<TransitionInput, TaskLifecycleError> {
    let input: TransitionInput = serde_json::from_str(payload)
        .map_err(|error| bad("TASK_AGREEMENT_INVALID", error.to_string()))?;
    if input.idempotency_key.trim().is_empty() {
        return Err(agreement("transition idempotency key is required"));
    }
    validate_completion_evidence(action, input.completion_evidence_digest.as_deref())?;
    Ok(input)
}

fn validate_provider(provider: &str) -> Result<(), TaskLifecycleError> {
    AgentDid::parse(provider)
        .map(|_| ())
        .map_err(|error| bad("TASK_PROVIDER_INVALID", error.to_string()))
}

fn validate_creator(creator: Option<&str>, actor: &str) -> Result<(), TaskLifecycleError> {
    if creator.is_none_or(|value| value == actor) {
        return Ok(());
    }
    Err(bad(
        "TASK_CREATOR_MISMATCH",
        "body creator differs from authenticated actor",
    ))
}

fn validate_agreement(input: &CreateInput) -> Result<(), TaskLifecycleError> {
    if !input.transaction_id.trim().is_empty()
        && !input.idempotency_key.trim().is_empty()
        && valid_digest(input.terms_digest.as_str())
    {
        return Ok(());
    }
    Err(agreement(
        "transaction, terms digest, and idempotency key are required",
    ))
}

fn validate_completion_evidence(
    action: &str,
    digest: Option<&str>,
) -> Result<(), TaskLifecycleError> {
    if action != "task:complete" || digest.is_some_and(valid_digest) {
        return Ok(());
    }
    Err(bad(
        "TASK_COMPLETION_EVIDENCE_INVALID",
        "completion evidence digest is required",
    ))
}

fn valid_digest(value: &str) -> bool {
    value.len() == DIGEST_LEN
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

use super::*;

const MAX_AMOUNT_LAMPORTS: u64 = 1_000_000;

pub(super) fn parse_fund(payload: &str) -> Result<FundInput, EscrowLifecycleError> {
    let input: FundInput = serde_json::from_str(payload)
        .map_err(|error| bad("ESCROW_AGREEMENT_INVALID", error.to_string()))?;
    if !(1..=MAX_AMOUNT_LAMPORTS).contains(&input.amount_lamports) {
        return Err(bad("ESCROW_AMOUNT_INVALID", "amount is outside MVP bounds"));
    }
    if input.network != "solana-devnet" {
        return Err(bad(
            "ESCROW_NETWORK_INVALID",
            "network must be solana-devnet",
        ));
    }
    if input.idempotency_key.trim().is_empty() || input.release_policy != "task-completed" {
        return Err(bad(
            "ESCROW_AGREEMENT_INVALID",
            "escrow policy or retry key is invalid",
        ));
    }
    Ok(input)
}

pub(super) fn parse_release_key(payload: &str) -> Result<String, EscrowLifecycleError> {
    let value: serde_json::Value = serde_json::from_str(payload)
        .map_err(|error| bad("ESCROW_AGREEMENT_INVALID", error.to_string()))?;
    value
        .get("idempotency_key")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            bad(
                "ESCROW_AGREEMENT_INVALID",
                "release idempotency key is required",
            )
        })
}

pub(super) fn validate_funding(
    actor: &str,
    input: &FundInput,
    task: &ServiceApiPersistedTaskRecord,
) -> Result<(), EscrowLifecycleError> {
    if task.state != "accepted" {
        return Err(conflict(
            "ESCROW_TASK_STATE_CONFLICT",
            "task is not accepted",
        ));
    }
    if task.creator_did.as_deref() != Some(actor) {
        return Err(forbidden(
            "ESCROW_FUNDER_MISMATCH",
            "funder is not task creator",
        ));
    }
    validate_participants(actor, input, task)?;
    validate_terms(input, task)
}

pub(super) fn validate_release(
    store: &ServiceApiMessageStore,
    actor: &str,
    escrow: &ServiceApiPersistedEscrowRecord,
) -> Result<(), EscrowLifecycleError> {
    if escrow.release_authority_did.as_deref() != Some(actor) {
        return Err(forbidden(
            "ESCROW_RELEASE_AUTHORITY_MISMATCH",
            "actor is not release authority",
        ));
    }
    let task_id = escrow.task_id.as_deref().ok_or_else(migration_required)?;
    let task = store
        .snapshot
        .tasks
        .get(task_id)
        .ok_or(EscrowLifecycleError::NotFound)?;
    if task.state != "completed" || task.completion_evidence_digest.is_none() {
        return Err(conflict(
            "ESCROW_RELEASE_NOT_ELIGIBLE",
            "task is not completed with evidence",
        ));
    }
    Ok(())
}

pub(super) fn build_record(
    id: &str,
    actor: &str,
    input: FundInput,
) -> ServiceApiPersistedEscrowRecord {
    ServiceApiPersistedEscrowRecord {
        escrow_id: id.to_owned(),
        state: "funded".to_owned(),
        task_id: Some(input.task_id),
        transaction_id: Some(input.transaction_id),
        funder_did: Some(actor.to_owned()),
        beneficiary_did: Some(input.beneficiary_did),
        amount_lamports: Some(input.amount_lamports),
        network: Some(input.network),
        terms_digest: Some(input.terms_digest),
        release_authority_did: Some(input.release_authority_did),
        release_policy: Some(input.release_policy),
        fund_idempotency_key: Some(input.idempotency_key),
        settlement: ServiceApiSettlementMetadata::default(),
    }
}

pub(super) fn issue_release_grant(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
    authority: &str,
) {
    let key = format!("escrow-lifecycle:{escrow_id}:{authority}:escrow:release");
    store.snapshot.agent_grants.insert(
        key.clone(),
        ServiceApiPersistedAgentGrantRecord {
            did: authority.to_owned(),
            resource: format!("escrow:{escrow_id}"),
            role: "initiator".to_owned(),
            action: "escrow:release".to_owned(),
            status: "active".to_owned(),
            idempotency_key: key,
        },
    );
}

fn validate_participants(
    actor: &str,
    input: &FundInput,
    task: &ServiceApiPersistedTaskRecord,
) -> Result<(), EscrowLifecycleError> {
    if input.release_authority_did != actor {
        return Err(forbidden(
            "ESCROW_RELEASE_AUTHORITY_MISMATCH",
            "release authority differs",
        ));
    }
    if task.provider_did.as_deref() != Some(input.beneficiary_did.as_str()) {
        return Err(conflict(
            "ESCROW_BENEFICIARY_MISMATCH",
            "beneficiary differs",
        ));
    }
    Ok(())
}

fn validate_terms(
    input: &FundInput,
    task: &ServiceApiPersistedTaskRecord,
) -> Result<(), EscrowLifecycleError> {
    if task.transaction_id.as_deref() != Some(input.transaction_id.as_str()) {
        return Err(conflict(
            "ESCROW_TRANSACTION_MISMATCH",
            "transaction differs",
        ));
    }
    if task.terms_digest.as_deref() != Some(input.terms_digest.as_str()) {
        return Err(conflict("ESCROW_TERMS_MISMATCH", "terms differ"));
    }
    Ok(())
}

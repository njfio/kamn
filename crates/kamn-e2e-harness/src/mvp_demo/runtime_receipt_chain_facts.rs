use super::pi_transaction_actor_model::Actor;
use super::pi_transaction_public_result::PublicResult;

pub(super) fn validate_step_public_result(
    actor: &Actor,
    action: &str,
    after_state: &str,
    result: &PublicResult,
) -> Result<(), String> {
    validate_present_facts(actor, result)?;
    if result
        .state
        .as_deref()
        .is_some_and(|state| state != after_state)
    {
        return Err(fact_mismatch());
    }
    match action {
        "register" => require_registration(actor, result),
        "create_task" | "accept_task" | "complete_task" => require_task(result),
        "fund_escrow" => require_funding(result),
        "release_escrow" => require_release(result),
        "query_participant_task_projection" => require_participant_projection(actor, result),
        "query_verifier_task_projection" => require_verifier_projection(result),
        _ => Err(invalid()),
    }
}

fn validate_present_facts(actor: &Actor, result: &PublicResult) -> Result<(), String> {
    validate_identity_facts(actor, result)?;
    validate_settlement_facts(actor, result)?;
    if result
        .amount_lamports
        .is_some_and(|amount| amount != actor.amount_lamports)
    {
        return Err(fact_mismatch());
    }
    Ok(())
}

fn validate_identity_facts(actor: &Actor, result: &PublicResult) -> Result<(), String> {
    require_optional_match(result.task_id.as_deref(), actor.task_id.as_str())?;
    require_optional_match(
        result.transaction_id.as_deref(),
        actor.transaction_id.as_str(),
    )?;
    require_optional_match(result.escrow_id.as_deref(), actor.escrow_id.as_str())?;
    require_optional_match(result.network.as_deref(), actor.network.as_str())?;
    Ok(())
}

fn validate_settlement_facts(actor: &Actor, result: &PublicResult) -> Result<(), String> {
    require_optional_match(
        result.settlement_tx_signature.as_deref(),
        actor.settlement_tx_signature.as_str(),
    )?;
    require_optional_match(
        result.settlement_commitment.as_deref(),
        actor.settlement_commitment.as_str(),
    )?;
    require_optional_match(
        result.public_commitment.as_deref(),
        actor.public_commitment.as_str(),
    )?;
    Ok(())
}

fn require_optional_match(actual: Option<&str>, expected: &str) -> Result<(), String> {
    if actual.is_some_and(|value| value != expected) {
        return Err(fact_mismatch());
    }
    Ok(())
}

fn require_registration(actor: &Actor, result: &PublicResult) -> Result<(), String> {
    if result.did.as_deref() == Some(actor.did.as_str()) {
        return Ok(());
    }
    Err(fact_mismatch())
}

fn require_task(result: &PublicResult) -> Result<(), String> {
    require_all([result.task_id.is_some(), result.state.is_some()])
}

fn require_funding(result: &PublicResult) -> Result<(), String> {
    require_all([result.escrow_id.is_some(), result.state.is_some()])
}

fn require_release(result: &PublicResult) -> Result<(), String> {
    require_all([result.escrow_id.is_some(), result.state.is_some()])
}

fn require_participant_projection(actor: &Actor, result: &PublicResult) -> Result<(), String> {
    let expected_role = if actor.actor == "agent_a" {
        "creator"
    } else {
        "provider"
    };
    require_projection(result)?;
    if result.view_scope.as_deref() == Some("participant-private")
        && result.participant_role.as_deref() == Some(expected_role)
    {
        return Ok(());
    }
    Err(fact_mismatch())
}

fn require_verifier_projection(result: &PublicResult) -> Result<(), String> {
    require_projection(result)?;
    if result.view_scope.as_deref() == Some("restricted-public")
        && result.participant_role.is_none()
    {
        return Ok(());
    }
    Err("RUNTIME_RECEIPT_CHAIN_VERIFIER_PRIVATE_LEAK".to_owned())
}

fn require_projection(result: &PublicResult) -> Result<(), String> {
    require_all([
        result.task_id.is_some(),
        result.escrow_id.is_some(),
        result.settlement_tx_signature.is_some(),
        result.settlement_commitment.is_some(),
        result.public_commitment.is_some(),
        result.view_scope.is_some(),
    ])
}

fn require_all<const N: usize>(fields: [bool; N]) -> Result<(), String> {
    if fields.into_iter().all(|present| present) {
        return Ok(());
    }
    Err(invalid())
}

fn invalid() -> String {
    "RUNTIME_RECEIPT_CHAIN_PUBLIC_RESULT_INVALID".to_owned()
}

fn fact_mismatch() -> String {
    "RUNTIME_RECEIPT_CHAIN_FACT_MISMATCH".to_owned()
}

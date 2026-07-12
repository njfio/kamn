use serde::Serialize;

use super::artifact_digest::attach_json_digest;
use super::pi_transaction_actor_model::{Actor, Outcome, RuntimeReceipt};
use super::pi_transaction_actor_verify::read_and_validate_actors;
use super::pi_transaction_public_result::PublicResult;
use super::runtime_receipt_chain_facts::validate_step_public_result;
use super::runtime_receipt_chain_precheck::precheck_required_operations;

const SCHEMA: &str = "kamn.mvp.runtime-receipt-chain.v1";

#[derive(Serialize)]
struct Chain<'a> {
    schema_version: &'static str,
    task_id: &'a str,
    transaction_id: &'a str,
    escrow_id: &'a str,
    amount_lamports: u64,
    network: &'a str,
    settlement_tx_signature: &'a str,
    settlement_commitment: &'a str,
    public_commitment: &'a str,
    steps: Vec<Step<'a>>,
    chain_digest: &'static str,
}

#[derive(Serialize)]
struct Step<'a> {
    actor: &'a str,
    action: &'a str,
    request_id: u64,
    outcome: &'a Outcome,
    response_digest: &'a str,
    state_domain: &'static str,
    before_state: &'static str,
    after_state: &'static str,
    public_result: &'a PublicResult,
}

struct RequiredStep {
    actor_index: usize,
    action: &'static str,
    domain: &'static str,
    before: &'static str,
    after: &'static str,
    final_match: bool,
}

/// Builds a digested transaction chain from three verified Pi actor artifacts.
pub fn build_runtime_receipt_chain_from_actor_paths(paths: &[String; 3]) -> Result<String, String> {
    precheck_required_operations(paths)?;
    let actors = read_and_validate_actors(paths).map_err(map_actor_error)?;
    let steps = required_steps()
        .iter()
        .map(|required| build_step(&actors, required))
        .collect::<Result<Vec<_>, _>>()?;
    let raw = serde_json::to_string(&Chain {
        schema_version: SCHEMA,
        task_id: actors[0].task_id.as_str(),
        transaction_id: actors[0].transaction_id.as_str(),
        escrow_id: actors[0].escrow_id.as_str(),
        amount_lamports: actors[0].amount_lamports,
        network: actors[0].network.as_str(),
        settlement_tx_signature: actors[0].settlement_tx_signature.as_str(),
        settlement_commitment: actors[0].settlement_commitment.as_str(),
        public_commitment: actors[0].public_commitment.as_str(),
        steps,
        chain_digest: "",
    })
    .map_err(|_| artifact_mismatch())?;
    Ok(attach_json_digest(raw, "chain_digest")?.json)
}

fn build_step<'a>(actors: &'a [Actor; 3], required: &RequiredStep) -> Result<Step<'a>, String> {
    let actor = &actors[required.actor_index];
    let receipt = select_receipt(actor, required)?;
    validate_step_public_result(
        actor,
        required.action,
        required.after,
        &receipt.public_result,
    )?;
    Ok(Step {
        actor: actor.actor.as_str(),
        action: required.action,
        request_id: receipt.request_id,
        outcome: &receipt.outcome,
        response_digest: receipt.digest.as_str(),
        state_domain: required.domain,
        before_state: required.before,
        after_state: required.after,
        public_result: &receipt.public_result,
    })
}

fn select_receipt<'a>(
    actor: &'a Actor,
    required: &RequiredStep,
) -> Result<&'a RuntimeReceipt, String> {
    let matches = actor
        .runtime_response_receipts
        .iter()
        .filter(|receipt| receipt.tool == required.action && receipt.outcome == Outcome::Success)
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return Err("RUNTIME_RECEIPT_CHAIN_STEP_MISSING".to_owned());
    }
    if !required.final_match && matches.len() != 1 {
        return Err("RUNTIME_RECEIPT_CHAIN_STEP_DUPLICATED".to_owned());
    }
    matches.last().copied().ok_or_else(missing)
}

fn required_steps() -> [RequiredStep; 11] {
    [
        required(0, "register", "identity", "none", "registered", false),
        required(1, "register", "identity", "none", "registered", false),
        required(2, "register", "identity", "none", "registered", false),
        required(0, "create_task", "task", "none", "submitted", false),
        required(1, "accept_task", "task", "submitted", "accepted", false),
        required(0, "fund_escrow", "escrow", "none", "funded", false),
        required(1, "complete_task", "task", "accepted", "completed", false),
        required(0, "release_escrow", "escrow", "funded", "released", false),
        required(
            0,
            "query_participant_task_projection",
            "projection",
            "completed",
            "completed",
            true,
        ),
        required(
            1,
            "query_participant_task_projection",
            "projection",
            "completed",
            "completed",
            true,
        ),
        required(
            2,
            "query_verifier_task_projection",
            "projection",
            "completed",
            "completed",
            true,
        ),
    ]
}

fn required(
    actor_index: usize,
    action: &'static str,
    domain: &'static str,
    before: &'static str,
    after: &'static str,
    final_match: bool,
) -> RequiredStep {
    RequiredStep {
        actor_index,
        action,
        domain,
        before,
        after,
        final_match,
    }
}

fn missing() -> String {
    "RUNTIME_RECEIPT_CHAIN_STEP_MISSING".to_owned()
}

fn artifact_mismatch() -> String {
    "RUNTIME_RECEIPT_CHAIN_ARTIFACT_MISMATCH".to_owned()
}

fn map_actor_error(error: String) -> String {
    match error.as_str() {
        "PI_RUNTIME_RECEIPT_MISMATCH" => "RUNTIME_RECEIPT_CHAIN_STEP_MISSING".to_owned(),
        "PI_TRANSACTION_FACT_MISMATCH" => "RUNTIME_RECEIPT_CHAIN_FACT_MISMATCH".to_owned(),
        "PI_VERIFIER_PRIVATE_LEAK" => "RUNTIME_RECEIPT_CHAIN_VERIFIER_PRIVATE_LEAK".to_owned(),
        _ => error,
    }
}

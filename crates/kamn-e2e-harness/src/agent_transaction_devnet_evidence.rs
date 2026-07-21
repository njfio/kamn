use serde_json::Value;

use super::agent_transaction_evidence::AgentTransactionEvidencePaths;
use super::agent_transaction_persisted_settlement::{
    read_persisted_settlement, ExpectedSettlement, PersistedSettlement,
};
use super::agent_transaction_rpc_artifact::{confirm_transfer, ConfirmedTransfer};
use super::AgentTransactionDemoConfig;
use crate::mvp_demo::{verify_pi_transaction_actor_paths, DevnetSettlementEvidence};

const EVIDENCE_ERROR: &str = "AGENT_TRANSACTION_SETTLEMENT_INVALID";

pub(super) fn collect_actor_settlement_evidence(
    config: &AgentTransactionDemoConfig,
    paths: &AgentTransactionEvidencePaths,
) -> Result<DevnetSettlementEvidence, String> {
    verify_pi_transaction_actor_paths(&paths.actors).map_err(settlement_error)?;
    let actor = read_actor(paths.actors[0].as_str())?;
    validate_actor_config(config, &actor)?;
    let persisted = persisted(config, &actor)?;
    let rpc = confirm_transfer(config, actor.signature.as_str())?;
    Ok(evidence(config, actor, persisted, rpc))
}

struct ActorSettlement {
    task_id: String,
    transaction_id: String,
    signature: String,
    escrow_id: String,
    lamports: u64,
    network: String,
    commitment: String,
}

fn read_actor(path: &str) -> Result<ActorSettlement, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| settlement_error(format!("actor read failed: {error}")))?;
    let value: Value = serde_json::from_str(raw.as_str())
        .map_err(|error| settlement_error(format!("actor JSON failed: {error}")))?;
    Ok(ActorSettlement {
        task_id: string_field(&value, "task_id")?,
        transaction_id: string_field(&value, "transaction_id")?,
        signature: string_field(&value, "settlement_tx_signature")?,
        escrow_id: string_field(&value, "escrow_id")?,
        lamports: u64_field(&value, "amount_lamports")?,
        network: string_field(&value, "network")?,
        commitment: string_field(&value, "settlement_commitment")?,
    })
}

fn validate_actor_config(
    config: &AgentTransactionDemoConfig,
    actor: &ActorSettlement,
) -> Result<(), String> {
    if actor.lamports == config.solana_lamports
        && actor.network == "solana-devnet"
        && actor.commitment == config.solana_commitment
    {
        return Ok(());
    }
    Err(settlement_error("actor settlement configuration mismatch"))
}

fn persisted(
    config: &AgentTransactionDemoConfig,
    actor: &ActorSettlement,
) -> Result<PersistedSettlement, String> {
    let state = std::path::Path::new(config.staging_root.as_str()).join("service-api-state.json");
    let expected = ExpectedSettlement {
        task_id: actor.task_id.as_str(),
        transaction_id: actor.transaction_id.as_str(),
        escrow_id: actor.escrow_id.as_str(),
        signature: actor.signature.as_str(),
        recipient: config.solana_recipient_pubkey.as_str(),
        amount: actor.lamports,
    };
    read_persisted_settlement(state.as_path(), &expected).map_err(settlement_error)
}

fn evidence(
    config: &AgentTransactionDemoConfig,
    actor: ActorSettlement,
    persisted: PersistedSettlement,
    rpc: ConfirmedTransfer,
) -> DevnetSettlementEvidence {
    let mut evidence = core_evidence(config, &actor, &rpc);
    apply_provenance(&mut evidence, actor, persisted, rpc);
    evidence
}

fn core_evidence(
    config: &AgentTransactionDemoConfig,
    actor: &ActorSettlement,
    rpc: &ConfirmedTransfer,
) -> DevnetSettlementEvidence {
    DevnetSettlementEvidence {
        network: "solana:devnet".to_owned(),
        execution_surface: "live-service-persisted-receipt".to_owned(),
        rpc_url: config.solana_rpc_url.clone(),
        payer_pubkey: rpc.payer.clone(),
        recipient_pubkey: config.solana_recipient_pubkey.clone(),
        lamports: actor.lamports,
        escrow_id: actor.escrow_id.clone(),
        settlement_tx_signature: actor.signature.clone(),
        settlement_commitment: "finalized".to_owned(),
        payer_balance_before: rpc.payer_before,
        payer_balance_after: rpc.payer_after,
        recipient_balance_before: rpc.recipient_before,
        recipient_balance_after: rpc.recipient_after,
        persisted_settlement_tx_signature: actor.signature.clone(),
        ..DevnetSettlementEvidence::default()
    }
}

fn apply_provenance(
    evidence: &mut DevnetSettlementEvidence,
    actor: ActorSettlement,
    persisted: PersistedSettlement,
    rpc: ConfirmedTransfer,
) {
    evidence.task_id = Some(actor.task_id);
    evidence.transaction_id = Some(persisted.transaction_id);
    evidence.terms_digest = Some(persisted.terms_digest);
    evidence.fee_lamports = Some(rpc.fee_lamports);
    evidence.settlement_receipt_hash = Some(persisted.receipt_hash);
    evidence.service_state_digest = Some(persisted.state_digest);
    evidence.settlement_intent_digest = Some(persisted.intent_digest);
    evidence.authoritative_rpc_artifact = Some(rpc.artifact_path.display().to_string());
}

fn string_field(value: &Value, field: &str) -> Result<String, String> {
    value[field]
        .as_str()
        .filter(|found| !found.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| settlement_error(format!("actor {field} missing")))
}

fn u64_field(value: &Value, field: &str) -> Result<u64, String> {
    value[field]
        .as_u64()
        .filter(|found| *found > 0)
        .ok_or_else(|| settlement_error(format!("actor {field} missing")))
}

fn settlement_error(message: impl AsRef<str>) -> String {
    format!("{EVIDENCE_ERROR}: {}", message.as_ref())
}

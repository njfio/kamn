use serde::{Deserialize, Serialize};
use std::path::Path;

use super::artifact_digest::{attach_json_digest, validate_json_digest};
use super::devnet_settlement::DevnetSettlementEvidence;

pub(super) const FILE_NAME: &str = "settlement-evidence.json";
const SCHEMA: &str = "kamn.mvp.offline-settlement-evidence.v1";

#[derive(Deserialize, Serialize)]
pub(super) struct SettlementEvidenceArtifact {
    schema_version: String,
    evidence_source: String,
    pub(super) network: String,
    pub(super) rpc_url: String,
    pub(super) payer_pubkey: String,
    pub(super) recipient_pubkey: String,
    pub(super) lamports: u64,
    pub(super) escrow_id: String,
    pub(super) task_id: Option<String>,
    pub(super) task_binding_digest: Option<String>,
    pub(super) settlement_tx_signature: String,
    pub(super) settlement_commitment: String,
    pub(super) payer_balance_before: u64,
    pub(super) payer_balance_after: u64,
    pub(super) recipient_balance_before: u64,
    pub(super) recipient_balance_after: u64,
    pub(super) persisted_settlement_tx_signature: String,
    evidence_digest: String,
}

pub(super) fn write_settlement_evidence_artifact(
    run_dir: &Path,
    evidence: &DevnetSettlementEvidence,
) -> Result<(), String> {
    let artifact = artifact_from_evidence(evidence);
    let raw = serde_json::to_string(&artifact).map_err(|_| invalid())?;
    let digested = attach_json_digest(raw, "evidence_digest").map_err(|_| invalid())?;
    std::fs::write(run_dir.join("proof").join(FILE_NAME), digested.json).map_err(|_| invalid())
}

pub(super) fn read_settlement_evidence_artifact(
    path: &Path,
) -> Result<SettlementEvidenceArtifact, String> {
    let raw = std::fs::read_to_string(path).map_err(|_| invalid())?;
    let artifact: SettlementEvidenceArtifact =
        serde_json::from_str(raw.as_str()).map_err(|_| invalid())?;
    validate_artifact(raw.as_str(), &artifact)?;
    Ok(artifact)
}

fn artifact_from_evidence(evidence: &DevnetSettlementEvidence) -> SettlementEvidenceArtifact {
    SettlementEvidenceArtifact {
        schema_version: SCHEMA.to_owned(),
        evidence_source: "solana-cli-confirm-and-balance-rpc".to_owned(),
        network: evidence.network.clone(),
        rpc_url: evidence.rpc_url.clone(),
        payer_pubkey: evidence.payer_pubkey.clone(),
        recipient_pubkey: evidence.recipient_pubkey.clone(),
        lamports: evidence.lamports,
        escrow_id: evidence.escrow_id.clone(),
        task_id: evidence.task_id.clone(),
        task_binding_digest: evidence.task_binding_digest.clone(),
        settlement_tx_signature: evidence.settlement_tx_signature.clone(),
        settlement_commitment: evidence.settlement_commitment.clone(),
        payer_balance_before: evidence.payer_balance_before,
        payer_balance_after: evidence.payer_balance_after,
        recipient_balance_before: evidence.recipient_balance_before,
        recipient_balance_after: evidence.recipient_balance_after,
        persisted_settlement_tx_signature: evidence.persisted_settlement_tx_signature.clone(),
        evidence_digest: String::new(),
    }
}

fn validate_artifact(raw: &str, artifact: &SettlementEvidenceArtifact) -> Result<(), String> {
    if artifact.schema_version != SCHEMA
        || artifact.evidence_source != "solana-cli-confirm-and-balance-rpc"
    {
        return Err(invalid());
    }
    validate_json_digest(
        raw,
        "evidence_digest",
        artifact.evidence_digest.as_str(),
        "settlement evidence",
    )
    .map_err(|_| invalid())
}

fn invalid() -> String {
    "SETTLEMENT_EVIDENCE_INVALID".to_owned()
}

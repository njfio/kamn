use serde::{Deserialize, Serialize};
use std::path::Path;

use super::artifact_digest::{attach_json_digest, sha256_hex, validate_json_digest};
use super::devnet_settlement::DevnetSettlementEvidence;

pub(super) const FILE_NAME: &str = "settlement-evidence.json";
pub(super) const RPC_FILE_NAME: &str = "solana-confirmation-response.json";
const SCHEMA: &str = "kamn.mvp.offline-settlement-evidence.v1";

#[derive(Deserialize, Serialize)]
pub(super) struct SettlementEvidenceArtifact {
    schema_version: String,
    pub(super) evidence_source: String,
    pub(super) execution_surface: String,
    pub(super) network: String,
    pub(super) rpc_url: String,
    pub(super) payer_pubkey: String,
    pub(super) recipient_pubkey: String,
    pub(super) lamports: u64,
    pub(super) escrow_id: String,
    pub(super) task_id: Option<String>,
    pub(super) transaction_id: Option<String>,
    pub(super) terms_digest: Option<String>,
    pub(super) task_binding_digest: Option<String>,
    pub(super) settlement_tx_signature: String,
    pub(super) settlement_commitment: String,
    pub(super) payer_balance_before: u64,
    pub(super) payer_balance_after: u64,
    pub(super) recipient_balance_before: u64,
    pub(super) recipient_balance_after: u64,
    pub(super) fee_lamports: Option<u64>,
    pub(super) settlement_receipt_hash: Option<String>,
    pub(super) persisted_settlement_tx_signature: String,
    pub(super) service_state_digest: Option<String>,
    pub(super) settlement_intent_digest: Option<String>,
    pub(super) receipt_chain_commitment: Option<String>,
    pub(super) authoritative_rpc_digest: Option<String>,
    evidence_digest: String,
}

pub(super) fn write_settlement_evidence_artifact(
    run_dir: &Path,
    evidence: &DevnetSettlementEvidence,
) -> Result<(), String> {
    let rpc_digest = copy_authoritative_rpc(run_dir, evidence)?;
    let artifact = artifact_from_evidence(evidence, rpc_digest);
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

fn artifact_from_evidence(
    evidence: &DevnetSettlementEvidence,
    authoritative_rpc_digest: Option<String>,
) -> SettlementEvidenceArtifact {
    SettlementEvidenceArtifact {
        schema_version: SCHEMA.to_owned(),
        evidence_source: evidence_source(evidence).to_owned(),
        execution_surface: evidence.execution_surface.clone(),
        network: evidence.network.clone(),
        rpc_url: evidence.rpc_url.clone(),
        payer_pubkey: evidence.payer_pubkey.clone(),
        recipient_pubkey: evidence.recipient_pubkey.clone(),
        lamports: evidence.lamports,
        escrow_id: evidence.escrow_id.clone(),
        task_id: evidence.task_id.clone(),
        transaction_id: evidence.transaction_id.clone(),
        terms_digest: evidence.terms_digest.clone(),
        task_binding_digest: evidence.task_binding_digest.clone(),
        settlement_tx_signature: evidence.settlement_tx_signature.clone(),
        settlement_commitment: evidence.settlement_commitment.clone(),
        payer_balance_before: evidence.payer_balance_before,
        payer_balance_after: evidence.payer_balance_after,
        recipient_balance_before: evidence.recipient_balance_before,
        recipient_balance_after: evidence.recipient_balance_after,
        fee_lamports: evidence.fee_lamports,
        settlement_receipt_hash: evidence.settlement_receipt_hash.clone(),
        persisted_settlement_tx_signature: evidence.persisted_settlement_tx_signature.clone(),
        service_state_digest: evidence.service_state_digest.clone(),
        settlement_intent_digest: evidence.settlement_intent_digest.clone(),
        receipt_chain_commitment: evidence.receipt_chain_commitment.clone(),
        authoritative_rpc_digest,
        evidence_digest: String::new(),
    }
}

fn copy_authoritative_rpc(
    run_dir: &Path,
    evidence: &DevnetSettlementEvidence,
) -> Result<Option<String>, String> {
    let Some(source) = evidence.authoritative_rpc_artifact.as_deref() else {
        return Ok(None);
    };
    let raw = std::fs::read_to_string(source).map_err(|_| invalid())?;
    std::fs::write(run_dir.join("proof").join(RPC_FILE_NAME), raw.as_bytes())
        .map_err(|_| invalid())?;
    Ok(Some(format!("sha256:{}", sha256_hex(raw.as_str()))))
}

fn validate_artifact(raw: &str, artifact: &SettlementEvidenceArtifact) -> Result<(), String> {
    if artifact.schema_version != SCHEMA || !valid_source(artifact) {
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

fn evidence_source(evidence: &DevnetSettlementEvidence) -> &'static str {
    if evidence.execution_surface == "live-service-persisted-receipt" {
        "kamn-live-service-persisted-receipt+solana-rpc"
    } else {
        "solana-cli-confirm-and-balance-rpc"
    }
}

fn valid_source(artifact: &SettlementEvidenceArtifact) -> bool {
    match artifact.execution_surface.as_str() {
        "live-service-persisted-receipt" => {
            artifact.evidence_source == "kamn-live-service-persisted-receipt+solana-rpc"
                && artifact
                    .receipt_chain_commitment
                    .as_deref()
                    .is_some_and(is_sha256)
        }
        _ => artifact.evidence_source == "solana-cli-confirm-and-balance-rpc",
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn invalid() -> String {
    "SETTLEMENT_EVIDENCE_INVALID".to_owned()
}

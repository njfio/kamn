use serde_json::Value;

use super::settlement_evidence_artifact::SettlementEvidenceArtifact;

pub(super) fn validate_settlement(
    report: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    let claim = claim(report)?;
    validate_claim_strings(claim, evidence)?;
    require_claim_u64(claim, "lamports", evidence.lamports)?;
    validate_finality(evidence)?;
    validate_balances(evidence)
}

fn validate_claim_strings(
    claim: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    validate_claim_context(claim, evidence)?;
    validate_claim_signatures(claim, evidence)?;
    validate_optional_binding(claim, evidence)
}

fn validate_claim_context(
    claim: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    require_claim_string(claim, "network", evidence.network.as_str())?;
    require_claim_string(
        claim,
        "execution_surface",
        evidence.execution_surface.as_str(),
    )?;
    require_claim_string(claim, "rpc_url", evidence.rpc_url.as_str())?;
    require_claim_string(claim, "payer_pubkey", evidence.payer_pubkey.as_str())?;
    require_claim_string(
        claim,
        "recipient_pubkey",
        evidence.recipient_pubkey.as_str(),
    )?;
    require_claim_string(claim, "escrow_id", evidence.escrow_id.as_str())
}

fn validate_claim_signatures(
    claim: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    require_claim_string(
        claim,
        "settlement_tx_signature",
        evidence.settlement_tx_signature.as_str(),
    )?;
    require_claim_string(
        claim,
        "persisted_settlement_tx_signature",
        evidence.persisted_settlement_tx_signature.as_str(),
    )
}

fn validate_optional_binding(
    claim: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    if let Some(task_id) = evidence.task_id.as_deref() {
        require_claim_string(claim, "task_id", task_id)?;
    }
    if let Some(digest) = evidence.task_binding_digest.as_deref() {
        require_claim_string(claim, "task_binding_digest", digest)?;
    }
    Ok(())
}

fn validate_finality(evidence: &SettlementEvidenceArtifact) -> Result<(), String> {
    if evidence.network == "solana:devnet"
        && evidence.settlement_commitment == "finalized"
        && evidence.settlement_tx_signature == evidence.persisted_settlement_tx_signature
    {
        return Ok(());
    }
    Err(invalid())
}

fn validate_balances(evidence: &SettlementEvidenceArtifact) -> Result<(), String> {
    let payer_delta = evidence
        .payer_balance_before
        .checked_sub(evidence.payer_balance_after)
        .ok_or_else(invalid)?;
    let recipient_delta = evidence
        .recipient_balance_after
        .checked_sub(evidence.recipient_balance_before)
        .ok_or_else(invalid)?;
    if payer_delta >= evidence.lamports && recipient_delta == evidence.lamports {
        return Ok(());
    }
    Err(invalid())
}

fn claim(report: &Value) -> Result<&Value, String> {
    report["claim_matrix"]
        .as_array()
        .and_then(|claims| {
            claims
                .iter()
                .find(|claim| claim["id"] == "devnet_settlement_asset_movement")
        })
        .ok_or_else(invalid)
}

fn require_claim_string(claim: &Value, field: &str, expected: &str) -> Result<(), String> {
    if claim[field].as_str() == Some(expected) {
        return Ok(());
    }
    Err(invalid())
}

fn require_claim_u64(claim: &Value, field: &str, expected: u64) -> Result<(), String> {
    if claim[field].as_u64() == Some(expected) {
        return Ok(());
    }
    Err(invalid())
}

fn invalid() -> String {
    "SETTLEMENT_EVIDENCE_INVALID".to_owned()
}

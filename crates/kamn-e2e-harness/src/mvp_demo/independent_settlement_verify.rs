use serde_json::Value;

use super::settlement_evidence_artifact::SettlementEvidenceArtifact;

pub(super) fn validate_settlement(
    report: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    let claim = claim(report)?;
    validate_claim_strings(claim, evidence)?;
    require_claim_u64(claim, "lamports", evidence.lamports)?;
    validate_direct_provenance(claim, evidence)?;
    validate_finality(evidence)?;
    validate_balances(evidence)
}

fn validate_direct_provenance(
    claim: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    if evidence.execution_surface != "live-service-persisted-receipt" {
        return Ok(());
    }
    for (field, expected) in provenance_fields(evidence) {
        require_claim_string(claim, field, expected.ok_or_else(invalid)?)?;
    }
    require_claim_u64(
        claim,
        "fee_lamports",
        evidence.fee_lamports.ok_or_else(invalid)?,
    )?;
    validate_provenance_shape(evidence)
}

fn provenance_fields(evidence: &SettlementEvidenceArtifact) -> [(&'static str, Option<&str>); 5] {
    [
        ("transaction_id", evidence.transaction_id.as_deref()),
        ("terms_digest", evidence.terms_digest.as_deref()),
        (
            "settlement_receipt_hash",
            evidence.settlement_receipt_hash.as_deref(),
        ),
        (
            "service_state_digest",
            evidence.service_state_digest.as_deref(),
        ),
        (
            "settlement_intent_digest",
            evidence.settlement_intent_digest.as_deref(),
        ),
    ]
}

fn validate_provenance_shape(evidence: &SettlementEvidenceArtifact) -> Result<(), String> {
    let valid_digests = [
        evidence.service_state_digest.as_deref(),
        evidence.settlement_intent_digest.as_deref(),
    ]
    .into_iter()
    .all(|value| value.is_some_and(is_sha256));
    let receipt_matches = evidence.settlement_receipt_hash.as_deref()
        == Some(evidence.settlement_tx_signature.as_str());
    if valid_digests && receipt_matches {
        return Ok(());
    }
    Err(invalid())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
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
    let expected_payer = evidence
        .fee_lamports
        .and_then(|fee| evidence.lamports.checked_add(fee));
    let valid_payer = match evidence.execution_surface.as_str() {
        "live-service-persisted-receipt" => Some(payer_delta) == expected_payer,
        _ => payer_delta >= evidence.lamports,
    };
    if valid_payer && recipient_delta == evidence.lamports {
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

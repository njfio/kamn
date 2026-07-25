use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
/// Driver-neutral projection of service-issued settlement authority.
pub struct AuthoritativeSettlementObservation {
    /// Bridge operation identifier.
    pub bridge_id: String,
    /// Finalized bridge receipt identifier.
    pub bridge_receipt_id: String,
    /// Finalized bridge receipt digest.
    pub bridge_receipt_digest: String,
    /// Settlement receipt identifier.
    pub settlement_receipt_id: String,
    /// Settlement receipt digest.
    pub settlement_receipt_digest: String,
    /// Canonical settlement action.
    pub action: String,
    /// Bound resource identifier.
    pub resource_id: String,
    /// Authenticated actor DID.
    pub actor_did: String,
    /// Resulting settlement state.
    pub resulting_state: String,
    /// Bound task identifier.
    pub task_id: String,
    /// Bound escrow identifier.
    pub escrow_id: String,
    /// Settlement recipient.
    pub recipient: String,
    /// Settled amount in lamports.
    pub amount_lamports: u64,
    /// Settled asset marker.
    pub asset: String,
    /// Settlement network.
    pub network: String,
    /// Finalized transaction signature.
    pub transaction_signature: String,
    /// Finality commitment.
    pub commitment: String,
    /// Finalized network slot.
    pub finalized_slot: u64,
    /// Full receipt-chain commitment.
    pub receipt_chain_commitment: String,
    /// Economic terms commitment.
    pub terms_digest: String,
    /// Durable operation identity.
    pub idempotency_key: String,
}

/// Normalizes and validates one service-issued settlement authority object.
pub fn normalize_authoritative_settlement(
    value: &Value,
    expected_escrow: &str,
    expected_actor: &str,
) -> Result<AuthoritativeSettlementObservation, String> {
    let source = value.get("authoritative_settlement").unwrap_or(value);
    let observation = serde_json::from_value::<AuthoritativeSettlementObservation>(source.clone())
        .map_err(|_| "SERVICE_AUTHORITY_MISSING".to_owned())?;
    validate_receipt_chain_commitment(&observation)?;
    validate_bridge_receipt_digest(&observation)?;
    validate_settlement_receipt_digest(&observation)?;
    validate_economic_terms(&observation, expected_escrow, expected_actor)?;
    Ok(observation)
}

fn validate_receipt_chain_commitment(
    value: &AuthoritativeSettlementObservation,
) -> Result<(), String> {
    validate_digest(value.receipt_chain_commitment.as_str())
}

fn validate_bridge_receipt_digest(
    value: &AuthoritativeSettlementObservation,
) -> Result<(), String> {
    validate_digest(value.bridge_receipt_digest.as_str())
}

fn validate_settlement_receipt_digest(
    value: &AuthoritativeSettlementObservation,
) -> Result<(), String> {
    validate_digest(value.settlement_receipt_digest.as_str())
}

fn validate_economic_terms(
    value: &AuthoritativeSettlementObservation,
    expected_escrow: &str,
    expected_actor: &str,
) -> Result<(), String> {
    let valid = value.escrow_id == expected_escrow
        && value.resource_id == expected_escrow
        && value.actor_did == expected_actor
        && value.action == "settlement:confirmed"
        && value.resulting_state == "confirmed"
        && value.network == "solana:devnet"
        && value.commitment == "finalized"
        && value.finalized_slot > 0
        && value.amount_lamports > 0
        && !value.terms_digest.is_empty()
        && required_values(value).iter().all(|field| !field.is_empty());
    valid
        .then_some(())
        .ok_or_else(|| "SERVICE_AUTHORITY_MISMATCH".to_owned())
}

fn required_values(value: &AuthoritativeSettlementObservation) -> [&str; 9] {
    [
        value.bridge_id.as_str(),
        value.bridge_receipt_id.as_str(),
        value.settlement_receipt_id.as_str(),
        value.task_id.as_str(),
        value.recipient.as_str(),
        value.asset.as_str(),
        value.transaction_signature.as_str(),
        value.idempotency_key.as_str(),
        value.actor_did.as_str(),
    ]
}

fn validate_digest(value: &str) -> Result<(), String> {
    let valid = value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    valid
        .then_some(())
        .ok_or_else(|| "SERVICE_AUTHORITY_DIGEST_INVALID".to_owned())
}

#[derive(Debug, Default)]
/// Rejects reordered operations and cross-resource receipt replay.
pub struct AuthoritativeSettlementReplayGuard {
    operations: HashMap<String, AuthoritativeSettlementObservation>,
    bridge_receipts: HashMap<String, String>,
}

impl AuthoritativeSettlementReplayGuard {
    /// Records one observation or rejects conflicting reuse.
    pub fn observe(&mut self, value: &AuthoritativeSettlementObservation) -> Result<(), String> {
        reject_replay(&self.operations, &self.bridge_receipts, value)?;
        self.operations
            .insert(value.idempotency_key.clone(), value.clone());
        self.bridge_receipts
            .insert(value.bridge_receipt_digest.clone(), value.escrow_id.clone());
        Ok(())
    }
}

fn reject_replay(
    operations: &HashMap<String, AuthoritativeSettlementObservation>,
    receipts: &HashMap<String, String>,
    value: &AuthoritativeSettlementObservation,
) -> Result<(), String> {
    if operations
        .get(value.idempotency_key.as_str())
        .is_some_and(|existing| existing != value)
        || receipts
            .get(value.bridge_receipt_digest.as_str())
            .is_some_and(|escrow| escrow != &value.escrow_id)
    {
        return Err("SERVICE_AUTHORITY_REPLAY".to_owned());
    }
    Ok(())
}

use super::{SdkError, ServiceAuthoritativeSettlement, ServiceSettlementReceipt};
use serde_json::Value;

pub(super) fn parse_authoritative_settlement(
    body: &str,
    expected_escrow: &str,
    receipt: Option<&ServiceSettlementReceipt>,
) -> Result<Option<ServiceAuthoritativeSettlement>, SdkError> {
    let root = serde_json::from_str::<Value>(body).map_err(|_| malformed())?;
    let Some(value) = root.get("authoritative_settlement") else {
        return if root.get("bridge_id").is_some() {
            Err(missing())
        } else {
            Ok(None)
        };
    };
    let authority = parse_fields(value)?;
    validate(&authority, expected_escrow, receipt)?;
    Ok(Some(authority))
}

fn validate(
    authority: &ServiceAuthoritativeSettlement,
    expected_escrow: &str,
    receipt: Option<&ServiceSettlementReceipt>,
) -> Result<(), SdkError> {
    let receipt_matches = receipt.is_some_and(|receipt| {
        receipt.receipt_id == authority.settlement_receipt_id
            && receipt.receipt_digest == authority.settlement_receipt_digest
            && receipt.action == authority.action
            && receipt.resource_id == authority.resource_id
            && receipt.state == authority.resulting_state
    });
    if receipt_matches && valid_bindings(authority, expected_escrow) {
        return Ok(());
    }
    Err(invalid())
}

fn valid_bindings(authority: &ServiceAuthoritativeSettlement, expected_escrow: &str) -> bool {
    [
        authority.bridge_receipt_digest.as_str(),
        authority.settlement_receipt_digest.as_str(),
        authority.receipt_chain_commitment.as_str(),
    ]
    .iter()
    .all(|value| valid_digest(value))
        && authority.action == "settlement:confirmed"
        && authority.resulting_state == "confirmed"
        && authority.resource_id == expected_escrow
        && authority.escrow_id == expected_escrow
        && authority.network == "solana:devnet"
        && authority.commitment == "finalized"
        && authority.finalized_slot > 0
        && authority.amount_lamports > 0
        && !authority.terms_digest.is_empty()
        && required_values(authority)
            .iter()
            .all(|value| !value.is_empty())
}

fn parse_fields(value: &Value) -> Result<ServiceAuthoritativeSettlement, SdkError> {
    Ok(ServiceAuthoritativeSettlement {
        bridge_id: field(value, "bridge_id")?,
        bridge_receipt_id: field(value, "bridge_receipt_id")?,
        bridge_receipt_digest: field(value, "bridge_receipt_digest")?,
        settlement_receipt_id: field(value, "settlement_receipt_id")?,
        settlement_receipt_digest: field(value, "settlement_receipt_digest")?,
        action: field(value, "action")?,
        resource_id: field(value, "resource_id")?,
        actor_did: field(value, "actor_did")?,
        resulting_state: field(value, "resulting_state")?,
        task_id: field(value, "task_id")?,
        escrow_id: field(value, "escrow_id")?,
        recipient: field(value, "recipient")?,
        amount_lamports: number(value, "amount_lamports")?,
        asset: field(value, "asset")?,
        network: field(value, "network")?,
        transaction_signature: field(value, "transaction_signature")?,
        commitment: field(value, "commitment")?,
        finalized_slot: number(value, "finalized_slot")?,
        receipt_chain_commitment: field(value, "receipt_chain_commitment")?,
        terms_digest: field(value, "terms_digest")?,
        idempotency_key: field(value, "idempotency_key")?,
    })
}

fn field(value: &Value, name: &str) -> Result<String, SdkError> {
    value
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(incomplete)
}

fn number(value: &Value, name: &str) -> Result<u64, SdkError> {
    value
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(incomplete)
}

fn required_values(authority: &ServiceAuthoritativeSettlement) -> [&str; 10] {
    [
        authority.bridge_id.as_str(),
        authority.bridge_receipt_id.as_str(),
        authority.settlement_receipt_id.as_str(),
        authority.actor_did.as_str(),
        authority.task_id.as_str(),
        authority.recipient.as_str(),
        authority.asset.as_str(),
        authority.transaction_signature.as_str(),
        authority.idempotency_key.as_str(),
        authority.resource_id.as_str(),
    ]
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn malformed() -> SdkError {
    SdkError::TransportFailure("service response payload was not valid json")
}

fn missing() -> SdkError {
    SdkError::TransportFailure("service response authoritative settlement was missing")
}

fn incomplete() -> SdkError {
    SdkError::TransportFailure("service response authoritative settlement was incomplete")
}

fn invalid() -> SdkError {
    SdkError::TransportFailure("service response authoritative settlement was invalid")
}

#[cfg(test)]
#[path = "service_authoritative_settlement_tests.rs"]
mod tests;

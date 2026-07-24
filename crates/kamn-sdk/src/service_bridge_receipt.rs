use super::{json_string_field, SdkError, ServiceBridgeReceipt, ServiceBridgeStatus};

pub(super) fn parse_bridge_status(body: &str) -> Result<ServiceBridgeStatus, SdkError> {
    let bridge_id = json_string_field(body, "bridge_id")?;
    Ok(ServiceBridgeStatus {
        bridge_status: json_string_field(body, "bridge_status")?,
        target_message_id: json_string_field(body, "target_message_id")?,
        forward_tx_hash: json_string_field(body, "forward_tx_hash")?,
        bridge_receipt: parse_bridge_receipt(body, bridge_id.as_str())?,
        bridge_id,
    })
}

fn parse_bridge_receipt(
    body: &str,
    expected_bridge: &str,
) -> Result<Option<ServiceBridgeReceipt>, SdkError> {
    let root = serde_json::from_str::<serde_json::Value>(body)
        .map_err(|_| SdkError::TransportFailure("service bridge response was not valid json"))?;
    let Some(value) = root.get("bridge_receipt") else {
        return Ok(None);
    };
    let receipt = bridge_receipt_fields(value)?;
    validate_bridge_receipt(&receipt, expected_bridge)?;
    Ok(Some(receipt))
}

fn bridge_receipt_fields(value: &serde_json::Value) -> Result<ServiceBridgeReceipt, SdkError> {
    Ok(ServiceBridgeReceipt {
        receipt_id: field(value, "receipt_id")?,
        receipt_digest: field(value, "receipt_digest")?,
        bridge_id: field(value, "bridge_id")?,
        transaction_signature: field(value, "transaction_signature")?,
        network: field(value, "network")?,
        commitment: field(value, "commitment")?,
        finalized_slot: number(value, "finalized_slot")?,
        action: field(value, "action")?,
        resource_id: field(value, "resource_id")?,
        state: field(value, "state")?,
    })
}

fn validate_bridge_receipt(
    receipt: &ServiceBridgeReceipt,
    expected_bridge: &str,
) -> Result<(), SdkError> {
    let valid = valid_digest(receipt.receipt_digest.as_str())
        && receipt.bridge_id == expected_bridge
        && receipt.resource_id == expected_bridge
        && receipt.action == "bridge:finalize"
        && receipt.state == "finalized"
        && receipt.network == "solana:devnet"
        && receipt.commitment == "finalized"
        && receipt.finalized_slot > 0
        && !receipt.transaction_signature.is_empty();
    valid.then_some(()).ok_or(SdkError::TransportFailure(
        "service bridge receipt was invalid",
    ))
}

fn field(value: &serde_json::Value, field: &str) -> Result<String, SdkError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or(SdkError::TransportFailure(
            "service bridge receipt was incomplete",
        ))
}

fn number(value: &serde_json::Value, field: &str) -> Result<u64, SdkError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or(SdkError::TransportFailure(
            "service bridge receipt was incomplete",
        ))
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finalized_bridge_receipt_is_preserved() {
        let body = fixture(&format!("sha256:{}", "a".repeat(64)), "bridge-1");
        let status = parse_bridge_status(body.as_str()).expect("bridge status");
        let receipt = status.bridge_receipt.expect("bridge receipt");
        assert_eq!(receipt.finalized_slot, 42);
        assert_eq!(receipt.action, "bridge:finalize");
    }

    #[test]
    fn tampered_or_cross_resource_bridge_receipt_fails_closed() {
        let bad_digest = fixture("sha256:bad", "bridge-1");
        assert!(parse_bridge_status(bad_digest.as_str()).is_err());
        let cross_resource = fixture(&format!("sha256:{}", "a".repeat(64)), "bridge-2");
        assert!(parse_bridge_status(cross_resource.as_str()).is_err());
    }

    fn fixture(digest: &str, resource: &str) -> String {
        serde_json::json!({
            "bridge_id": "bridge-1", "bridge_status": "finalized",
            "target_message_id": "message-1", "forward_tx_hash": "signature-1",
            "bridge_receipt": {
                "receipt_id": "receipt-1", "receipt_digest": digest,
                "bridge_id": "bridge-1", "transaction_signature": "signature-1",
                "network": "solana:devnet", "commitment": "finalized",
                "finalized_slot": 42, "action": "bridge:finalize",
                "resource_id": resource, "state": "finalized",
            }
        })
        .to_string()
    }
}

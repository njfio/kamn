use super::{SdkError, ServiceSettlementReceipt};
use serde_json::Value;

const FIELDS: [&str; 5] = [
    "settlement_receipt_id",
    "settlement_receipt_digest",
    "settlement_receipt_action",
    "settlement_receipt_resource_id",
    "settlement_receipt_state",
];

pub(super) fn parse_settlement_receipt(
    body: &str,
    expected_resource: &str,
) -> Result<Option<ServiceSettlementReceipt>, SdkError> {
    let root = serde_json::from_str::<Value>(body).map_err(|_| malformed())?;
    if FIELDS.iter().all(|field| root.get(field).is_none()) {
        return Ok(None);
    }
    let receipt = ServiceSettlementReceipt {
        receipt_id: field(&root, FIELDS[0])?,
        receipt_digest: field(&root, FIELDS[1])?,
        action: field(&root, FIELDS[2])?,
        resource_id: field(&root, FIELDS[3])?,
        state: field(&root, FIELDS[4])?,
    };
    validate(&receipt, expected_resource)?;
    Ok(Some(receipt))
}

fn validate(receipt: &ServiceSettlementReceipt, expected_resource: &str) -> Result<(), SdkError> {
    let digest = receipt.receipt_digest.strip_prefix("sha256:");
    let digest_valid = digest.is_some_and(|value| {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    if digest_valid
        && receipt.action == "settlement:confirmed"
        && receipt.resource_id == expected_resource
        && receipt.state == "confirmed"
    {
        return Ok(());
    }
    Err(invalid())
}

fn field(root: &Value, name: &str) -> Result<String, SdkError> {
    root.get(name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(incomplete)
}

fn malformed() -> SdkError {
    SdkError::TransportFailure("service response payload was not valid json")
}

fn incomplete() -> SdkError {
    SdkError::TransportFailure("service response settlement receipt was incomplete")
}

fn invalid() -> SdkError {
    SdkError::TransportFailure("service response settlement receipt was invalid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_settlement_receipt_remains_optional() {
        assert_eq!(
            parse_settlement_receipt(r#"{"escrow_id":"escrow-1"}"#, "escrow-1"),
            Ok(None)
        );
    }

    #[test]
    fn complete_settlement_receipt_is_preserved() {
        let body = complete_receipt(
            "settlement:confirmed",
            "escrow-1",
            "confirmed",
            &"a".repeat(64),
        );
        let receipt = parse_settlement_receipt(&body, "escrow-1")
            .expect("receipt should parse")
            .expect("receipt");
        assert_eq!(receipt.receipt_id, "intent-1");
        assert_eq!(receipt.resource_id, "escrow-1");
    }

    #[test]
    fn partial_settlement_receipt_fails_closed() {
        let error = parse_settlement_receipt(r#"{"settlement_receipt_id":"intent-1"}"#, "escrow-1")
            .expect_err("partial receipt must fail");
        assert!(error
            .to_string()
            .contains("settlement receipt was incomplete"));
    }

    #[test]
    fn invalid_settlement_receipt_bindings_fail_closed() {
        let digest = "a".repeat(64);
        assert_invalid_receipt("settlement:failed", "escrow-1", "confirmed", &digest);
        assert_invalid_receipt("settlement:confirmed", "escrow-2", "confirmed", &digest);
        assert_invalid_receipt("settlement:confirmed", "escrow-1", "pending", &digest);
        assert_invalid_receipt("settlement:confirmed", "escrow-1", "confirmed", "abc");
    }

    fn assert_invalid_receipt(action: &str, resource: &str, state: &str, digest: &str) {
        let body = complete_receipt(action, resource, state, digest);
        let error = parse_settlement_receipt(&body, "escrow-1")
            .expect_err("invalid settlement authority must fail");
        assert!(error.to_string().contains("settlement receipt was invalid"));
    }

    fn complete_receipt(action: &str, resource: &str, state: &str, digest: &str) -> String {
        format!(
            r#"{{"settlement_receipt_id":"intent-1","settlement_receipt_digest":"sha256:{digest}","settlement_receipt_action":"{action}","settlement_receipt_resource_id":"{resource}","settlement_receipt_state":"{state}"}}"#
        )
    }
}

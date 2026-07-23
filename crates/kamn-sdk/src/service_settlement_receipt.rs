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
) -> Result<Option<ServiceSettlementReceipt>, SdkError> {
    let root = serde_json::from_str::<Value>(body).map_err(|_| malformed())?;
    if FIELDS.iter().all(|field| root.get(field).is_none()) {
        return Ok(None);
    }
    Ok(Some(ServiceSettlementReceipt {
        receipt_id: field(&root, FIELDS[0])?,
        receipt_digest: field(&root, FIELDS[1])?,
        action: field(&root, FIELDS[2])?,
        resource_id: field(&root, FIELDS[3])?,
        state: field(&root, FIELDS[4])?,
    }))
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

use super::*;

const FIELDS: [&str; 5] = [
    "settlement_receipt_id",
    "settlement_receipt_digest",
    "settlement_receipt_action",
    "settlement_receipt_resource_id",
    "settlement_receipt_state",
];

pub(super) fn authority(
    value: &Value,
    tool: &str,
    actor: &str,
    primary_resource: &str,
) -> Result<Option<Value>, &'static str> {
    if let Some(authority) = value.get("authoritative_settlement") {
        return authoritative(authority, actor, primary_resource).map(Some);
    }
    if FIELDS.iter().all(|field| value.get(field).is_none()) {
        return Ok(None);
    }
    if tool != "release_escrow" {
        return Err(INVALID);
    }
    let fields = SettlementFields::parse(value)?;
    fields.validate(primary_resource)?;
    Ok(Some(fields.envelope(actor, tool)))
}

fn authoritative(
    value: &Value,
    actor: &str,
    primary_resource: &str,
) -> Result<Value, &'static str> {
    for digest in [
        "bridge_receipt_digest",
        "settlement_receipt_digest",
        "receipt_chain_commitment",
    ] {
        validate_digest(required(value, digest)?)?;
    }
    validate_equal(required(value, "actor_did")?, actor)?;
    validate_equal(required(value, "resource_id")?, primary_resource)?;
    validate_equal(required(value, "escrow_id")?, primary_resource)?;
    validate_equal(required(value, "action")?, "settlement:confirmed")?;
    validate_equal(required(value, "resulting_state")?, "confirmed")?;
    validate_equal(required(value, "network")?, "solana:devnet")?;
    validate_equal(required(value, "commitment")?, "finalized")?;
    validate_required_authority_fields(value)?;
    Ok(value.clone())
}

fn validate_required_authority_fields(value: &Value) -> Result<(), &'static str> {
    for field in [
        "bridge_id",
        "bridge_receipt_id",
        "settlement_receipt_id",
        "task_id",
        "recipient",
        "asset",
        "transaction_signature",
        "terms_digest",
        "idempotency_key",
    ] {
        required(value, field)?;
    }
    let amount = value
        .get("amount_lamports")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or(INVALID)?;
    let slot = value
        .get("finalized_slot")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or(INVALID)?;
    let _ = (amount, slot);
    Ok(())
}

struct SettlementFields<'a> {
    id: &'a str,
    digest: &'a str,
    action: &'a str,
    resource: &'a str,
    state: &'a str,
}

impl<'a> SettlementFields<'a> {
    fn parse(value: &'a Value) -> Result<Self, &'static str> {
        Ok(Self {
            id: required(value, FIELDS[0])?,
            digest: required(value, FIELDS[1])?,
            action: required(value, FIELDS[2])?,
            resource: required(value, FIELDS[3])?,
            state: required(value, FIELDS[4])?,
        })
    }

    fn validate(&self, primary_resource: &str) -> Result<(), &'static str> {
        validate_digest(self.digest)?;
        validate_equal(self.action, "settlement:confirmed")?;
        validate_equal(self.resource, primary_resource)?;
        validate_equal(self.state, "confirmed")
    }

    fn envelope(&self, actor: &str, tool: &str) -> Value {
        json!({
            "actor_did": actor, "tool": tool, "action": self.action,
            "resource_id": self.resource, "resulting_state": self.state,
            "service_receipt_id": self.id, "service_receipt_digest": self.digest,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_settlement_authority_fails_closed() {
        let value = json!({"settlement_receipt_id": "intent-1"});
        assert_eq!(
            authority(&value, "release_escrow", "did:a", "escrow-1"),
            Err(MISSING)
        );
    }

    #[test]
    fn tampered_settlement_authority_fails_closed() {
        let value = settlement_value("settlement:failed");
        assert_eq!(
            authority(&value, "release_escrow", "did:a", "escrow-1"),
            Err(INVALID)
        );
    }

    #[test]
    fn released_primary_authority_fails_closed() {
        assert_eq!(validate_state("release_escrow", "released"), Err(INVALID));
    }

    #[test]
    fn complete_authoritative_settlement_is_preserved() {
        let value = json!({"authoritative_settlement": authoritative_value()});
        let result = authority(&value, "release_escrow", "did:a", "escrow-1")
            .expect("authority should validate")
            .expect("authority");
        assert_eq!(result["bridge_id"], "bridge-1");
        assert_eq!(result["finalized_slot"], 42);
    }

    #[test]
    fn authoritative_settlement_rejects_actor_and_digest_tampering() {
        let mut actor = authoritative_value();
        actor["actor_did"] = json!("did:other");
        let value = json!({"authoritative_settlement": actor});
        assert_eq!(
            authority(&value, "release_escrow", "did:a", "escrow-1"),
            Err(INVALID)
        );

        let mut digest = authoritative_value();
        digest["bridge_receipt_digest"] = json!("sha256:bad");
        let value = json!({"authoritative_settlement": digest});
        assert_eq!(
            authority(&value, "release_escrow", "did:a", "escrow-1"),
            Err(INVALID)
        );
    }

    fn settlement_value(action: &str) -> Value {
        json!({
            "settlement_receipt_id": "intent-1", "settlement_receipt_digest": format!("sha256:{}", "a".repeat(64)),
            "settlement_receipt_action": action, "settlement_receipt_resource_id": "escrow-1",
            "settlement_receipt_state": "confirmed",
        })
    }

    fn authoritative_value() -> Value {
        json!({
            "bridge_id": "bridge-1", "bridge_receipt_id": "bridge-receipt-1",
            "bridge_receipt_digest": digest(), "settlement_receipt_id": "intent-1",
            "settlement_receipt_digest": digest(), "action": "settlement:confirmed",
            "resource_id": "escrow-1", "actor_did": "did:a", "resulting_state": "confirmed",
            "task_id": "task-1", "escrow_id": "escrow-1", "recipient": "recipient-1",
            "amount_lamports": 31, "asset": "lamports", "network": "solana:devnet",
            "transaction_signature": "signature-1", "commitment": "finalized",
            "finalized_slot": 42, "receipt_chain_commitment": digest(),
            "terms_digest": digest(), "idempotency_key": "operation-1",
        })
    }

    fn digest() -> String {
        format!("sha256:{}", "a".repeat(64))
    }
}

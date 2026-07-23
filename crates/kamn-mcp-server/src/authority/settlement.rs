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

    fn settlement_value(action: &str) -> Value {
        json!({
            "settlement_receipt_id": "intent-1", "settlement_receipt_digest": format!("sha256:{}", "a".repeat(64)),
            "settlement_receipt_action": action, "settlement_receipt_resource_id": "escrow-1",
            "settlement_receipt_state": "confirmed",
        })
    }
}

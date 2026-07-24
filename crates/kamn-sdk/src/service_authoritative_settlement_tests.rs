use super::*;
use serde_json::json;

#[test]
fn complete_authority_is_parsed_and_bound_to_receipt() {
    let receipt = receipt();
    let body = json!({
        "bridge_id": "bridge-1",
        "authoritative_settlement": authority(),
    })
    .to_string();
    let parsed = parse_authoritative_settlement(&body, "escrow-1", Some(&receipt))
        .expect("authority should parse")
        .expect("authority");
    assert_eq!(parsed.bridge_id, "bridge-1");
    assert_eq!(parsed.finalized_slot, 42);
}

#[test]
fn bridge_response_without_authority_fails_closed() {
    let error =
        parse_authoritative_settlement(r#"{"bridge_id":"bridge-1"}"#, "escrow-1", Some(&receipt()))
            .expect_err("bridge authority is required");
    assert!(error.to_string().contains("was missing"));
}

#[test]
fn partial_tampered_and_cross_resource_authority_fail_closed() {
    let receipt = receipt();
    let mut partial = authority();
    partial
        .as_object_mut()
        .expect("object")
        .remove("finalized_slot");
    assert_rejected(partial, &receipt);

    let mut tampered = authority();
    tampered["bridge_receipt_digest"] = json!("sha256:bad");
    assert_rejected(tampered, &receipt);

    let mut cross_resource = authority();
    cross_resource["escrow_id"] = json!("escrow-2");
    assert_rejected(cross_resource, &receipt);
}

fn assert_rejected(authority: Value, receipt: &ServiceSettlementReceipt) {
    let body = json!({
        "bridge_id": "bridge-1",
        "authoritative_settlement": authority,
    })
    .to_string();
    assert!(parse_authoritative_settlement(&body, "escrow-1", Some(receipt)).is_err());
}

fn receipt() -> ServiceSettlementReceipt {
    ServiceSettlementReceipt {
        receipt_id: "intent-1".to_owned(),
        receipt_digest: digest('b'),
        action: "settlement:confirmed".to_owned(),
        resource_id: "escrow-1".to_owned(),
        state: "confirmed".to_owned(),
    }
}

fn authority() -> Value {
    json!({
        "bridge_id": "bridge-1", "bridge_receipt_id": "bridge-receipt-1",
        "bridge_receipt_digest": digest('a'), "settlement_receipt_id": "intent-1",
        "settlement_receipt_digest": digest('b'), "action": "settlement:confirmed",
        "resource_id": "escrow-1", "actor_did": "did:actor", "resulting_state": "confirmed",
        "task_id": "task-1", "escrow_id": "escrow-1", "recipient": "recipient-1",
        "amount_lamports": 31, "asset": "lamports", "network": "solana:devnet",
        "transaction_signature": "signature-1", "commitment": "finalized",
        "finalized_slot": 42, "receipt_chain_commitment": digest('c'),
        "terms_digest": "terms-1", "idempotency_key": "operation-1",
    })
}

fn digest(value: char) -> String {
    format!("sha256:{}", value.to_string().repeat(64))
}

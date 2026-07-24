use kamn_e2e_harness::drivers::{
    normalize_authoritative_settlement, AuthoritativeSettlementReplayGuard,
};
use serde_json::{json, Value};

#[test]
fn equivalent_driver_payloads_normalize_identically() {
    let fixture = fixture();
    let sdk =
        normalize_authoritative_settlement(&fixture, "escrow-1", "did:actor").expect("SDK fixture");
    let cli =
        normalize_authoritative_settlement(&fixture, "escrow-1", "did:actor").expect("CLI fixture");
    let mcp =
        normalize_authoritative_settlement(&fixture, "escrow-1", "did:actor").expect("MCP fixture");
    assert_eq!(sdk, cli);
    assert_eq!(cli, mcp);
}

#[test]
fn missing_partial_tampered_and_cross_resource_authority_fail_closed() {
    let mut partial = fixture();
    partial
        .as_object_mut()
        .expect("object")
        .remove("bridge_receipt_digest");
    assert_rejected(partial);

    let mut tampered = fixture();
    tampered["settlement_receipt_digest"] = json!("sha256:bad");
    assert_rejected(tampered);

    let mut cross_resource = fixture();
    cross_resource["escrow_id"] = json!("escrow-2");
    assert_rejected(cross_resource);

    let mut wrong_actor = fixture();
    wrong_actor["actor_did"] = json!("did:other");
    assert_rejected(wrong_actor);
}

#[test]
fn replay_guard_accepts_idempotent_retry_and_rejects_reorder_or_reuse() {
    let observation =
        normalize_authoritative_settlement(&fixture(), "escrow-1", "did:actor").expect("fixture");
    let mut guard = AuthoritativeSettlementReplayGuard::default();
    guard.observe(&observation).expect("first observation");
    guard.observe(&observation).expect("idempotent retry");

    let mut reordered = fixture();
    reordered["finalized_slot"] = json!(41);
    let reordered = normalize_authoritative_settlement(&reordered, "escrow-1", "did:actor")
        .expect("reordered fixture");
    assert!(guard.observe(&reordered).is_err());

    let mut replayed = fixture();
    replayed["escrow_id"] = json!("escrow-2");
    replayed["resource_id"] = json!("escrow-2");
    let replayed =
        normalize_authoritative_settlement(&replayed, "escrow-2", "did:actor").expect("replay");
    assert!(guard.observe(&replayed).is_err());
}

fn assert_rejected(value: Value) {
    assert!(normalize_authoritative_settlement(&value, "escrow-1", "did:actor").is_err());
}

fn fixture() -> Value {
    json!({
        "bridge_id": "bridge-1",
        "bridge_receipt_id": "bridge-receipt-1",
        "bridge_receipt_digest": digest('a'),
        "settlement_receipt_id": "settlement-1",
        "settlement_receipt_digest": digest('b'),
        "action": "settlement:confirmed",
        "resource_id": "escrow-1",
        "actor_did": "did:actor",
        "resulting_state": "confirmed",
        "task_id": "task-1",
        "escrow_id": "escrow-1",
        "recipient": "recipient-1",
        "amount_lamports": 31,
        "asset": "lamports",
        "network": "solana:devnet",
        "transaction_signature": "signature-1",
        "commitment": "finalized",
        "finalized_slot": 42,
        "receipt_chain_commitment": digest('c'),
        "terms_digest": digest('d'),
        "idempotency_key": "operation-1"
    })
}

fn digest(value: char) -> String {
    format!("sha256:{}", value.to_string().repeat(64))
}

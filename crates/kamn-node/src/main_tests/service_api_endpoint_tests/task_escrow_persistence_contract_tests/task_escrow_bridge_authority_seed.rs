use super::super::super::support::read_state_json;
use super::{AssetMovementHarness, RECIPIENT_ENV};
use crate::service_api_endpoint::{
    bridge_receipt_digest, ServiceApiBridgeReceiptRecord, ServiceApiBridgeSettlementTermsRecord,
};
use serde_json::{json, Value};

pub(crate) fn seed_finalized_bridge_receipt(
    harness: &AssetMovementHarness,
    escrow_id: &str,
    bridge_id: &str,
) -> SeededBridgeReceipt {
    seed_finalized_bridge_receipt_with(harness, escrow_id, bridge_id, |_| {})
}

pub(crate) fn seed_finalized_bridge_receipt_with(
    harness: &AssetMovementHarness,
    escrow_id: &str,
    bridge_id: &str,
    mutate_terms: impl FnOnce(&mut ServiceApiBridgeSettlementTermsRecord),
) -> SeededBridgeReceipt {
    let mut state = read_state_json(harness.state_file.as_path());
    let escrow = &state["escrows"][escrow_id];
    let task_id = required_string(escrow, "task_id");
    let mut terms = ServiceApiBridgeSettlementTermsRecord {
        escrow_id: escrow_id.to_owned(),
        task_id: task_id.clone(),
        actor_did: required_string(escrow, "release_authority_did"),
        recipient_pubkey: std::env::var(RECIPIENT_ENV).expect("recipient env"),
        amount_lamports: escrow["amount_lamports"].as_u64().expect("amount"),
        asset: "lamports".to_owned(),
        network: "solana:devnet".to_owned(),
        terms_digest: required_string(escrow, "terms_digest"),
    };
    mutate_terms(&mut terms);
    let mut receipt = receipt(bridge_id, terms.clone());
    receipt.receipt_digest = bridge_receipt_digest(&receipt);
    persist_bridge(&mut state, bridge_id, &receipt, &terms);
    std::fs::write(
        harness.state_file.as_path(),
        serde_json::to_vec(&state).expect("bridge state"),
    )
    .expect("bridge state write");
    SeededBridgeReceipt {
        bridge_id: bridge_id.to_owned(),
        task_id,
        receipt_digest: receipt.receipt_digest,
        signature: receipt.transaction_signature,
    }
}

pub(crate) fn clone_replay_target(
    harness: &AssetMovementHarness,
    source_escrow: &str,
    target_escrow: &str,
) {
    let mut state = read_state_json(harness.state_file.as_path());
    state["escrows"][target_escrow] = state["escrows"][source_escrow].clone();
    state["escrows"][target_escrow]["escrow_id"] = json!(target_escrow);
    state["escrows"][target_escrow]["state"] = json!("funded");
    state["escrows"][target_escrow]["settlement_tx_signature"] = Value::Null;
    std::fs::write(
        harness.state_file.as_path(),
        serde_json::to_vec(&state).expect("replay target state"),
    )
    .expect("replay target state write");
}

fn receipt(
    bridge_id: &str,
    terms: ServiceApiBridgeSettlementTermsRecord,
) -> ServiceApiBridgeReceiptRecord {
    ServiceApiBridgeReceiptRecord {
        receipt_id: format!("bridge-receipt-{bridge_id}"),
        receipt_digest: String::new(),
        bridge_id: bridge_id.to_owned(),
        source_message_id: format!("msg-bridge-source-{bridge_id}"),
        target_network: "solana:devnet".to_owned(),
        payload_hash: "sha256:bridge-authority-payload".to_owned(),
        settlement_authority: Some(terms),
        transaction_signature: "11111111111111111111111111111111BridgeSettleSig".to_owned(),
        network: "solana:devnet".to_owned(),
        commitment: "finalized".to_owned(),
        finalized_slot: 42,
        action: "bridge:forward".to_owned(),
        resource_id: bridge_id.to_owned(),
        state: "finalized".to_owned(),
    }
}

fn persist_bridge(
    state: &mut Value,
    bridge_id: &str,
    receipt: &ServiceApiBridgeReceiptRecord,
    terms: &ServiceApiBridgeSettlementTermsRecord,
) {
    state["bridges"][bridge_id] = json!({
        "bridge_id": bridge_id,
        "source_message_id": receipt.source_message_id,
        "bridge_status": "finalized",
        "target_message_id": format!("msg-bridge-target-{bridge_id}"),
        "forward_tx_hash": receipt.transaction_signature,
        "target_network": "solana:devnet",
        "payload_hash": receipt.payload_hash,
        "settlement_authority": terms,
        "bridge_receipt": receipt,
        "submission_attempt_count": 1,
        "last_error_code": Value::Null
    });
}

fn required_string(value: &Value, field: &str) -> String {
    value[field].as_str().expect(field).to_owned()
}

pub(crate) struct SeededBridgeReceipt {
    pub(crate) bridge_id: String,
    pub(crate) task_id: String,
    pub(crate) receipt_digest: String,
    pub(crate) signature: String,
}

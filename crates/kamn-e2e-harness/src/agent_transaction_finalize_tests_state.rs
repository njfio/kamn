use std::path::Path;

pub(super) fn write(root: &Path, recipient: &str) {
    let path = root.join("staging/service-api-state.json");
    let state = serde_json::json!({
        "schema_version": "kamn.runtime.service-api-message-store.v4",
        "tasks": {"task-local-bound-7086": task()},
        "escrows": {"escrow-local-bound-7086": escrow()},
        "settlement_intents": {"escrow-local-bound-7086": intent(recipient)},
    });
    std::fs::write(path, serde_json::to_vec(&state).expect("state JSON"))
        .expect("persisted service state");
}

fn task() -> serde_json::Value {
    serde_json::json!({"task_id": "task-local-bound-7086", "state": "completed",
        "transaction_id": "transaction-live-7086", "terms_digest": "a".repeat(64)})
}

fn escrow() -> serde_json::Value {
    serde_json::json!({"escrow_id": "escrow-local-bound-7086", "state": "released",
        "task_id": "task-local-bound-7086", "transaction_id": "transaction-live-7086",
        "amount_lamports": 1000000, "network": "solana-devnet", "terms_digest": "a".repeat(64),
        "settlement_receipt_hash": "devnet-signature-111", "settlement_tx_signature": "devnet-signature-111",
        "settlement_network": "solana:devnet", "settlement_commitment": "finalized"})
}

fn intent(recipient: &str) -> serde_json::Value {
    serde_json::json!({"settlement_intent_id": "intent-local-bound-7086",
        "escrow_id": "escrow-local-bound-7086", "actor_did": "kamn:did:a",
        "idempotency_key": "release-local-bound-7086", "recipient_pubkey": recipient,
        "amount_lamports": 1000000, "network": "solana:devnet",
        "expected_signature": "devnet-signature-111",
        "signed_transaction_digest": format!("sha256:{}", "b".repeat(64)),
        "signed_transaction_json": "signed-transaction-secret", "state": "confirmed",
        "submission_attempt_count": 1})
}

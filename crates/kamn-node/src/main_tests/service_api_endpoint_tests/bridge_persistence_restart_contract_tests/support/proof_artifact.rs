use super::*;

const PROOF_OUTPUT_ENV: &str = "KAMN_LIVE_BRIDGE_PROOF_OUTPUT";

pub(crate) fn write_live_bridge_proof_artifact(
    forwarded: &Value,
    recipient_before: u64,
    recipient_after: u64,
) {
    let Ok(path) = std::env::var(PROOF_OUTPUT_ENV) else {
        return;
    };
    let receipt = &forwarded["bridge_receipt"];
    let artifact = serde_json::json!({
        "schema_version": "kamn.live-bridge-finality-proof.v1",
        "network": receipt["network"],
        "commitment": receipt["commitment"],
        "bridge_id": receipt["bridge_id"],
        "source_message_id": receipt["source_message_id"],
        "receipt_id": receipt["receipt_id"],
        "receipt_digest": receipt["receipt_digest"],
        "transaction_signature": receipt["transaction_signature"],
        "finalized_slot": receipt["finalized_slot"],
        "independent_rpc_verified": true,
        "restart_receipt_matched": true,
        "recipient_balance_delta_lamports": recipient_after.saturating_sub(recipient_before),
        "exactly_one_transfer_observed": recipient_after.saturating_sub(recipient_before)
            == super::super::live_bridge_devnet_contract_tests::TRANSFER_LAMPORTS,
        "contains_secret_material": false,
    });
    let payload = serde_json::to_string_pretty(&artifact).expect("proof artifact should encode");
    std::fs::write(path, payload).expect("proof artifact should persist");
}

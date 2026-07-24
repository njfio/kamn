use crate::service_api_endpoint::projection_models::ServiceApiTaskPublicProjection;
use k256::sha2::{Digest, Sha256};

pub(super) fn public_commitment(projection: &ServiceApiTaskPublicProjection) -> String {
    let amount = projection.amount_lamports.to_string();
    let fields = [
        projection.task_id.as_str(),
        projection.transaction_id.as_str(),
        projection.task_state.as_str(),
        projection.escrow_id.as_str(),
        projection.escrow_state.as_str(),
        amount.as_str(),
        projection.network.as_str(),
        projection.settlement_tx_signature.as_deref().unwrap_or(""),
        projection.settlement_commitment.as_deref().unwrap_or(""),
        projection.bridge_receipt_digest.as_deref().unwrap_or(""),
        projection
            .bridge_transaction_signature
            .as_deref()
            .unwrap_or(""),
        projection.receipt_chain_commitment.as_str(),
    ];
    let canonical = fields.map(length_frame).join("|");
    format!("sha256:{}", hex_digest(Sha256::digest(canonical)))
}

fn length_frame(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn hex_digest(digest: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in digest.as_ref() {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

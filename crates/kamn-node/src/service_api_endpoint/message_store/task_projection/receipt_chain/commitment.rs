use super::ReceiptChainEntry;
use k256::sha2::{Digest, Sha256};

const DOMAIN: &str = "kamn.service.receipt-chain.v1";

pub(super) fn chain(entries: &[ReceiptChainEntry]) -> String {
    let mut hasher = Sha256::new();
    append(&mut hasher, DOMAIN);
    append(&mut hasher, entries.len().to_string().as_str());
    for entry in entries {
        append_entry(&mut hasher, entry);
    }
    format!("sha256:{}", hex(&hasher.finalize()))
}

fn append_entry(hasher: &mut Sha256, entry: &ReceiptChainEntry) {
    for value in [
        entry.receipt_id.as_str(),
        entry.receipt_digest.as_str(),
        entry.authorization_digest.as_str(),
        entry.actor_did.as_str(),
        entry.action.as_str(),
        entry.resource_id.as_str(),
        entry.correlation_id.as_str(),
        entry.idempotency_key.as_str(),
        entry.prior_state.as_str(),
        entry.resulting_state.as_str(),
    ] {
        append(hasher, value);
    }
}

fn append(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

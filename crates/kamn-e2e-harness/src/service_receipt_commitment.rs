use sha2::{Digest, Sha256};

pub(crate) struct ReceiptProjection<'a> {
    pub(crate) actor_did: &'a str,
    pub(crate) action: &'a str,
    pub(crate) resource_id: &'a str,
    pub(crate) resulting_state: &'a str,
    pub(crate) receipt_id: &'a str,
    pub(crate) receipt_digest: &'a str,
}

pub(crate) fn commitment(receipts: &[ReceiptProjection<'_>]) -> String {
    let mut hasher = Sha256::new();
    append(&mut hasher, "kamn.service.receipt-projection.v1");
    append(&mut hasher, receipts.len().to_string().as_str());
    for receipt in receipts {
        append_receipt(&mut hasher, receipt);
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn append_receipt(hasher: &mut Sha256, receipt: &ReceiptProjection<'_>) {
    for field in [
        receipt.actor_did,
        receipt.action,
        receipt.resource_id,
        receipt.resulting_state,
        receipt.receipt_id,
        receipt.receipt_digest,
    ] {
        append(hasher, field);
    }
}

fn append(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

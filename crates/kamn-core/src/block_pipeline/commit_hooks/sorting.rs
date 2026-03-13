use crate::block_pipeline::block_pipeline_support::CanonicalCommitRecord;
use crate::transaction::BaselineTransaction;

pub(crate) fn sort_candidates_for_ingress(candidates: &mut [BaselineTransaction]) {
    candidates.sort_by(|left, right| {
        left.nonce
            .cmp(&right.nonce)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.sender.cmp(&right.sender))
    });
}

pub(crate) fn sort_canonical_candidates_for_reconciliation(
    candidates: &mut [CanonicalCommitRecord],
) {
    candidates.sort_by(|left, right| {
        left.block_height
            .cmp(&right.block_height)
            .then_with(|| left.payload_digest.cmp(&right.payload_digest))
            .then_with(|| {
                left.producer_role
                    .as_str()
                    .cmp(right.producer_role.as_str())
            })
            .then_with(|| left.transaction_ids.cmp(&right.transaction_ids))
    });
}

pub(crate) fn payload_digest_for_transactions(transactions: &[BaselineTransaction]) -> String {
    let mut ordered = transactions.to_vec();
    sort_candidates_for_ingress(&mut ordered);
    let mut digest = String::from("block-payload");
    for tx in ordered {
        digest.push('|');
        digest.push_str(&tx.id);
        digest.push(':');
        digest.push_str(&tx.sender);
        digest.push(':');
        digest.push_str(&tx.nonce.to_string());
        digest.push(':');
        digest.push_str(&tx.state_hash);
    }
    digest
}

use super::support::{approval_mut, challenge_mut, request_mut, result_mut, settlement_mut};
use kamn_e2e_harness::PeerReceiptAuthorityAttempt;

pub fn missing_fields() -> Vec<fn(&mut PeerReceiptAuthorityAttempt)> {
    vec![
        |value| challenge_mut(value).challenge_id.clear(),
        |value| challenge_mut(value).nonce.clear(),
        |value| approval_mut(value).payer.clear(),
        |value| settlement_mut(value).receipt_id.clear(),
        |value| settlement_mut(value).transaction_id.clear(),
        |value| result_mut(value).canonical_result.clear(),
    ]
}

pub fn digests() -> Vec<fn(&mut PeerReceiptAuthorityAttempt)> {
    vec![
        corrupt_request_digest,
        corrupt_challenge_digest,
        corrupt_approval_digest,
        corrupt_settlement_digest,
        corrupt_result_digest,
    ]
}

fn corrupt_request_digest(value: &mut PeerReceiptAuthorityAttempt) {
    corrupt(&mut request_mut(value).request_digest);
}

fn corrupt_challenge_digest(value: &mut PeerReceiptAuthorityAttempt) {
    corrupt(&mut challenge_mut(value).challenge_digest);
}

fn corrupt_approval_digest(value: &mut PeerReceiptAuthorityAttempt) {
    corrupt(&mut approval_mut(value).approval_digest);
}

fn corrupt_settlement_digest(value: &mut PeerReceiptAuthorityAttempt) {
    corrupt(&mut settlement_mut(value).settlement_digest);
}

fn corrupt_result_digest(value: &mut PeerReceiptAuthorityAttempt) {
    corrupt(&mut result_mut(value).result_digest);
}

fn corrupt(value: &mut String) {
    let replacement = if &value[7..8] == "0" { "1" } else { "0" };
    value.replace_range(7..8, replacement);
}

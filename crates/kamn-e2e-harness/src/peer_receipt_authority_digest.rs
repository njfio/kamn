use crate::{
    PeerApprovalAuthority, PeerChallengeAuthority, PeerResultAuthority, PeerSettlementAuthority,
};
use sha2::{Digest, Sha256};

/// Computes the digest of exact canonical request bytes.
pub fn peer_request_digest(canonical_body: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(canonical_body.as_bytes()))
}

/// Computes the canonical challenge commitment.
pub fn peer_challenge_digest(value: &PeerChallengeAuthority) -> String {
    digest_values(
        "kamn.peer.challenge.v1",
        vec![
            value.request_digest.clone(),
            value.challenge_id.clone(),
            value.nonce.clone(),
            value.expires_at_unix.to_string(),
            value.payer.clone(),
            value.payee.clone(),
            value.asset.clone(),
            value.network.clone(),
            value.amount_minor.to_string(),
        ],
    )
}

/// Computes the canonical approval commitment.
pub fn peer_approval_digest(value: &PeerApprovalAuthority) -> String {
    digest_values(
        "kamn.peer.approval.v1",
        vec![
            value.request_digest.clone(),
            value.challenge_digest.clone(),
            value.challenge_id.clone(),
            value.nonce.clone(),
            value.approved_at_unix.to_string(),
            value.payer.clone(),
            value.payee.clone(),
            value.asset.clone(),
            value.network.clone(),
            value.amount_minor.to_string(),
        ],
    )
}

/// Computes the canonical settlement commitment.
pub fn peer_settlement_digest(value: &PeerSettlementAuthority) -> String {
    digest_values(
        "kamn.peer.settlement.v1",
        vec![
            value.request_digest.clone(),
            value.challenge_digest.clone(),
            value.approval_digest.clone(),
            value.receipt_id.clone(),
            value.transaction_id.clone(),
            value.finalized_at_unix.to_string(),
            value.payer.clone(),
            value.payee.clone(),
            value.asset.clone(),
            value.network.clone(),
            value.amount_minor.to_string(),
        ],
    )
}

/// Computes the canonical service-result commitment.
pub fn peer_result_digest(value: &PeerResultAuthority) -> String {
    digest_values(
        "kamn.peer.result.v1",
        vec![
            value.request_digest.clone(),
            value.settlement_digest.clone(),
            value.canonical_result.clone(),
        ],
    )
}

fn digest_values(domain: &str, values: Vec<String>) -> String {
    let mut hasher = Sha256::new();
    append(&mut hasher, domain);
    for value in values {
        append(&mut hasher, value.as_str());
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn append(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

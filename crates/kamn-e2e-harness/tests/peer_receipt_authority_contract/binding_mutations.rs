use super::support::{approval_mut, result_mut, settlement_mut};
use kamn_e2e_harness::PeerReceiptAuthorityAttempt;

pub fn bindings() -> Vec<fn(&mut PeerReceiptAuthorityAttempt)> {
    vec![
        |value| approval_mut(value).request_digest = digest('f'),
        |value| approval_mut(value).challenge_id = "challenge-other".into(),
        |value| approval_mut(value).nonce = "nonce-other".into(),
        |value| approval_mut(value).payer = "did:peer:other".into(),
        |value| settlement_mut(value).payee = "wallet-other".into(),
        |value| settlement_mut(value).asset = "USDC".into(),
        |value| settlement_mut(value).network = "solana:mainnet".into(),
        |value| settlement_mut(value).amount_minor += 1,
        |value| result_mut(value).settlement_digest = digest('e'),
    ]
}

pub fn times() -> Vec<fn(&mut PeerReceiptAuthorityAttempt)> {
    vec![
        |value| approval_mut(value).approved_at_unix = 1_800_000_001,
        |value| settlement_mut(value).finalized_at_unix = 1_700_000_099,
        |value| result_mut(value).produced_at_unix = 1_700_000_199,
    ]
}

fn digest(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}

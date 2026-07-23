use super::SolanaSettlementFixture;
use super::Value;

pub(crate) fn assert_released_escrow_has_solana_signature_metadata(released_escrow: &Value) {
    assert_eq!(released_escrow["state"], "release-authorized");
    assert_eq!(released_escrow["settlement_network"], "solana:devnet");
    assert_eq!(released_escrow["settlement_commitment"], "finalized");
    assert_base58ish_signature(settlement_tx_signature(released_escrow));
}

pub(crate) fn assert_released_escrow_has_durable_authority(released_escrow: &Value) {
    assert_eq!(released_escrow["state"], "release-authorized");
    assert_eq!(released_escrow["action"], "escrow:release-authorize");
    assert!(released_escrow["receipt_id"]
        .as_str()
        .is_some_and(|value| value.starts_with("escrow-transition-receipt-")));
    assert!(released_escrow["receipt_digest"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
}

pub(crate) fn assert_replayed_release_authority(first: &Value, second: &Value) {
    assert_released_escrow_has_durable_authority(first);
    assert_released_escrow_has_durable_authority(second);
    assert_eq!(first["receipt_id"], second["receipt_id"]);
    assert_eq!(first["receipt_digest"], second["receipt_digest"]);
}

pub(crate) fn assert_persisted_solana_signature_metadata(state_json: &Value, escrow_id: &str) {
    let persisted = &state_json["escrows"][escrow_id];
    assert_eq!(persisted["state"], "released");
    assert_eq!(persisted["settlement_network"], "solana:devnet");
    assert_eq!(persisted["settlement_commitment"], "finalized");
    assert_base58ish_signature(settlement_tx_signature(persisted));
}

pub(crate) fn settlement_tx_signature(payload: &Value) -> &str {
    payload["settlement_tx_signature"]
        .as_str()
        .expect("release payload must expose a Solana transaction signature")
}

pub(crate) fn cleanup_state_file(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

pub(crate) fn cleanup_solana_settlement_fixture(fixture: &SolanaSettlementFixture) {
    let _ = std::fs::remove_file(fixture.keypair_file.as_path());
}

fn assert_base58ish_signature(signature: &str) {
    let valid = !signature.is_empty()
        && signature.chars().all(
            |ch| matches!(ch, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z'),
        );
    assert!(
        valid,
        "expected a base58ish Solana signature, got: {signature}"
    );
}

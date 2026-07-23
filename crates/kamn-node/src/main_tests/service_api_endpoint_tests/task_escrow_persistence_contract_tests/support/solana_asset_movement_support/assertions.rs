use super::SolanaSettlementFixture;
use super::{read_state_json, release_escrow_response, LiveSolanaAssetMovementContext, Value};

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

pub(crate) fn assert_released_escrow_has_settlement_authority(released_escrow: &Value) {
    assert_eq!(
        released_escrow["settlement_receipt_action"],
        "settlement:confirmed"
    );
    assert_eq!(released_escrow["settlement_receipt_state"], "confirmed");
    assert_eq!(
        released_escrow["settlement_receipt_resource_id"],
        released_escrow["escrow_id"]
    );
    assert!(released_escrow["settlement_receipt_id"]
        .as_str()
        .is_some_and(|value| value.starts_with("settlement-intent-")));
    assert!(released_escrow["settlement_receipt_digest"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
}

pub(crate) fn assert_replayed_release_authority(first: &Value, second: &Value) {
    assert_released_escrow_has_durable_authority(first);
    assert_released_escrow_has_durable_authority(second);
    assert_released_escrow_has_settlement_authority(first);
    assert_released_escrow_has_settlement_authority(second);
    assert_eq!(first["receipt_id"], second["receipt_id"]);
    assert_eq!(first["receipt_digest"], second["receipt_digest"]);
    assert_eq!(
        first["settlement_receipt_id"],
        second["settlement_receipt_id"]
    );
    assert_eq!(
        first["settlement_receipt_digest"],
        second["settlement_receipt_digest"]
    );
}

pub(crate) fn assert_replayed_live_release(
    context: &LiveSolanaAssetMovementContext,
    first: &Value,
    second: &Value,
) {
    assert_eq!(
        settlement_tx_signature(first),
        settlement_tx_signature(second)
    );
    assert_replayed_release_authority(first, second);
    let escrow_id = first["escrow_id"].as_str().expect("escrow ID");
    assert_tampered_settlement_authority_rejected(context, escrow_id);
}

pub(crate) fn assert_tampered_settlement_authority_rejected(
    context: &LiveSolanaAssetMovementContext,
    escrow_id: &str,
) {
    let mut state = read_state_json(context.harness.state_file.as_path());
    state["settlement_intents"][escrow_id]["actor_did"] = serde_json::json!("kamn:did:tampered");
    std::fs::write(
        context.harness.state_file.as_path(),
        serde_json::to_vec(&state).expect("tampered state should serialize"),
    )
    .expect("tampered state should persist");
    let response = release_escrow_response(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        context.harness.caller_did,
        115,
        escrow_id,
    );
    assert!(
        response.contains("SETTLEMENT_RECEIPT_INVALID"),
        "{response}"
    );
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

use super::super::*;
use super::support::read_state_json;
use serde_json::json;

#[path = "task_escrow_bridge_authority_support.rs"]
mod bridge_support;
use bridge_support::*;

#[test]
fn integration_bridge_authorized_release_reuses_finalized_bridge_receipt() {
    let _env = acquire_service_api_test_env();
    let _override = crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = bridge_context("bridge-authorized-release", "127.0.0.1:34271", 31);
    register_verifier(&context.harness, 150);
    let escrow_id = super::support::fund_live_escrow(&context.harness, 201, 31);
    let seeded =
        seed_finalized_bridge_receipt(&context.harness, escrow_id.as_str(), "bridge-auth-1");

    let first = release_with_bridge_authority(
        &context.harness,
        210,
        escrow_id.as_str(),
        seeded.bridge_id.as_str(),
    );
    let second = restart_release_with_bridge_authority(
        "127.0.0.1:34274",
        211,
        escrow_id.as_str(),
        seeded.bridge_id.as_str(),
        "bridge-release-210",
    );
    let projection = query_projection(
        &context.harness,
        ACTOR,
        212,
        seeded.task_id.as_str(),
        "participant-view",
    );
    let verifier = query_projection(
        &context.harness,
        VERIFIER,
        213,
        seeded.task_id.as_str(),
        "verifier-view",
    );
    let state = read_state_json(context.harness.state_file.as_path());

    assert_eq!(first["settlement_tx_signature"], seeded.signature);
    assert_eq!(first["settlement_receipt_hash"], seeded.signature);
    assert_eq!(first["bridge_receipt_digest"], seeded.receipt_digest);
    assert_eq!(first["bridge_transaction_signature"], seeded.signature);
    let authority = &first["authoritative_settlement"];
    assert_eq!(authority["bridge_id"], seeded.bridge_id);
    assert_eq!(authority["bridge_receipt_digest"], seeded.receipt_digest);
    assert_eq!(authority["transaction_signature"], seeded.signature);
    assert_eq!(authority["task_id"], seeded.task_id);
    assert_eq!(authority["escrow_id"], escrow_id);
    assert_eq!(authority["actor_did"], first["release_authority_did"]);
    assert_eq!(
        authority["recipient"],
        std::env::var(RECIPIENT_ENV).expect("recipient fixture")
    );
    assert_eq!(authority["amount_lamports"], 31);
    assert_eq!(authority["asset"], "lamports");
    assert_eq!(authority["network"], "solana:devnet");
    assert_eq!(authority["commitment"], "finalized");
    assert_eq!(authority["finalized_slot"], 42);
    assert_eq!(authority["action"], "settlement:confirmed");
    assert_eq!(authority["resource_id"], escrow_id);
    assert_eq!(authority["resulting_state"], "confirmed");
    assert!(authority["receipt_chain_commitment"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
    assert!(authority["terms_digest"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert_eq!(authority["idempotency_key"], "bridge-release-210");
    assert_eq!(
        first["settlement_receipt_id"],
        second["settlement_receipt_id"]
    );
    assert_eq!(
        first["authoritative_settlement"],
        second["authoritative_settlement"]
    );
    assert_eq!(
        first["settlement_receipt_digest"],
        second["settlement_receipt_digest"]
    );
    assert_eq!(
        state["settlement_intents"][&escrow_id]["bridge_id"],
        seeded.bridge_id
    );
    assert_eq!(
        state["settlement_intents"][&escrow_id]["bridge_receipt_digest"],
        seeded.receipt_digest
    );
    assert_eq!(
        state["settlement_intents"][&escrow_id]["submission_attempt_count"],
        0
    );
    assert_eq!(
        state["bridges"][&seeded.bridge_id]["submission_attempt_count"],
        1
    );
    assert_eq!(projection["bridge_receipt_digest"], seeded.receipt_digest);
    assert_eq!(projection["bridge_transaction_signature"], seeded.signature);
    assert_eq!(verifier["bridge_receipt_digest"], seeded.receipt_digest);
    assert_eq!(verifier["bridge_transaction_signature"], seeded.signature);
}

#[test]
fn integration_bridge_authorized_release_rejects_cross_resource_receipt() {
    let _env = acquire_service_api_test_env();
    let _override = crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = bridge_context("bridge-cross-resource", "127.0.0.1:34272", 41);
    let escrow_id = super::support::fund_live_escrow(&context.harness, 301, 41);
    seed_finalized_bridge_receipt_with(
        &context.harness,
        escrow_id.as_str(),
        "bridge-cross-resource",
        |terms| terms.escrow_id = "escrow-other".to_owned(),
    );

    let response = bridge_release_response(
        &context.harness,
        310,
        escrow_id.as_str(),
        "bridge-cross-resource",
        "bridge-release-cross-resource",
    );
    let state = read_state_json(context.harness.state_file.as_path());

    assert!(response.contains("HTTP/1.1 409 Conflict"), "{response}");
    assert!(
        response.contains("BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH"),
        "{response}"
    );
    assert!(state["settlement_intents"][&escrow_id].is_null());
    assert_eq!(state["escrows"][&escrow_id]["state"], "funded");
}

#[test]
fn integration_bridge_authorized_release_rejects_missing_and_tampered_receipts() {
    let _env = acquire_service_api_test_env();
    let _override = crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = bridge_context("bridge-invalid-authority", "127.0.0.1:34273", 51);
    let escrow_id = super::support::fund_live_escrow(&context.harness, 401, 51);

    let missing = bridge_release_response(
        &context.harness,
        410,
        escrow_id.as_str(),
        "bridge-missing",
        "bridge-release-missing",
    );
    assert!(
        missing.contains("BRIDGE_SETTLEMENT_AUTHORITY_MISSING"),
        "{missing}"
    );

    seed_finalized_bridge_receipt(&context.harness, escrow_id.as_str(), "bridge-tampered");
    let mut state = read_state_json(context.harness.state_file.as_path());
    state["bridges"]["bridge-tampered"]["bridge_receipt"]["receipt_digest"] =
        json!("sha256:tampered");
    std::fs::write(
        context.harness.state_file.as_path(),
        serde_json::to_vec(&state).expect("tampered state"),
    )
    .expect("tampered state write");
    let tampered = bridge_release_response(
        &context.harness,
        411,
        escrow_id.as_str(),
        "bridge-tampered",
        "bridge-release-tampered",
    );
    assert!(
        tampered.contains("BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH"),
        "{tampered}"
    );
    let state = read_state_json(context.harness.state_file.as_path());
    assert!(state["settlement_intents"][&escrow_id].is_null());
    assert_eq!(state["escrows"][&escrow_id]["state"], "funded");
}

#[test]
fn integration_bridge_authorized_release_rejects_cross_actor_and_replay() {
    let _env = acquire_service_api_test_env();
    let _override = crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = bridge_context("bridge-replay-authority", "127.0.0.1:34275", 61);
    let first_escrow = super::support::fund_live_escrow(&context.harness, 501, 61);
    seed_finalized_bridge_receipt_with(
        &context.harness,
        first_escrow.as_str(),
        "bridge-cross-actor",
        |terms| terms.actor_did = "kamn:did:agent:other".to_owned(),
    );
    let cross_actor = bridge_release_response(
        &context.harness,
        510,
        first_escrow.as_str(),
        "bridge-cross-actor",
        "bridge-release-cross-actor",
    );
    assert!(
        cross_actor.contains("BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH"),
        "{cross_actor}"
    );

    seed_finalized_bridge_receipt(&context.harness, first_escrow.as_str(), "bridge-replay");
    release_with_bridge_authority(&context.harness, 511, &first_escrow, "bridge-replay");
    let second_escrow = "escrow-replay-target";
    clone_replay_target(&context.harness, &first_escrow, second_escrow);
    let replay = bridge_release_response(
        &context.harness,
        530,
        second_escrow,
        "bridge-replay",
        "bridge-release-replay",
    );
    assert!(
        replay.contains("BRIDGE_SETTLEMENT_RECEIPT_REPLAY"),
        "{replay}"
    );
    let state = read_state_json(context.harness.state_file.as_path());
    assert_eq!(state["escrows"][second_escrow]["state"], "funded");
}

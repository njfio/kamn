use kamn_core::{
    BaselineTransaction, RoleSmokeNetwork, SmokeError, TransactionGuardError, GENESIS_STATE_HASH,
};

fn signed_tx(
    network: &RoleSmokeNetwork,
    id: &str,
    sender: &str,
    nonce: u64,
) -> BaselineTransaction {
    BaselineTransaction::signed(
        id,
        sender,
        nonce,
        &format!("payload-{id}"),
        network.expected_state_hash(),
    )
}

#[test]
fn functional_transaction_guards_advance_state_hash_after_commit() {
    let mut network = RoleSmokeNetwork::new(true);
    let initial_state_hash = network.expected_state_hash().to_owned();

    network
        .submit_transaction(signed_tx(&network, "tx-1", "agent-a", 1))
        .expect("transaction submit should succeed");
    network
        .produce_block()
        .expect("block production should succeed");

    assert_ne!(network.expected_state_hash(), initial_state_hash);
}

#[test]
fn integration_rejects_stale_state_hash_after_block_commit() {
    let mut network = RoleSmokeNetwork::new(true);
    let stale_state_hash = network.expected_state_hash().to_owned();

    network
        .submit_transaction(signed_tx(&network, "tx-1", "agent-a", 1))
        .expect("first transaction submit should succeed");
    network
        .produce_block()
        .expect("first block production should succeed");

    let stale_tx =
        BaselineTransaction::signed("tx-2", "agent-a", 2, "payload-tx-2", &stale_state_hash);
    assert!(matches!(
        network.submit_transaction(stale_tx),
        Err(SmokeError::Guard(
            TransactionGuardError::StateHashMismatch { .. }
        ))
    ));
}

#[test]
fn integration_rejects_out_of_sequence_nonce_per_sender() {
    let mut network = RoleSmokeNetwork::new(true);

    let out_of_sequence = BaselineTransaction::signed(
        "tx-1",
        "agent-a",
        2,
        "payload-tx-1",
        network.expected_state_hash(),
    );
    assert_eq!(
        network.submit_transaction(out_of_sequence),
        Err(SmokeError::Guard(
            TransactionGuardError::NonceOutOfSequence {
                sender: "agent-a".to_owned(),
                expected: 1,
                found: 2
            }
        ))
    );
}

#[test]
fn regression_tampered_signature_is_rejected() {
    // Regression: #78
    let mut network = RoleSmokeNetwork::new(true);
    let mut tx =
        BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-tx-1", GENESIS_STATE_HASH);
    tx.signature = format!("{}-tampered", tx.signature);

    assert!(matches!(
        network.submit_transaction(tx),
        Err(SmokeError::Guard(
            TransactionGuardError::InvalidSignature { .. }
        ))
    ));
}

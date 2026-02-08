use kamn_core::{
    classify_smoke_error, classify_transaction_guard_error, invariant_catalog, validate_catalog,
    BaselineTransaction, InvariantFailureCode, RoleSmokeNetwork, SmokeError, TransactionGuardError,
};

#[test]
fn functional_nonce_violation_maps_to_canonical_taxonomy() {
    let mut network = RoleSmokeNetwork::new(true);
    let tx = BaselineTransaction::signed(
        "tx-1",
        "agent-a",
        0,
        "payload-tx-1",
        network.expected_state_hash(),
    );

    let error = network
        .submit_transaction(tx)
        .expect_err("nonce 0 should be rejected");
    let violation = classify_smoke_error(&error).expect("guard failures should classify");

    assert_eq!(violation.invariant_id, "INV-TX-002");
    assert_eq!(violation.failure_code, InvariantFailureCode::InvalidNonce);
}

#[test]
fn integration_catalog_and_guard_taxonomy_are_consistent() {
    validate_catalog(invariant_catalog()).expect("catalog must validate");

    let violation = classify_transaction_guard_error(&TransactionGuardError::InvalidSignature {
        tx_id: "tx-1".to_owned(),
        expected: "sig:expected".to_owned(),
        found: "sig:found".to_owned(),
    });

    assert_eq!(violation.invariant_id, "INV-TX-003");
    assert_eq!(
        violation.failure_code.as_code(),
        "INV-TX-003-INVALID-SIGNATURE"
    );
}

#[test]
fn integration_non_guard_smoke_errors_do_not_produce_taxonomy() {
    let empty_mempool = SmokeError::EmptyMempool(kamn_core::NodeRole::Processor);
    assert!(classify_smoke_error(&empty_mempool).is_none());
}

#[test]
fn regression_stale_state_hash_maps_to_expected_invariant_id() {
    // Regression: #77
    let mut network = RoleSmokeNetwork::new(true);
    let stale_hash = network.expected_state_hash().to_owned();
    let tx1 = BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-tx-1", &stale_hash);
    network
        .submit_transaction(tx1)
        .expect("first transaction should be accepted");
    network
        .produce_block()
        .expect("block production should succeed");

    let stale_tx = BaselineTransaction::signed("tx-2", "agent-a", 2, "payload-tx-2", &stale_hash);
    let error = network
        .submit_transaction(stale_tx)
        .expect_err("stale state hash must be rejected");
    let violation = classify_smoke_error(&error).expect("guard failures should classify");

    assert_eq!(violation.invariant_id, "INV-TX-004");
    assert_eq!(
        violation.failure_code,
        InvariantFailureCode::StateHashMismatch
    );
}

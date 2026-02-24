use kamn_core::{
    classify_smoke_error, classify_transaction_guard_error, invariant_catalog, validate_catalog,
    BaselineTransaction, InvariantFailureCode, RoleSmokeNetwork, SmokeError, TransactionGuardError,
};
use std::sync::OnceLock;

const TEST_SIGNER_PRIVATE_KEY_A_HEX: &str =
    "7f2dcf2ef6bcf53b1af2359954f04eb6d25688fd87cbf09f7f9db4c6522f4c6b";

fn ensure_default_signer_key_env() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        std::env::set_var("KAMN_SIGNER_PRIVATE_KEY_HEX", TEST_SIGNER_PRIVATE_KEY_A_HEX);
        std::env::set_var(
            "KAMN_SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_HEX",
            TEST_SIGNER_PRIVATE_KEY_A_HEX,
        );
    });
}

#[test]
fn functional_nonce_violation_maps_to_canonical_taxonomy() {
    ensure_default_signer_key_env();
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
    ensure_default_signer_key_env();
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

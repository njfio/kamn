use super::transaction::{build_escrow_bound_transaction, validate_persisted_transaction};
use super::*;
use solana_sdk::{hash::Hash, pubkey::Pubkey, signature::Keypair};

#[test]
fn false_confirmation_is_an_ambiguous_outcome() {
    let error = require_confirmation(false, "finalized").expect_err("ambiguous result");

    assert!(error.starts_with("SETTLEMENT_OUTCOME_AMBIGUOUS"));
}

#[test]
fn escrow_binding_makes_same_blockhash_transfers_unique() {
    let payer = Keypair::new();
    let recipient = Pubkey::new_unique();
    let blockhash = Hash::new_unique();

    let first = build_escrow_bound_transaction(&payer, &recipient, 7, blockhash, "escrow-a")
        .expect("first transaction");
    let second = build_escrow_bound_transaction(&payer, &recipient, 7, blockhash, "escrow-b")
        .expect("second transaction");

    assert_ne!(first.signatures[0], second.signatures[0]);
}

#[test]
fn expired_persisted_transaction_is_not_resubmittable() {
    let error = require_resubmittable_blockhash(false).expect_err("expired blockhash must fail");

    assert_eq!(error, "SETTLEMENT_TRANSACTION_EXPIRED");
}

#[test]
fn valid_persisted_transaction_remains_resubmittable() {
    require_resubmittable_blockhash(true).expect("valid blockhash should remain resubmittable");
}

#[test]
fn persisted_transaction_integrity_rejects_tampering() {
    let payer = Keypair::new();
    let recipient = Pubkey::new_unique();
    let mut transaction =
        build_escrow_bound_transaction(&payer, &recipient, 7, Hash::new_unique(), "escrow-a")
            .expect("transaction");
    let expected = transaction.signatures[0].to_string();
    transaction.message.instructions[0].data.push(0);
    let json = serde_json::to_string(&transaction).expect("json");

    let error = validate_persisted_transaction(json.as_str(), expected.as_str())
        .expect_err("tampering must fail");
    assert!(error.contains("integrity"));
}

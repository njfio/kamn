use super::transaction::{build_escrow_bound_transaction, validate_persisted_transaction};
use super::*;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{hash::Hash, pubkey::Pubkey, signature::Keypair};

#[test]
fn pending_signature_is_distinct_from_absent_signature() {
    assert_ne!(
        SignatureReconciliation::Pending,
        SignatureReconciliation::Absent
    );
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

#[test]
fn persistence_failure_prevents_settlement_submission() {
    let _guard = super::super::test_support::set_test_live_solana_settlement_override(true);
    let config = test_config();
    let prepared = super::super::test_support::maybe_prepare_test_live_settlement(
        &config,
        "escrow-persist-failure",
    )
    .expect("test override")
    .expect("prepared settlement");
    let mut reject = || Err("SETTLEMENT_SUBMISSION_PERSISTENCE_FAILED: injected".to_owned());

    let error = submit_or_reconcile_live_settlement(
        &config,
        &prepared,
        "escrow-persist-failure",
        &mut reject,
    )
    .expect_err("persistence failure must prevent submission");

    assert!(error.starts_with("SETTLEMENT_SUBMISSION_PERSISTENCE_FAILED"));
    assert_eq!(
        super::super::test_support::test_live_solana_settlement_submission_count(),
        0
    );
}

fn test_config() -> LiveSolanaSettlementConfig {
    LiveSolanaSettlementConfig {
        rpc_url: "http://127.0.0.1:1".to_owned(),
        keypair_file: "unused".to_owned(),
        recipient_pubkey: Pubkey::new_unique(),
        lamports: 1,
        commitment: CommitmentConfig::finalized(),
        commitment_label: "finalized".to_owned(),
    }
}

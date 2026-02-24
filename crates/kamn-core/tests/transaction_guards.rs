use kamn_core::{
    baseline_signature_for_fields, legacy_signature_for_fields,
    service_auth_public_key_hex_from_private_key_hex,
    signature_profile_compatibility_fixtures_for_fields, BaselineTransaction, RoleSmokeNetwork,
    SmokeError, TransactionGuardError, GENESIS_STATE_HASH,
};
use std::sync::{Mutex, OnceLock};

const TEST_SIGNER_PRIVATE_KEY_A_HEX: &str =
    "7f2dcf2ef6bcf53b1af2359954f04eb6d25688fd87cbf09f7f9db4c6522f4c6b";
const TEST_SIGNER_PRIVATE_KEY_B_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

fn signer_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_signer_env() -> std::sync::MutexGuard<'static, ()> {
    signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = std::env::var(key).ok();
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

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
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
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
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
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
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
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
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
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

#[test]
fn regression_transaction_guard_rejects_signature_when_public_key_mismatch_is_forced() {
    // Regression: #5897
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
    let _private_key_guard = EnvVarGuard::set(
        "KAMN_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_SIGNER_PRIVATE_KEY_A_HEX),
    );
    let mismatched_public_key =
        service_auth_public_key_hex_from_private_key_hex(TEST_SIGNER_PRIVATE_KEY_B_HEX)
            .expect("test key should decode");
    let _public_key_guard = EnvVarGuard::set(
        "KAMN_SIGNER_PUBLIC_KEY_HEX",
        Some(mismatched_public_key.as_str()),
    );

    let mut network = RoleSmokeNetwork::new(true);
    let tx = BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-tx-1", GENESIS_STATE_HASH);

    assert!(matches!(
        network.submit_transaction(tx),
        Err(SmokeError::Guard(
            TransactionGuardError::InvalidSignature { .. }
        ))
    ));
}

#[test]
fn regression_signature_profile_matches_transaction_expected_signature() {
    // Regression: #400
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
    let tx = BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-tx-1", GENESIS_STATE_HASH);
    assert_eq!(tx.signature, tx.expected_signature());
    assert!(
        tx.expected_signature()
            .starts_with("sig:secp256k1:baseline-v2:"),
        "transaction signatures must default to cryptographic baseline-v2 format"
    );
}

#[test]
fn regression_non_versioned_signature_profile_is_rejected() {
    // Regression: #404
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
    let mut network = RoleSmokeNetwork::new(true);
    let mut tx =
        BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-tx-1", GENESIS_STATE_HASH);
    tx.signature = legacy_signature_for_fields("agent-a", 1, GENESIS_STATE_HASH, "payload-tx-1");

    assert!(matches!(
        network.submit_transaction(tx),
        Err(SmokeError::Guard(
            TransactionGuardError::InvalidSignature { .. }
        ))
    ));
}

#[test]
fn regression_signature_profile_fixture_matrix_matches_transaction_guard_expectations() {
    // Regression: #677
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", None);
    let fixtures = signature_profile_compatibility_fixtures_for_fields(
        "agent-a",
        1,
        GENESIS_STATE_HASH,
        "payload-tx-1",
    );

    for fixture in fixtures {
        let mut network = RoleSmokeNetwork::new(true);
        let mut tx =
            BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-tx-1", GENESIS_STATE_HASH);
        tx.signature = fixture.signature.clone();
        let accepted = network.submit_transaction(tx).is_ok();
        assert_eq!(
            accepted, false,
            "transaction guard compatibility fixture {} should be rejected in default fail-closed mode",
            fixture.fixture_id
        );
    }
}

#[test]
fn regression_transaction_guard_accepts_baseline_v1_only_with_explicit_compat_switch() {
    // Regression: #5897
    let _lock = lock_signer_env();
    let _compat_guard = EnvVarGuard::set("KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1", Some("1"));

    let mut network = RoleSmokeNetwork::new(true);
    let mut tx =
        BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-tx-1", GENESIS_STATE_HASH);
    tx.signature = baseline_signature_for_fields("agent-a", 1, GENESIS_STATE_HASH, "payload-tx-1");

    assert!(
        network.submit_transaction(tx).is_ok(),
        "baseline-v1 signatures should only be accepted when explicit compatibility switch is enabled"
    );
}

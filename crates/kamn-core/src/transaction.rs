use crate::signature_profile::{
    debug_fallback_signer_private_key_hex, service_auth_public_key_hex_from_private_key_hex,
    service_auth_sign_with_private_key_hex, service_auth_verify_with_public_key_hex,
    signature_matches_supported_profile_for_fields, SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV,
    SERVICE_AUTH_SIGNATURE_PUBLIC_KEY_ENV,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;

/// Initial expected state hash before any block commits.
pub const GENESIS_STATE_HASH: &str = "state:genesis";
const SIGNER_PRIVATE_KEY_ENV: &str = "KAMN_SIGNER_PRIVATE_KEY_HEX";
const SIGNER_PUBLIC_KEY_ENV: &str = "KAMN_SIGNER_PUBLIC_KEY_HEX";
const SIGNER_LEGACY_BASELINE_V1_COMPAT_ENV: &str = "KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1";
const SIGNING_KEY_ERROR_SIGNATURE_ALGORITHM: &str = "signing-key-invalid";
const TRANSACTION_GUARDS_MAX_TRACKED_TX_IDS: usize = 100_000;
const TRANSACTION_GUARDS_MAX_TRACKED_SENDERS: usize = 10_000;

/// Baseline transaction payload used by transaction guard validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineTransaction {
    /// Unique transaction identifier.
    pub id: String,
    /// Sender identifier.
    pub sender: String,
    /// Sender nonce expected to increase sequentially.
    pub nonce: u64,
    /// Serialized payload content.
    pub payload: String,
    /// State hash the transaction expects to build upon.
    pub state_hash: String,
    /// Transaction signature for baseline profile validation.
    pub signature: String,
}

impl BaselineTransaction {
    /// Creates a transaction and fills a baseline signature for its fields.
    pub fn signed(id: &str, sender: &str, nonce: u64, payload: &str, state_hash: &str) -> Self {
        let mut tx = Self {
            id: id.to_owned(),
            sender: sender.to_owned(),
            nonce,
            payload: payload.to_owned(),
            state_hash: state_hash.to_owned(),
            signature: String::new(),
        };
        tx.signature = tx.expected_signature();
        tx
    }

    /// Computes the expected baseline signature for this transaction.
    pub fn expected_signature(&self) -> String {
        if let Some(private_key_hex) = resolve_transaction_signer_private_key_hex() {
            if let Ok(signature) = service_auth_sign_with_private_key_hex(
                &self.sender,
                self.nonce,
                &self.state_hash,
                &self.payload,
                private_key_hex.as_str(),
            ) {
                return signature;
            }
            return signing_key_error_signature_for_fields(
                &self.sender,
                self.nonce,
                &self.state_hash,
                &self.payload,
            );
        }
        signing_key_error_signature_for_fields(
            &self.sender,
            self.nonce,
            &self.state_hash,
            &self.payload,
        )
    }

    fn validate_shape(&self) -> Result<(), TransactionGuardError> {
        if self.id.trim().is_empty() {
            return Err(TransactionGuardError::EmptyField("id"));
        }
        if self.sender.trim().is_empty() {
            return Err(TransactionGuardError::EmptyField("sender"));
        }
        if self.nonce == 0 {
            return Err(TransactionGuardError::InvalidNonce(self.nonce));
        }
        if self.payload.trim().is_empty() {
            return Err(TransactionGuardError::EmptyField("payload"));
        }
        if self.state_hash.trim().is_empty() {
            return Err(TransactionGuardError::EmptyField("state_hash"));
        }
        if self.signature.trim().is_empty() {
            return Err(TransactionGuardError::EmptyField("signature"));
        }
        Ok(())
    }
}

fn signing_key_error_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sig:{SIGNING_KEY_ERROR_SIGNATURE_ALGORITHM}:baseline-v2:{sender}:{nonce}:{state_hash}:{}",
        payload.len()
    )
}

fn signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value(
    debug_assertions: bool,
    env_value: Option<&str>,
) -> bool {
    if !debug_assertions {
        return false;
    }
    match env_value {
        Some(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        None => false,
    }
}

fn signer_legacy_baseline_v1_compat_enabled_for_mode(debug_assertions: bool) -> bool {
    let env_value = env::var(SIGNER_LEGACY_BASELINE_V1_COMPAT_ENV).ok();
    signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value(
        debug_assertions,
        env_value.as_deref(),
    )
}

fn signer_legacy_baseline_v1_compat_enabled() -> bool {
    signer_legacy_baseline_v1_compat_enabled_for_mode(cfg!(debug_assertions))
}

fn resolve_transaction_signer_private_key_hex_for_mode(debug_assertions: bool) -> Option<String> {
    for env_name in [
        SIGNER_PRIVATE_KEY_ENV,
        SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV,
    ] {
        if let Ok(value) = env::var(env_name) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_owned());
            }
        }
    }

    if debug_assertions {
        if let Some(private_key_hex) = debug_fallback_signer_private_key_hex() {
            return Some(private_key_hex.to_owned());
        }
    }

    None
}

fn resolve_transaction_signer_private_key_hex() -> Option<String> {
    resolve_transaction_signer_private_key_hex_for_mode(cfg!(debug_assertions))
}

fn resolve_transaction_signer_public_key_hex() -> Option<String> {
    for env_name in [SIGNER_PUBLIC_KEY_ENV, SERVICE_AUTH_SIGNATURE_PUBLIC_KEY_ENV] {
        if let Ok(value) = env::var(env_name) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_owned());
            }
        }
    }

    let private_key_hex = resolve_transaction_signer_private_key_hex()?;
    service_auth_public_key_hex_from_private_key_hex(private_key_hex.as_str()).ok()
}

pub(crate) fn signature_matches_transaction_contract(tx: &BaselineTransaction) -> bool {
    if let Some(public_key_hex) = resolve_transaction_signer_public_key_hex() {
        if service_auth_verify_with_public_key_hex(
            tx.signature.as_str(),
            tx.sender.as_str(),
            tx.nonce,
            tx.state_hash.as_str(),
            tx.payload.as_str(),
            public_key_hex.as_str(),
        )
        .is_ok()
        {
            return true;
        }
    }

    signer_legacy_baseline_v1_compat_enabled()
        && signature_matches_supported_profile_for_fields(
            tx.signature.as_str(),
            tx.sender.as_str(),
            tx.nonce,
            tx.state_hash.as_str(),
            tx.payload.as_str(),
        )
}

/// Guard engine that validates transaction shape, signature, nonce, and state continuity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionGuards {
    expected_state_hash: String,
    seen_tx_ids: BTreeSet<String>,
    next_nonce_by_sender: BTreeMap<String, u64>,
    max_tracked_tx_ids: usize,
    max_tracked_senders: usize,
}

impl Default for TransactionGuards {
    fn default() -> Self {
        Self {
            expected_state_hash: GENESIS_STATE_HASH.to_owned(),
            seen_tx_ids: BTreeSet::new(),
            next_nonce_by_sender: BTreeMap::new(),
            max_tracked_tx_ids: TRANSACTION_GUARDS_MAX_TRACKED_TX_IDS,
            max_tracked_senders: TRANSACTION_GUARDS_MAX_TRACKED_SENDERS,
        }
    }
}

impl TransactionGuards {
    /// Creates a new guard engine initialized at `GENESIS_STATE_HASH`.
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn with_limits(max_tracked_tx_ids: usize, max_tracked_senders: usize) -> Self {
        Self {
            max_tracked_tx_ids,
            max_tracked_senders,
            ..Self::default()
        }
    }

    /// Returns the state hash expected by the next transaction validation.
    pub fn expected_state_hash(&self) -> &str {
        &self.expected_state_hash
    }

    /// Validates a transaction against guard rules and records nonce/id progression.
    pub fn validate_and_record(
        &mut self,
        tx: &BaselineTransaction,
    ) -> Result<(), TransactionGuardError> {
        tx.validate_shape()?;

        if tx.state_hash != self.expected_state_hash {
            return Err(TransactionGuardError::StateHashMismatch {
                expected: self.expected_state_hash.clone(),
                found: tx.state_hash.clone(),
            });
        }

        if !signature_matches_transaction_contract(tx) {
            return Err(TransactionGuardError::InvalidSignature {
                tx_id: tx.id.clone(),
                expected: tx.expected_signature(),
                found: tx.signature.clone(),
            });
        }

        if self.seen_tx_ids.contains(&tx.id) {
            return Err(TransactionGuardError::DuplicateTransactionId(tx.id.clone()));
        }
        if self.seen_tx_ids.len() >= self.max_tracked_tx_ids {
            return Err(TransactionGuardError::ReplayWindowExhausted {
                tx_id: tx.id.clone(),
                max_tracked_tx_ids: self.max_tracked_tx_ids,
            });
        }

        if !self.next_nonce_by_sender.contains_key(&tx.sender)
            && self.next_nonce_by_sender.len() >= self.max_tracked_senders
        {
            return Err(TransactionGuardError::SenderWindowExhausted {
                sender: tx.sender.clone(),
                max_tracked_senders: self.max_tracked_senders,
            });
        }
        let expected_nonce = self
            .next_nonce_by_sender
            .get(&tx.sender)
            .copied()
            .unwrap_or(1);
        if tx.nonce != expected_nonce {
            return Err(TransactionGuardError::NonceOutOfSequence {
                sender: tx.sender.clone(),
                expected: expected_nonce,
                found: tx.nonce,
            });
        }

        self.seen_tx_ids.insert(tx.id.clone());
        self.next_nonce_by_sender
            .insert(tx.sender.clone(), tx.nonce + 1);

        Ok(())
    }

    /// Commits a validated block and advances expected state hash.
    pub fn commit_block(
        &mut self,
        transactions: &[BaselineTransaction],
    ) -> Result<(), TransactionGuardError> {
        for tx in transactions {
            if !self.seen_tx_ids.contains(&tx.id) {
                return Err(TransactionGuardError::UnvalidatedCommittedTransaction(
                    tx.id.clone(),
                ));
            }
        }

        if transactions.is_empty() {
            return Ok(());
        }

        let mut tx_digest = String::new();
        for tx in transactions {
            tx_digest.push('|');
            tx_digest.push_str(&tx.id);
            tx_digest.push(':');
            tx_digest.push_str(&tx.nonce.to_string());
        }

        self.expected_state_hash = format!("state:{}{}", self.expected_state_hash, tx_digest);
        Ok(())
    }
}

/// Errors emitted by transaction guard validation and commit flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionGuardError {
    /// Transaction id has already been recorded.
    DuplicateTransactionId(String),
    /// Required field was empty.
    EmptyField(&'static str),
    /// Nonce value was invalid.
    InvalidNonce(u64),
    /// Signature did not match baseline profile expectations.
    InvalidSignature {
        /// Transaction identifier.
        tx_id: String,
        /// Expected signature value.
        expected: String,
        /// Observed signature value.
        found: String,
    },
    /// Nonce did not match expected sender sequence.
    NonceOutOfSequence {
        /// Sender identifier.
        sender: String,
        /// Expected nonce value.
        expected: u64,
        /// Observed nonce value.
        found: u64,
    },
    /// Transaction state hash did not match current guard expectation.
    StateHashMismatch {
        /// Expected state hash.
        expected: String,
        /// Observed state hash.
        found: String,
    },
    /// Replay-window tracking reached configured transaction-id capacity.
    ReplayWindowExhausted {
        /// Incoming transaction identifier rejected by capacity guard.
        tx_id: String,
        /// Maximum tracked transaction-id window size.
        max_tracked_tx_ids: usize,
    },
    /// Sender nonce-window tracking reached configured sender capacity.
    SenderWindowExhausted {
        /// Incoming sender identifier rejected by capacity guard.
        sender: String,
        /// Maximum tracked sender window size.
        max_tracked_senders: usize,
    },
    /// Block commit attempted for a transaction that was never validated.
    UnvalidatedCommittedTransaction(String),
}

impl fmt::Display for TransactionGuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateTransactionId(tx_id) => write!(f, "duplicate transaction id: {tx_id}"),
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidNonce(value) => write!(f, "transaction nonce must be positive: {value}"),
            Self::InvalidSignature {
                tx_id,
                expected,
                found,
            } => write!(
                f,
                "invalid signature for {tx_id}; expected {expected}, found {found}"
            ),
            Self::NonceOutOfSequence {
                sender,
                expected,
                found,
            } => write!(
                f,
                "nonce out of sequence for sender {sender}; expected {expected}, found {found}"
            ),
            Self::StateHashMismatch { expected, found } => {
                write!(f, "state hash mismatch; expected {expected}, found {found}")
            }
            Self::ReplayWindowExhausted {
                tx_id,
                max_tracked_tx_ids,
            } => write!(
                f,
                "replay window exhausted at {max_tracked_tx_ids} tracked transaction ids; rejected {tx_id}"
            ),
            Self::SenderWindowExhausted {
                sender,
                max_tracked_senders,
            } => write!(
                f,
                "sender nonce window exhausted at {max_tracked_senders} tracked senders; rejected {sender}"
            ),
            Self::UnvalidatedCommittedTransaction(tx_id) => {
                write!(f, "committed transaction was not validated: {tx_id}")
            }
        }
    }
}

impl std::error::Error for TransactionGuardError {}

#[cfg(test)]
mod tests {
    use super::{
        resolve_transaction_signer_private_key_hex,
        resolve_transaction_signer_private_key_hex_for_mode,
        signer_legacy_baseline_v1_compat_enabled,
        signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value, BaselineTransaction,
        TransactionGuardError, TransactionGuards, GENESIS_STATE_HASH,
        SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV, SIGNER_LEGACY_BASELINE_V1_COMPAT_ENV,
        SIGNER_PRIVATE_KEY_ENV,
    };
    use std::sync::Mutex;

    const TEST_SIGNER_PRIVATE_KEY_HEX: &str =
        "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

    fn signer_env_lock() -> &'static Mutex<()> {
        crate::signer_test_env_lock()
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

    fn signed_tx(id: &str, sender: &str, nonce: u64, state_hash: &str) -> BaselineTransaction {
        BaselineTransaction::signed(id, sender, nonce, &format!("payload-{id}"), state_hash)
    }

    #[test]
    fn regression_legacy_baseline_compat_helper_fails_closed_for_non_debug_policy() {
        // Regression: #5911
        assert!(
            !signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value(false, Some("1")),
            "non-debug policy branch must not permit legacy baseline compatibility even with truthy env"
        );
    }

    #[test]
    fn regression_legacy_baseline_compat_helper_accepts_truthy_env_for_debug_policy() {
        // Regression: #5911
        assert!(
            signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value(true, Some("1")),
            "debug policy branch should preserve explicit legacy baseline compatibility opt-in"
        );
    }

    #[test]
    fn regression_legacy_baseline_compat_wrapper_defaults_false_without_env() {
        // Regression: #5911
        let _lock = lock_signer_env();
        let _compat_guard = EnvVarGuard::set(SIGNER_LEGACY_BASELINE_V1_COMPAT_ENV, None);
        assert!(
            !signer_legacy_baseline_v1_compat_enabled(),
            "wrapper should default to fail-closed when compatibility env is unset"
        );
    }

    #[test]
    fn regression_transaction_key_resolution_requires_explicit_env() {
        // Regression: #5913
        let _lock = lock_signer_env();
        let _private_key_guard = EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, None);
        let _service_private_key_guard =
            EnvVarGuard::set(SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV, None);
        assert!(
            resolve_transaction_signer_private_key_hex_for_mode(false).is_none(),
            "transaction key resolution must fail closed without explicit key env"
        );
    }

    #[test]
    fn regression_transaction_key_resolution_uses_explicit_env() {
        // Regression: #5913
        let _lock = lock_signer_env();
        let _private_key_guard =
            EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some(TEST_SIGNER_PRIVATE_KEY_HEX));
        let _service_private_key_guard =
            EnvVarGuard::set(SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV, None);
        assert_eq!(
            resolve_transaction_signer_private_key_hex().as_deref(),
            Some(TEST_SIGNER_PRIVATE_KEY_HEX),
            "transaction key resolution must honor explicit signer private key env"
        );
    }

    #[test]
    fn validates_signed_transaction() {
        let _lock = lock_signer_env();
        let _key_guard =
            EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some(TEST_SIGNER_PRIVATE_KEY_HEX));
        let mut guards = TransactionGuards::new();
        let tx = signed_tx("tx-1", "agent-a", 1, guards.expected_state_hash());

        assert!(guards.validate_and_record(&tx).is_ok());
    }

    #[test]
    fn rejects_invalid_signature() {
        let _lock = lock_signer_env();
        let _key_guard =
            EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some(TEST_SIGNER_PRIVATE_KEY_HEX));
        let mut guards = TransactionGuards::new();
        let mut tx = signed_tx("tx-1", "agent-a", 1, guards.expected_state_hash());
        tx.signature = "sig:tampered".to_owned();

        assert!(matches!(
            guards.validate_and_record(&tx),
            Err(TransactionGuardError::InvalidSignature { .. })
        ));
    }

    #[test]
    fn rejects_nonce_out_of_sequence() {
        let _lock = lock_signer_env();
        let _key_guard =
            EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some(TEST_SIGNER_PRIVATE_KEY_HEX));
        let mut guards = TransactionGuards::new();
        let tx = signed_tx("tx-1", "agent-a", 2, guards.expected_state_hash());

        assert_eq!(
            guards.validate_and_record(&tx),
            Err(TransactionGuardError::NonceOutOfSequence {
                sender: "agent-a".to_owned(),
                expected: 1,
                found: 2,
            })
        );
    }

    #[test]
    fn rejects_stale_state_hash() {
        let _lock = lock_signer_env();
        let _key_guard =
            EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some(TEST_SIGNER_PRIVATE_KEY_HEX));
        let mut guards = TransactionGuards::new();
        let tx1 = signed_tx("tx-1", "agent-a", 1, guards.expected_state_hash());
        guards
            .validate_and_record(&tx1)
            .expect("first transaction should validate");
        guards
            .commit_block(&[tx1])
            .expect("block commit should succeed");

        // Regression: #78
        let stale = signed_tx("tx-2", "agent-a", 2, GENESIS_STATE_HASH);
        assert!(matches!(
            guards.validate_and_record(&stale),
            Err(TransactionGuardError::StateHashMismatch { .. })
        ));
    }

    #[test]
    fn commit_advances_expected_state_hash() {
        let _lock = lock_signer_env();
        let _key_guard =
            EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some(TEST_SIGNER_PRIVATE_KEY_HEX));
        let mut guards = TransactionGuards::new();
        let initial = guards.expected_state_hash().to_owned();
        let tx = signed_tx("tx-1", "agent-a", 1, guards.expected_state_hash());
        guards
            .validate_and_record(&tx)
            .expect("transaction should validate");

        guards
            .commit_block(&[tx])
            .expect("block commit should succeed");
        assert_ne!(guards.expected_state_hash(), initial);
    }

    #[test]
    fn regression_expected_signature_fails_closed_for_invalid_key_material() {
        // Regression: #5899
        let _lock = lock_signer_env();
        let _key_guard = EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some("invalid-hex"));
        let tx = BaselineTransaction {
            id: "tx-invalid-key".to_owned(),
            sender: "agent-a".to_owned(),
            nonce: 1,
            payload: "payload-invalid-key".to_owned(),
            state_hash: GENESIS_STATE_HASH.to_owned(),
            signature: String::new(),
        };

        let expected = tx.expected_signature();
        assert!(
            expected.starts_with("sig:signing-key-invalid:baseline-v2:"),
            "invalid key material must not silently downgrade to deterministic baseline signature: {expected}"
        );
    }

    #[test]
    fn regression_expected_signature_fails_closed_without_signer_key_material() {
        // Regression: #5916
        let _lock = lock_signer_env();
        let _key_guard = EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, None);
        let _service_key_guard = EnvVarGuard::set(SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV, None);
        let tx = BaselineTransaction {
            id: "tx-missing-key".to_owned(),
            sender: "agent-a".to_owned(),
            nonce: 1,
            payload: "payload-missing-key".to_owned(),
            state_hash: GENESIS_STATE_HASH.to_owned(),
            signature: String::new(),
        };

        let expected = tx.expected_signature();
        assert!(
            !expected.starts_with("sig:deterministic-v1:baseline-v1:"),
            "missing key material must not downgrade to deterministic baseline-v1 signatures: {expected}"
        );
    }

    #[test]
    fn regression_replay_window_capacity_rejects_new_transaction_ids() {
        // Regression: #5899
        let _lock = lock_signer_env();
        let _key_guard =
            EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some(TEST_SIGNER_PRIVATE_KEY_HEX));
        let mut guards = TransactionGuards::with_limits(1, 4);
        let first = signed_tx("tx-1", "agent-a", 1, guards.expected_state_hash());
        guards
            .validate_and_record(&first)
            .expect("first transaction should fit in replay window");

        let second = signed_tx("tx-2", "agent-a", 2, guards.expected_state_hash());
        assert_eq!(
            guards.validate_and_record(&second),
            Err(TransactionGuardError::ReplayWindowExhausted {
                tx_id: "tx-2".to_owned(),
                max_tracked_tx_ids: 1,
            })
        );
    }

    #[test]
    fn regression_sender_window_capacity_rejects_new_senders() {
        // Regression: #5899
        let _lock = lock_signer_env();
        let _key_guard =
            EnvVarGuard::set(SIGNER_PRIVATE_KEY_ENV, Some(TEST_SIGNER_PRIVATE_KEY_HEX));
        let mut guards = TransactionGuards::with_limits(8, 1);
        let first = signed_tx("tx-1", "agent-a", 1, guards.expected_state_hash());
        guards
            .validate_and_record(&first)
            .expect("first sender should fit in sender window");

        let second = signed_tx("tx-2", "agent-b", 1, guards.expected_state_hash());
        assert_eq!(
            guards.validate_and_record(&second),
            Err(TransactionGuardError::SenderWindowExhausted {
                sender: "agent-b".to_owned(),
                max_tracked_senders: 1,
            })
        );
    }
}

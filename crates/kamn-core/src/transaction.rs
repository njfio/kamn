use crate::signature_profile::{
    baseline_signature_for_fields, signature_matches_supported_profile_for_fields,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Initial expected state hash before any block commits.
pub const GENESIS_STATE_HASH: &str = "state:genesis";

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
        baseline_signature_for_fields(&self.sender, self.nonce, &self.state_hash, &self.payload)
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

/// Guard engine that validates transaction shape, signature, nonce, and state continuity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionGuards {
    expected_state_hash: String,
    seen_tx_ids: BTreeSet<String>,
    next_nonce_by_sender: BTreeMap<String, u64>,
}

impl Default for TransactionGuards {
    fn default() -> Self {
        Self {
            expected_state_hash: GENESIS_STATE_HASH.to_owned(),
            seen_tx_ids: BTreeSet::new(),
            next_nonce_by_sender: BTreeMap::new(),
        }
    }
}

impl TransactionGuards {
    /// Creates a new guard engine initialized at `GENESIS_STATE_HASH`.
    pub fn new() -> Self {
        Self::default()
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

        if !signature_matches_supported_profile_for_fields(
            &tx.signature,
            &tx.sender,
            tx.nonce,
            &tx.state_hash,
            &tx.payload,
        ) {
            return Err(TransactionGuardError::InvalidSignature {
                tx_id: tx.id.clone(),
                expected: tx.expected_signature(),
                found: tx.signature.clone(),
            });
        }

        if self.seen_tx_ids.contains(&tx.id) {
            return Err(TransactionGuardError::DuplicateTransactionId(tx.id.clone()));
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
        BaselineTransaction, TransactionGuardError, TransactionGuards, GENESIS_STATE_HASH,
    };

    fn signed_tx(id: &str, sender: &str, nonce: u64, state_hash: &str) -> BaselineTransaction {
        BaselineTransaction::signed(id, sender, nonce, &format!("payload-{id}"), state_hash)
    }

    #[test]
    fn validates_signed_transaction() {
        let mut guards = TransactionGuards::new();
        let tx = signed_tx("tx-1", "agent-a", 1, guards.expected_state_hash());

        assert!(guards.validate_and_record(&tx).is_ok());
    }

    #[test]
    fn rejects_invalid_signature() {
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
}

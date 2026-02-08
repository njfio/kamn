use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const GENESIS_STATE_HASH: &str = "state:genesis";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineTransaction {
    pub id: String,
    pub sender: String,
    pub nonce: u64,
    pub payload: String,
    pub state_hash: String,
    pub signature: String,
}

impl BaselineTransaction {
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

    pub fn expected_signature(&self) -> String {
        // Lightweight deterministic placeholder for baseline integrity checks.
        format!(
            "sig:{}:{}:{}:{}",
            self.sender,
            self.nonce,
            self.state_hash,
            self.payload.len()
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expected_state_hash(&self) -> &str {
        &self.expected_state_hash
    }

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

        if tx.signature != tx.expected_signature() {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionGuardError {
    DuplicateTransactionId(String),
    EmptyField(&'static str),
    InvalidNonce(u64),
    InvalidSignature {
        tx_id: String,
        expected: String,
        found: String,
    },
    NonceOutOfSequence {
        sender: String,
        expected: u64,
        found: u64,
    },
    StateHashMismatch {
        expected: String,
        found: String,
    },
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

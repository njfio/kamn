//! Deterministic triadic smoke network used by runtime contract tests.

use std::fmt;

use crate::config::NodeRole;
use crate::transaction::{BaselineTransaction, TransactionGuardError, TransactionGuards};

/// Block produced by the processor role during smoke simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducedBlock {
    /// Sequential block height.
    pub height: u64,
    /// Producer role that emitted the block.
    pub producer: NodeRole,
    /// Ordered transactions committed into the block.
    pub transactions: Vec<BaselineTransaction>,
}

/// In-memory role state tracked during smoke simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSmokeState {
    /// Node role represented by this state record.
    pub role: NodeRole,
    mempool: Vec<BaselineTransaction>,
    committed_tx_ids: Vec<String>,
}

impl NodeSmokeState {
    /// Returns current number of pending mempool transactions.
    pub fn mempool_len(&self) -> usize {
        self.mempool.len()
    }

    /// Returns number of committed transaction IDs.
    pub fn committed_len(&self) -> usize {
        self.committed_tx_ids.len()
    }

    /// Returns true when the transaction appears in mempool or committed history.
    pub fn has_seen_transaction(&self, tx_id: &str) -> bool {
        self.mempool.iter().any(|tx| tx.id == tx_id)
            || self.committed_tx_ids.iter().any(|seen_id| seen_id == tx_id)
    }

    fn push_mempool(&mut self, tx: BaselineTransaction) {
        self.mempool.push(tx);
    }

    fn commit_transactions(&mut self, txs: &[BaselineTransaction]) {
        for tx in txs {
            self.committed_tx_ids.push(tx.id.clone());
        }
    }
}

/// Triadic runtime smoke network with processor/listener/approver roles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleSmokeNetwork {
    /// Processor role state.
    pub processor: NodeSmokeState,
    /// Listener role state.
    pub listener: NodeSmokeState,
    /// Approver role state.
    pub approver: NodeSmokeState,
    /// Enables transaction gossip from processor to listener/approver.
    pub gossip_enabled: bool,
    guards: TransactionGuards,
    next_height: u64,
}

impl RoleSmokeNetwork {
    /// Creates a new triadic smoke network with optional gossip behavior.
    pub fn new(gossip_enabled: bool) -> Self {
        Self {
            processor: NodeSmokeState {
                role: NodeRole::Processor,
                mempool: Vec::new(),
                committed_tx_ids: Vec::new(),
            },
            listener: NodeSmokeState {
                role: NodeRole::Listener,
                mempool: Vec::new(),
                committed_tx_ids: Vec::new(),
            },
            approver: NodeSmokeState {
                role: NodeRole::Approver,
                mempool: Vec::new(),
                committed_tx_ids: Vec::new(),
            },
            gossip_enabled,
            guards: TransactionGuards::new(),
            next_height: 1,
        }
    }

    /// Returns expected state hash required for the next accepted transaction.
    pub fn expected_state_hash(&self) -> &str {
        self.guards.expected_state_hash()
    }

    /// Validates and submits a transaction into processor state.
    pub fn submit_transaction(&mut self, tx: BaselineTransaction) -> Result<(), SmokeError> {
        self.guards.validate_and_record(&tx)?;

        self.processor.push_mempool(tx.clone());
        if self.gossip_enabled {
            self.listener.push_mempool(tx.clone());
            self.approver.push_mempool(tx);
        }

        Ok(())
    }

    /// Produces a block from processor mempool and advances network height.
    pub fn produce_block(&mut self) -> Result<ProducedBlock, SmokeError> {
        if self.processor.mempool.is_empty() {
            return Err(SmokeError::EmptyMempool(NodeRole::Processor));
        }

        self.processor
            .mempool
            .sort_by(|lhs, rhs| lhs.nonce.cmp(&rhs.nonce).then(lhs.id.cmp(&rhs.id)));
        let transactions = std::mem::take(&mut self.processor.mempool);
        self.processor.commit_transactions(&transactions);
        self.guards.commit_block(&transactions)?;

        if self.gossip_enabled {
            self.listener.mempool.clear();
            self.approver.mempool.clear();
            self.listener.commit_transactions(&transactions);
            self.approver.commit_transactions(&transactions);
        }

        let block = ProducedBlock {
            height: self.next_height,
            producer: NodeRole::Processor,
            transactions,
        };
        self.next_height += 1;

        Ok(block)
    }

    /// Returns true when listener and approver have both observed the transaction.
    pub fn gossip_reached_all_roles(&self, tx_id: &str) -> bool {
        self.listener.has_seen_transaction(tx_id) && self.approver.has_seen_transaction(tx_id)
    }
}

/// Smoke network error type surfaced by submit/produce operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmokeError {
    /// Attempted block production with an empty processor mempool.
    EmptyMempool(NodeRole),
    /// Transaction guard rejected an operation.
    Guard(TransactionGuardError),
}

impl From<TransactionGuardError> for SmokeError {
    fn from(error: TransactionGuardError) -> Self {
        Self::Guard(error)
    }
}

impl fmt::Display for SmokeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMempool(role) => {
                write!(f, "no pending transactions in {} mempool", role.as_str())
            }
            Self::Guard(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for SmokeError {}

#[cfg(test)]
mod tests {
    use super::{RoleSmokeNetwork, SmokeError};
    use crate::config::NodeRole;
    use crate::transaction::{BaselineTransaction, TransactionGuardError};

    fn sample_tx(
        id: &str,
        sender: &str,
        nonce: u64,
        state_hash: &str,
        payload: &str,
    ) -> BaselineTransaction {
        BaselineTransaction::signed(id, sender, nonce, payload, state_hash)
    }

    #[test]
    fn rejects_invalid_transaction_payload() {
        let mut network = RoleSmokeNetwork::new(true);
        let state_hash = network.expected_state_hash().to_owned();
        let tx = sample_tx("tx-1", "agent-a", 1, &state_hash, "");

        assert_eq!(
            network.submit_transaction(tx),
            Err(SmokeError::Guard(TransactionGuardError::EmptyField(
                "payload"
            )))
        );
    }

    #[test]
    fn rejects_duplicate_transactions() {
        let mut network = RoleSmokeNetwork::new(true);
        let state_hash = network.expected_state_hash().to_owned();
        network
            .submit_transaction(sample_tx("tx-1", "agent-a", 1, &state_hash, "payload-1"))
            .expect("first transaction should be accepted");

        let second_state_hash = network.expected_state_hash().to_owned();
        assert_eq!(
            network.submit_transaction(sample_tx(
                "tx-1",
                "agent-b",
                1,
                &second_state_hash,
                "payload-2"
            )),
            Err(SmokeError::Guard(
                TransactionGuardError::DuplicateTransactionId("tx-1".to_owned())
            ))
        );
    }

    #[test]
    fn produce_block_orders_transactions_by_nonce() {
        let mut network = RoleSmokeNetwork::new(true);
        let state_hash = network.expected_state_hash().to_owned();
        network
            .submit_transaction(sample_tx("tx-2", "agent-a", 1, &state_hash, "payload-2"))
            .expect("first submit works");
        network
            .submit_transaction(sample_tx("tx-1", "agent-b", 1, &state_hash, "payload-1"))
            .expect("second submit works");

        let block = network
            .produce_block()
            .expect("block production should succeed");
        assert_eq!(block.producer, NodeRole::Processor);
        assert_eq!(block.transactions[0].id, "tx-1");
        assert_eq!(block.transactions[1].id, "tx-2");
    }

    #[test]
    fn produce_block_requires_processor_transactions() {
        let mut network = RoleSmokeNetwork::new(true);
        assert_eq!(
            network.produce_block(),
            Err(SmokeError::EmptyMempool(NodeRole::Processor))
        );
    }
}

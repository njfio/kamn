use std::fmt;

use crate::config::NodeRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineTransaction {
    pub id: String,
    pub nonce: u64,
    pub payload: String,
}

impl BaselineTransaction {
    pub fn validate(&self) -> Result<(), SmokeError> {
        if self.id.trim().is_empty() {
            return Err(SmokeError::EmptyTransactionId);
        }
        if self.nonce == 0 {
            return Err(SmokeError::InvalidNonce(self.nonce));
        }
        if self.payload.trim().is_empty() {
            return Err(SmokeError::EmptyPayload);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducedBlock {
    pub height: u64,
    pub producer: NodeRole,
    pub transactions: Vec<BaselineTransaction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSmokeState {
    pub role: NodeRole,
    mempool: Vec<BaselineTransaction>,
    committed_tx_ids: Vec<String>,
}

impl NodeSmokeState {
    pub fn mempool_len(&self) -> usize {
        self.mempool.len()
    }

    pub fn committed_len(&self) -> usize {
        self.committed_tx_ids.len()
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleSmokeNetwork {
    pub processor: NodeSmokeState,
    pub listener: NodeSmokeState,
    pub approver: NodeSmokeState,
    pub gossip_enabled: bool,
    next_height: u64,
}

impl RoleSmokeNetwork {
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
            next_height: 1,
        }
    }

    pub fn submit_transaction(&mut self, tx: BaselineTransaction) -> Result<(), SmokeError> {
        tx.validate()?;

        if self.processor.has_seen_transaction(&tx.id)
            || self.listener.has_seen_transaction(&tx.id)
            || self.approver.has_seen_transaction(&tx.id)
        {
            return Err(SmokeError::DuplicateTransaction(tx.id));
        }

        self.processor.push_mempool(tx.clone());
        if self.gossip_enabled {
            self.listener.push_mempool(tx.clone());
            self.approver.push_mempool(tx);
        }

        Ok(())
    }

    pub fn produce_block(&mut self) -> Result<ProducedBlock, SmokeError> {
        if self.processor.mempool.is_empty() {
            return Err(SmokeError::EmptyMempool(NodeRole::Processor));
        }

        self.processor
            .mempool
            .sort_by(|lhs, rhs| lhs.nonce.cmp(&rhs.nonce).then(lhs.id.cmp(&rhs.id)));
        let transactions = std::mem::take(&mut self.processor.mempool);
        self.processor.commit_transactions(&transactions);

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

    pub fn gossip_reached_all_roles(&self, tx_id: &str) -> bool {
        self.listener.has_seen_transaction(tx_id) && self.approver.has_seen_transaction(tx_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmokeError {
    DuplicateTransaction(String),
    EmptyMempool(NodeRole),
    EmptyPayload,
    EmptyTransactionId,
    InvalidNonce(u64),
}

impl fmt::Display for SmokeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateTransaction(id) => write!(f, "duplicate transaction id: {id}"),
            Self::EmptyMempool(role) => {
                write!(f, "no pending transactions in {} mempool", role.as_str())
            }
            Self::EmptyPayload => write!(f, "transaction payload must not be empty"),
            Self::EmptyTransactionId => write!(f, "transaction id must not be empty"),
            Self::InvalidNonce(value) => write!(f, "transaction nonce must be positive: {value}"),
        }
    }
}

impl std::error::Error for SmokeError {}

#[cfg(test)]
mod tests {
    use super::{BaselineTransaction, RoleSmokeNetwork, SmokeError};
    use crate::config::NodeRole;

    fn sample_tx(id: &str, nonce: u64) -> BaselineTransaction {
        BaselineTransaction {
            id: id.to_owned(),
            nonce,
            payload: format!("payload-{id}"),
        }
    }

    #[test]
    fn rejects_invalid_transaction_payload() {
        let tx = BaselineTransaction {
            id: "tx-1".to_owned(),
            nonce: 1,
            payload: String::new(),
        };

        assert_eq!(tx.validate(), Err(SmokeError::EmptyPayload));
    }

    #[test]
    fn rejects_duplicate_transactions() {
        let mut network = RoleSmokeNetwork::new(true);
        network
            .submit_transaction(sample_tx("tx-1", 1))
            .expect("first transaction should be accepted");

        assert_eq!(
            network.submit_transaction(sample_tx("tx-1", 2)),
            Err(SmokeError::DuplicateTransaction("tx-1".to_owned()))
        );
    }

    #[test]
    fn produce_block_orders_transactions_by_nonce() {
        let mut network = RoleSmokeNetwork::new(true);
        network
            .submit_transaction(sample_tx("tx-2", 2))
            .expect("first submit works");
        network
            .submit_transaction(sample_tx("tx-1", 1))
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

//! Mempool block production and consensus validation pipeline contracts.

use crate::runtime::{
    ApproverAttestation, ApproverQuorumDecision, ApproverQuorumError, ApproverQuorumEvaluator,
    ApproverQuorumInput, ListenerAttestation, ListenerQuorumDecision, ListenerQuorumError,
    ListenerQuorumEvaluator, ListenerQuorumInput,
};
use crate::smoke::{ProducedBlock, RoleSmokeNetwork, SmokeError};
use crate::transaction::BaselineTransaction;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Input contract for one consensus-validation and block-commit round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockConsensusRoundInput {
    /// Listener quorum event identifier.
    pub listener_event_id: String,
    /// Listener quorum event sequence.
    pub listener_event_sequence: u64,
    /// Approver outbound action identifier.
    pub outbound_action_id: String,
    /// Listener votes as `(listener_did, attestation_id)`.
    pub listener_votes: Vec<(String, String)>,
    /// Approver votes as `(approver_did, attestation_id, payload_digest_override)`.
    pub approver_votes: Vec<(String, String, Option<String>)>,
}

/// Commit report emitted when a consensus round commits a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPipelineCommitReport {
    /// Committed block payload.
    pub block: ProducedBlock,
    /// Listener quorum decision used for admission.
    pub listener_decision: ListenerQuorumDecision,
    /// Approver quorum decision used for authorization.
    pub approver_decision: ApproverQuorumDecision,
    /// Deterministic payload digest used by approver quorum validation.
    pub payload_digest: String,
}

/// Error variants for block pipeline validation and commit flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockPipelineError {
    /// Listener quorum validation failed.
    Listener(ListenerQuorumError),
    /// Approver quorum validation failed.
    Approver(ApproverQuorumError),
    /// Smoke network transport/guard operation failed.
    Smoke(SmokeError),
    /// Pipeline attempted a consensus round with an empty mempool.
    EmptyMempool,
    /// Approver payload digest override mismatched deterministic block digest.
    ConsensusPayloadDigestMismatch {
        /// Expected deterministic digest.
        expected: String,
        /// Found override digest.
        found: String,
    },
}

impl From<ListenerQuorumError> for BlockPipelineError {
    fn from(value: ListenerQuorumError) -> Self {
        Self::Listener(value)
    }
}

impl From<ApproverQuorumError> for BlockPipelineError {
    fn from(value: ApproverQuorumError) -> Self {
        Self::Approver(value)
    }
}

impl From<SmokeError> for BlockPipelineError {
    fn from(value: SmokeError) -> Self {
        Self::Smoke(value)
    }
}

impl Display for BlockPipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Listener(error) => write!(f, "{error}"),
            Self::Approver(error) => write!(f, "{error}"),
            Self::Smoke(error) => write!(f, "{error}"),
            Self::EmptyMempool => write!(
                f,
                "block pipeline requires at least one mempool transaction"
            ),
            Self::ConsensusPayloadDigestMismatch { expected, found } => {
                write!(
                    f,
                    "block pipeline payload digest mismatch: expected {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for BlockPipelineError {}

/// Deterministic mempool->consensus->commit pipeline for processor runtime flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolBlockPipeline {
    network: RoleSmokeNetwork,
    listener_evaluator: ListenerQuorumEvaluator,
    approver_required_approvals: usize,
}

impl MempoolBlockPipeline {
    /// Builds a new block pipeline with explicit listener/approver quorum thresholds.
    pub fn new(
        gossip_enabled: bool,
        listener_required_confirmations: usize,
        approver_required_approvals: usize,
    ) -> Result<Self, BlockPipelineError> {
        let listener_evaluator = ListenerQuorumEvaluator::new(listener_required_confirmations)?;
        // Validate approver threshold during construction.
        let _ = ApproverQuorumEvaluator::new(approver_required_approvals)?;

        Ok(Self {
            network: RoleSmokeNetwork::new(gossip_enabled),
            listener_evaluator,
            approver_required_approvals,
        })
    }

    /// Returns expected state hash for the next mempool submission.
    pub fn expected_state_hash(&self) -> &str {
        self.network.expected_state_hash()
    }

    /// Returns current processor mempool length.
    pub fn processor_mempool_len(&self) -> usize {
        self.network.processor.mempool_len()
    }

    /// Submits a transaction into the pipeline mempool using existing guardrails.
    pub fn submit_transaction(
        &mut self,
        tx: BaselineTransaction,
    ) -> Result<(), BlockPipelineError> {
        self.network.submit_transaction(tx)?;
        Ok(())
    }

    /// Runs listener/approver consensus validation and commits one produced block.
    pub fn run_consensus_round(
        &mut self,
        input: BlockConsensusRoundInput,
    ) -> Result<BlockPipelineCommitReport, BlockPipelineError> {
        let pending = self.network.processor_mempool_snapshot();
        if pending.is_empty() {
            return Err(BlockPipelineError::EmptyMempool);
        }

        let payload_digest = payload_digest_for_transactions(&pending);
        for (_, _, override_digest) in &input.approver_votes {
            if let Some(found) = override_digest {
                if found != &payload_digest {
                    return Err(BlockPipelineError::ConsensusPayloadDigestMismatch {
                        expected: payload_digest.clone(),
                        found: found.clone(),
                    });
                }
            }
        }

        let listener_attestations = input
            .listener_votes
            .into_iter()
            .map(|(listener_did, attestation_id)| {
                ListenerAttestation::new(&listener_did, &attestation_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let listener_input = ListenerQuorumInput::new(
            &input.listener_event_id,
            input.listener_event_sequence,
            listener_attestations,
        )?;
        let listener_decision = self.listener_evaluator.evaluate(listener_input)?;

        let approver_attestations = input
            .approver_votes
            .into_iter()
            .map(|(approver_did, attestation_id, override_digest)| {
                let digest = override_digest.unwrap_or_else(|| payload_digest.clone());
                ApproverAttestation::new(&approver_did, &digest, &attestation_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let approver_input = ApproverQuorumInput::new(
            &input.outbound_action_id,
            &payload_digest,
            approver_attestations,
        )?;
        let approver_evaluator = ApproverQuorumEvaluator::new(self.approver_required_approvals)?;
        let approver_decision = approver_evaluator.authorize(approver_input)?;

        let block = self.network.produce_block()?;

        Ok(BlockPipelineCommitReport {
            block,
            listener_decision,
            approver_decision,
            payload_digest,
        })
    }
}

fn payload_digest_for_transactions(transactions: &[BaselineTransaction]) -> String {
    let mut ordered = transactions.to_vec();
    ordered.sort_by(|left, right| {
        left.nonce
            .cmp(&right.nonce)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.sender.cmp(&right.sender))
    });

    let mut digest = String::from("block-payload");
    for tx in ordered {
        digest.push('|');
        digest.push_str(&tx.id);
        digest.push(':');
        digest.push_str(&tx.sender);
        digest.push(':');
        digest.push_str(&tx.nonce.to_string());
        digest.push(':');
        digest.push_str(&tx.state_hash);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::{payload_digest_for_transactions, BlockPipelineError, MempoolBlockPipeline};
    use crate::transaction::BaselineTransaction;

    #[test]
    fn constructor_rejects_zero_listener_quorum_threshold() {
        let result = MempoolBlockPipeline::new(true, 0, 1);
        assert!(matches!(result, Err(BlockPipelineError::Listener(_))));
    }

    #[test]
    fn constructor_rejects_zero_approver_quorum_threshold() {
        let result = MempoolBlockPipeline::new(true, 1, 0);
        assert!(matches!(result, Err(BlockPipelineError::Approver(_))));
    }

    #[test]
    fn regression_consensus_round_rejects_empty_mempool() {
        // Regression: #2927
        let mut pipeline = MempoolBlockPipeline::new(true, 1, 1).expect("pipeline builds");
        let result = pipeline.run_consensus_round(super::BlockConsensusRoundInput {
            listener_event_id: "event-1".to_owned(),
            listener_event_sequence: 1,
            outbound_action_id: "outbound-1".to_owned(),
            listener_votes: vec![("kamn:did:listener:alpha".to_owned(), "att-1".to_owned())],
            approver_votes: vec![(
                "kamn:did:agent:approver-alpha".to_owned(),
                "att-1".to_owned(),
                None,
            )],
        });
        assert_eq!(result, Err(BlockPipelineError::EmptyMempool));
    }

    #[test]
    fn payload_digest_is_deterministic_across_orderings() {
        let tx1 = BaselineTransaction::signed("tx-1", "agent-a", 1, "p1", "state:genesis");
        let tx2 = BaselineTransaction::signed("tx-2", "agent-b", 1, "p2", "state:genesis");
        let digest_a = payload_digest_for_transactions(&[tx1.clone(), tx2.clone()]);
        let digest_b = payload_digest_for_transactions(&[tx2, tx1]);
        assert_eq!(digest_a, digest_b);
    }
}

use crate::block_pipeline::block_pipeline_support::{
    CanonicalCommitRecord, ForkChoiceDecision, TransportMempoolFeed,
};
use crate::block_pipeline::commit_hooks::sorting::{
    payload_digest_for_transactions, sort_candidates_for_ingress,
};
use crate::block_pipeline::models::{
    BlockConsensusRoundInput, BlockPipelineCommitReport, BlockPipelineError,
};
use crate::runtime::{
    ApproverAttestation, ApproverQuorumEvaluator, ApproverQuorumInput, ListenerAttestation,
    ListenerQuorumEvaluator, ListenerQuorumInput,
};
use crate::smoke::RoleSmokeNetwork;
use crate::transaction::BaselineTransaction;

/// Deterministic mempool->consensus->commit pipeline for processor runtime flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolBlockPipeline {
    network: RoleSmokeNetwork,
    listener_evaluator: ListenerQuorumEvaluator,
    approver_required_approvals: usize,
}

impl MempoolBlockPipeline {
    pub fn new(
        gossip_enabled: bool,
        listener_required_confirmations: usize,
        approver_required_approvals: usize,
    ) -> Result<Self, BlockPipelineError> {
        let listener_evaluator = ListenerQuorumEvaluator::new(listener_required_confirmations)?;
        let _ = ApproverQuorumEvaluator::new(approver_required_approvals)?;
        Ok(Self {
            network: RoleSmokeNetwork::new(gossip_enabled),
            listener_evaluator,
            approver_required_approvals,
        })
    }

    pub fn expected_state_hash(&self) -> &str {
        self.network.expected_state_hash()
    }
    pub fn processor_mempool_len(&self) -> usize {
        self.network.processor.mempool_len()
    }

    pub fn submit_transaction(
        &mut self,
        tx: BaselineTransaction,
    ) -> Result<(), BlockPipelineError> {
        self.network.submit_transaction(tx)?;
        Ok(())
    }

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
